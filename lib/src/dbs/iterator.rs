use crate::cnf::ID_CHARS;
use crate::ctx::Canceller;
use crate::ctx::Context;
use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Statement;
use crate::dbs::Transaction;
use crate::doc::Document;
use crate::err::Error;
use crate::sql::group::Groups;
use crate::sql::limit::Limit;
use crate::sql::order::Orders;
use crate::sql::split::Splits;
use crate::sql::start::Start;
use crate::sql::statements::create::CreateStatement;
use crate::sql::statements::delete::DeleteStatement;
use crate::sql::statements::insert::InsertStatement;
use crate::sql::statements::relate::RelateStatement;
use crate::sql::statements::select::SelectStatement;
use crate::sql::statements::update::UpdateStatement;
use crate::sql::table::Table;
use crate::sql::thing::Thing;
use crate::sql::value::Value;
use crate::sql::version::Version;
use nanoid::nanoid;
use std::mem;

#[derive(Default)]
pub struct Iterator<'a> {
	// Iterator status
	run: Canceller,
	// Iterator runtime error
	error: Option<Error>,
	// Iterator input values
	readies: Vec<Value>,
	// Iterator output results
	results: Vec<Value>,
	// Iterate options
	pub parallel: bool,
	// Underlying statement
	pub stmt: Statement<'a>,
	// Iterator options
	pub split: Option<&'a Splits>,
	pub group: Option<&'a Groups>,
	pub order: Option<&'a Orders>,
	pub limit: Option<&'a Limit>,
	pub start: Option<&'a Start>,
	pub version: Option<&'a Version>,
}

