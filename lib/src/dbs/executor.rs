use crate::ctx::Context;
use crate::dbs::response::Response;
use crate::dbs::Auth;
use crate::dbs::Level;
use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Transaction;
use crate::err::Error;
use crate::kvs::Datastore;
use crate::sql::query::Query;
use crate::sql::statement::Statement;
use crate::sql::value::Value;
use futures::lock::Mutex;
use std::sync::Arc;
use trice::Instant;

pub struct Executor<'a> {
	err: bool,
	kvs: &'a Datastore,
	txn: Option<Transaction>,
}

impl<'a> Executor<'a> {
	pub fn new(kvs: &'a Datastore) -> Executor<'a> {
		Executor {
			kvs,
			txn: None,
			err: false,
		}
	}

	fn txn(&self) -> Transaction {
		match self.txn.as_ref() {
			Some(txn) => txn.clone(),
			None => unreachable!(),
		}
	}

	async fn begin(&mut self) -> bool {
		match self.txn.as_ref() {
			Some(_) => false,
			None => match self.kvs.transaction(true, false).await {
				Ok(v) => {
					self.txn = Some(Arc::new(Mutex::new(v)));
					true
				}
				Err(_) => {
					self.err = true;
					false
				}
			},
		}
	}

	async fn commit(&mut self, local: bool) {
		if local {
			if let Some(txn) = self.txn.as_ref() {
				match &self.err {
					true => {
						let txn = txn.clone();
						let mut txn = txn.lock().await;
						if txn.cancel().await.is_err() {
							self.err = true;
						}
						self.txn = None;
					}
					false => {
						let txn = txn.clone();
						let mut txn = txn.lock().await;
						if txn.commit().await.is_err() {
							self.err = true;
						}
						self.txn = None;
					}
				}
			}
		}
	}

	async fn cancel(&mut self, local: bool) {
		if local {
			match self.txn.as_ref() {
				Some(txn) => {
					let txn = txn.clone();
					let mut txn = txn.lock().await;
					if txn.cancel().await.is_err() {
						self.err = true;
					}
					self.txn = None;
				}
				None => unreachable!(),
			}
		}
	}

	fn buf_cancel(&self, v: Response) -> Response {
		Response {
			sql: v.sql,
			time: v.time,
			result: Err(Error::QueryCancelled),
		}
	}

	fn buf_commit(&self, v: Response) -> Response {
		match &self.err {
			true => Response {
				sql: v.sql,
				time: v.time,
				result: match v.result {
					Ok(_) => Err(Error::QueryNotExecuted),
					Err(e) => Err(e),
				},
			},
			_ => v,
		}
	}

	pub async fn execute(
		&mut self,
		mut ctx: Runtime,
		mut opt: Options,
		qry: Query,
	) -> Result<Vec<Response>, Error> {
		// Initialise buffer of responses
		let mut buf: Vec<Response> = vec![];
		// Initialise array of responses
		let mut out: Vec<Response> = vec![];
		// Process all statements in query
		for stm in qry.iter() {
			// Log the statement
			debug!("Executing: {}", stm);
			// Reset errors
			if self.txn.is_none() {
				self.err = false;
			}
			// Get the statement start time
			let now = Instant::now();
			// Process a single statement
			let res = match stm {
				// Specify runtime options
				Statement::Option(stm) => {
					// Allowed to run?
					opt.check(Level::Db)?;
					// Process the option
					match &stm.name.to_uppercase()[..] {
						"FIELDS" => opt = opt.fields(stm.what),
						"EVENTS" => opt = opt.events(stm.what),
						"TABLES" => opt = opt.tables(stm.what),
						"IMPORT" => opt = opt.import(stm.what),
						"FORCE" => opt = opt.force(stm.what),
						"DEBUG" => opt = opt.debug(stm.what),
						_ => break,
					}
					// Continue
					continue;
				}
				// Begin a new transaction
				Statement::Begin(_) => {
					self.begin().await;
					continue;
				}
				// Cancel a running transaction
				Statement::Cancel(_) => {
					self.cancel(true).await;
					buf = buf.into_iter().map(|v| self.buf_cancel(v)).collect();
					out.append(&mut buf);
					self.txn = None;
					continue;
				}
				// Commit a running transaction
				Statement::Commit(_) => {
					self.commit(true).await;
					buf = buf.into_iter().map(|v| self.buf_commit(v)).collect();
					out.append(&mut buf);
					self.txn = None;
					continue;
				}
				// Switch to a different NS or DB
				Statement::Use(stm) => {
					if let Some(ref ns) = stm.ns {
						match &*opt.auth {
							Auth::No => opt.ns = Some(Arc::new(ns.to_owned())),
							Auth::Kv => opt.ns = Some(Arc::new(ns.to_owned())),
							Auth::Ns(v) if v == ns => opt.ns = Some(Arc::new(ns.to_owned())),
							_ => {
								opt.ns = None;
								return Err(Error::NsNotAllowed {
									ns: ns.to_owned(),
								});
							}
						}
					}
					if let Some(ref db) = stm.db {
						match &*opt.auth {
							Auth::No => opt.db = Some(Arc::new(db.to_owned())),
							Auth::Kv => opt.db = Some(Arc::new(db.to_owned())),
							Auth::Ns(_) => opt.db = Some(Arc::new(db.to_owned())),
							Auth::Db(_, v) if v == db => opt.db = Some(Arc::new(db.to_owned())),
							_ => {
								opt.db = None;
								return Err(Error::DbNotAllowed {
									db: db.to_owned(),
								});
							}
						}
					}
					Ok(Value::None)
				}
				// Process param definition statements
				Statement::Set(stm) => {
					// Create a transaction
					let loc = self.begin().await;
					// Process the statement
					match stm.compute(&ctx, &opt, &self.txn(), None).await {
						Ok(val) => {
							let mut new = Context::new(&ctx);
							let key = stm.name.to_owned();
							new.add_value(key, val);
							ctx = new.freeze();
						}
						_ => break,
					}
					// Cancel transaction
					self.cancel(loc).await;
					// Return nothing
					Ok(Value::None)
				}
				// Process all other normal statements
				_ => match self.err {
					// This transaction has failed
					true => Err(Error::QueryNotExecuted),
					// Compute the statement normally
					false => {
						// Create a transaction
						let loc = self.begin().await;
						// Process the statement
						let res = match stm.timeout() {
							// There is a timeout clause
							Some(timeout) => {
								// Set statement timeout
								let mut ctx = Context::new(&ctx);
								ctx.add_timeout(timeout);
								let ctx = ctx.freeze();
								// Process the statement
								let res = stm.compute(&ctx, &opt, &self.txn(), None).await;
								// Catch statement timeout
								match ctx.is_timedout() {
									true => Err(Error::QueryTimeout {
										timer: timeout,
									}),
									false => res,
								}
							}
							// There is no timeout clause
							None => stm.compute(&ctx, &opt, &self.txn(), None).await,
						};
						// Finalise transaction
						match &res {
							Ok(_) => self.commit(loc).await,
							Err(_) => self.cancel(loc).await,
						};
						// Return the result
						res
					}
				},
			};
			// Get the statement end time
			let dur = now.elapsed();
			// Produce the response
			let res = match res {
				Ok(v) => Response {
					sql: match opt.debug {
						true => Some(format!("{}", stm)),
						false => None,
					},
					time: dur,
					result: Ok(v),
				},
				Err(e) => {
					// Produce the response
					let res = Response {
						sql: match opt.debug {
							true => Some(format!("{}", stm)),
							false => None,
						},
						time: dur,
						result: Err(e),
					};
					// Mark the error
					self.err = true;
					// Return
					res
				}
			};
			// Output the response
			match self.txn {
				Some(_) => match stm {
					Statement::Output(_) => {
						buf.clear();
						buf.push(res);
					}
					_ => buf.push(res),
				},
				None => out.push(res),
			}
		}
		// Return responses
		Ok(out)
	}
}