impl<'a> From<&'a SelectStatement> for Iterator<'a> {
	fn from(v: &'a SelectStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			split: v.split.as_ref(),
			group: v.group.as_ref(),
			order: v.order.as_ref(),
			limit: v.limit.as_ref(),
			start: v.start.as_ref(),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> From<&'a CreateStatement> for Iterator<'a> {
	fn from(v: &'a CreateStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> From<&'a UpdateStatement> for Iterator<'a> {
	fn from(v: &'a UpdateStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> From<&'a RelateStatement> for Iterator<'a> {
	fn from(v: &'a RelateStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> From<&'a DeleteStatement> for Iterator<'a> {
	fn from(v: &'a DeleteStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> From<&'a InsertStatement> for Iterator<'a> {
	fn from(v: &'a InsertStatement) -> Self {
		Iterator {
			stmt: Statement::from(v),
			parallel: v.parallel,
			..Iterator::default()
		}
	}
}

impl<'a> Iterator<'a> {
	// Prepares a value for processing
	pub fn prepare(&mut self, val: Value) {
		self.readies.push(val)
	}

	// Create a new record for processing
	pub fn produce(&mut self, val: Table) {
		self.prepare(Value::Thing(Thing {
			tb: val.name.to_string(),
			id: nanoid!(20, &ID_CHARS),
		}))
	}

	// Process the records and output
	pub async fn output(
		&mut self,
		ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Value, Error> {
		// Log the statement
		trace!("Iterating: {}", self.stmt);
		// Enable context override
		let mut ctx = Context::new(&ctx);
		self.run = ctx.add_cancel();
		let ctx = ctx.freeze();
		// Process prepared values
		self.iterate(&ctx, opt, txn).await?;
		// Return any document errors
		if let Some(e) = self.error.take() {
			return Err(e);
		}
		// Process any SPLIT clause
		self.output_split(&ctx, opt, txn);
		// Process any GROUP clause
		self.output_group(&ctx, opt, txn);
		// Process any ORDER clause
		self.output_order(&ctx, opt, txn);
		// Process any START clause
		self.output_start(&ctx, opt, txn);
		// Process any LIMIT clause
		self.output_limit(&ctx, opt, txn);
		// Output the results
		Ok(mem::take(&mut self.results).into())
	}

	#[inline]
	fn output_split(&mut self, ctx: &Runtime, opt: &Options, txn: &Transaction) {
		if self.split.is_some() {
			// Ignore
		}
	}

	#[inline]
	fn output_group(&mut self, ctx: &Runtime, opt: &Options, txn: &Transaction) {
		if self.group.is_some() {
			// Ignore
		}
	}

	#[inline]
	fn output_order(&mut self, ctx: &Runtime, opt: &Options, txn: &Transaction) {
		if self.order.is_some() {
			// Ignore
		}
	}

	#[inline]
	fn output_start(&mut self, ctx: &Runtime, opt: &Options, txn: &Transaction) {
		if let Some(v) = self.start {
			self.results = mem::take(&mut self.results).into_iter().skip(v.0).collect();
		}
	}

	#[inline]
	fn output_limit(&mut self, ctx: &Runtime, opt: &Options, txn: &Transaction) {
		if let Some(v) = self.limit {
			self.results = mem::take(&mut self.results).into_iter().take(v.0).collect();
		}
	}

	#[cfg(not(feature = "parallel"))]
	async fn iterate(
		&mut self,
		ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
	) -> Result<(), Error> {
		// Process all prepared values
		for v in mem::take(&mut self.readies) {
			v.iterate(ctx, opt, txn, self).await?;
		}
		// Everything processed ok
		Ok(())
	}

	#[cfg(feature = "parallel")]
	async fn iterate(
		&mut self,
		ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
	) -> Result<(), Error> {
		match self.parallel {
			// Run statements sequentially
			false => {
				// Process all prepared values
				for v in mem::take(&mut self.readies) {
					v.iterate(ctx, opt, txn, self).await?;
				}
				// Everything processed ok
				Ok(())
			}
			// Run statements in parallel
			true => {
				let mut rcv = {
					// Use multi producer channel
					use tokio::sync::mpsc;
					// Create an unbounded channel
					let (chn, rcv) = mpsc::unbounded_channel();
					// Process all prepared values
					for v in mem::take(&mut self.readies) {
						tokio::spawn(v.channel(ctx.clone(), opt.clone(), txn.clone(), chn.clone()));
					}
					//
					rcv
				};
				// Process all processed values
				while let Some((k, v)) = rcv.recv().await {
					self.process(&ctx, opt, txn, k, v).await;
				}
				// Everything processed ok
				Ok(())
			}
		}
	}

	// Process a new record Thing and Value
	pub async fn process(
		&mut self,
		ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
		thg: Option<Thing>,
		val: Value,
	) {
		// Check current context
		if ctx.is_done() {
			return;
		}
		// Setup a new document
		let mut doc = Document::new(thg, &val);
		// Process the document
		let res = match self.stmt {
			Statement::Select(_) => doc.select(ctx, opt, txn, &self.stmt).await,
			Statement::Create(_) => doc.create(ctx, opt, txn, &self.stmt).await,
			Statement::Update(_) => doc.update(ctx, opt, txn, &self.stmt).await,
			Statement::Relate(_) => doc.relate(ctx, opt, txn, &self.stmt).await,
			Statement::Delete(_) => doc.delete(ctx, opt, txn, &self.stmt).await,
			Statement::Insert(_) => doc.insert(ctx, opt, txn, &self.stmt).await,
			_ => unreachable!(),
		};
		// Process the result
		self.result(res);
	}

	// Accept a processed record result
	fn result(&mut self, res: Result<Value, Error>) {
		// Process the result
		match res {
			Err(Error::IgnoreError) => {
				return;
			}
			Err(e) => {
				self.error = Some(e);
				self.run.cancel();
				return;
			}
			Ok(v) => self.results.push(v),
		}
		// Check if we can exit
		if self.group.is_none() {
			if self.order.is_none() {
				if let Some(l) = self.limit {
					if let Some(s) = self.start {
						if self.results.len() == l.0 + s.0 {
							self.run.cancel()
						}
					} else {
						if self.results.len() == l.0 {
							self.run.cancel()
						}
					}
				}
			}
		}
	}
}
