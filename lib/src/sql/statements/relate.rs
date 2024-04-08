use crate::ctx::Context;
use crate::dbs::Iterable;
use crate::dbs::Iterator;
use crate::dbs::Level;
use crate::dbs::Options;
use crate::dbs::Statement;
use crate::dbs::Transaction;
use crate::err::Error;
use crate::sql::comment::mightbespace;
use crate::sql::comment::shouldbespace;
use crate::sql::data::{data, Data};
use crate::sql::error::IResult;
use crate::sql::output::{output, Output};
use crate::sql::table::{table, Table};
use crate::sql::timeout::{timeout, Timeout};
use crate::sql::value::{whats, Value, Values};
use derive::Store;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::combinator::opt;
use nom::sequence::preceded;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Store)]
pub struct RelateStatement {
	pub kind: Table,
	pub from: Values,
	pub with: Values,
	pub uniq: bool,
	pub data: Option<Data>,
	pub output: Option<Output>,
	pub timeout: Option<Timeout>,
	pub parallel: bool,
}

impl RelateStatement {
	pub(crate) fn writeable(&self) -> bool {
		true
	}

	pub(crate) async fn compute(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
		doc: Option<&Value>,
	) -> Result<Value, Error> {
		// Allowed to run?
		opt.check(Level::No)?;
		// Create a new iterator
		let mut i = Iterator::new();
		// Ensure futures are stored
		let opt = &opt.futures(false);
		// Loop over the from targets
		let from = {
			let mut out = Vec::new();
			for w in self.from.0.iter() {
				let v = w.compute(ctx, opt, txn, doc).await?;
				match v {
					Value::Thing(v) => out.push(v),
					Value::Array(v) => {
						for v in v {
							match v {
								Value::Thing(v) => out.push(v),
								Value::Object(v) => match v.rid() {
									Some(v) => out.push(v),
									_ => {
										return Err(Error::RelateStatement {
											value: v.to_string(),
										})
									}
								},
								v => {
									return Err(Error::RelateStatement {
										value: v.to_string(),
									})
								}
							}
						}
					}
					Value::Object(v) => match v.rid() {
						Some(v) => out.push(v),
						None => {
							return Err(Error::RelateStatement {
								value: v.to_string(),
							})
						}
					},
					v => {
						return Err(Error::RelateStatement {
							value: v.to_string(),
						})
					}
				};
			}
			out
		};
		// Loop over the with targets
		let with = {
			let mut out = Vec::new();
			for w in self.with.0.iter() {
				let v = w.compute(ctx, opt, txn, doc).await?;
				match v {
					Value::Thing(v) => out.push(v),
					Value::Array(v) => {
						for v in v {
							match v {
								Value::Thing(v) => out.push(v),
								Value::Object(v) => match v.rid() {
									Some(v) => out.push(v),
									None => {
										return Err(Error::RelateStatement {
											value: v.to_string(),
										})
									}
								},
								v => {
									return Err(Error::RelateStatement {
										value: v.to_string(),
									})
								}
							}
						}
					}
					Value::Object(v) => match v.rid() {
						Some(v) => out.push(v),
						None => {
							return Err(Error::RelateStatement {
								value: v.to_string(),
							})
						}
					},
					v => {
						return Err(Error::RelateStatement {
							value: v.to_string(),
						})
					}
				};
			}
			out
		};
		//
		for f in from.iter() {
			for w in with.iter() {
				let f = f.clone();
				let w = w.clone();
				let t = self.kind.generate();
				i.ingest(Iterable::Relatable(f, t, w));
			}
		}
		// Assign the statement
		let stm = Statement::from(self);
		// Output the results
		i.output(ctx, opt, txn, &stm).await
	}
}

impl fmt::Display for RelateStatement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "RELATE {} -> {} -> {}", self.from, self.kind, self.with)?;
		if self.uniq {
			write!(f, " UNIQUE")?
		}
		if let Some(ref v) = self.data {
			write!(f, " {}", v)?
		}
		if let Some(ref v) = self.output {
			write!(f, " {}", v)?
		}
		if let Some(ref v) = self.timeout {
			write!(f, " {}", v)?
		}
		if self.parallel {
			write!(f, " PARALLEL")?
		}
		Ok(())
	}
}

pub fn relate(i: &str) -> IResult<&str, RelateStatement> {
	let (i, _) = tag_no_case("RELATE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, path) = alt((relate_o, relate_i))(i)?;
	let (i, uniq) = opt(preceded(shouldbespace, tag_no_case("UNIQUE")))(i)?;
	let (i, data) = opt(preceded(shouldbespace, data))(i)?;
	let (i, output) = opt(preceded(shouldbespace, output))(i)?;
	let (i, timeout) = opt(preceded(shouldbespace, timeout))(i)?;
	let (i, parallel) = opt(preceded(shouldbespace, tag_no_case("PARALLEL")))(i)?;
	Ok((
		i,
		RelateStatement {
			kind: path.0,
			from: path.1,
			with: path.2,
			uniq: uniq.is_some(),
			data,
			output,
			timeout,
			parallel: parallel.is_some(),
		},
	))
}

fn relate_o(i: &str) -> IResult<&str, (Table, Values, Values)> {
	let (i, from) = whats(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, _) = char('-')(i)?;
	let (i, _) = char('>')(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, kind) = table(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, _) = char('-')(i)?;
	let (i, _) = char('>')(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, with) = whats(i)?;
	Ok((i, (kind, from, with)))
}

fn relate_i(i: &str) -> IResult<&str, (Table, Values, Values)> {
	let (i, with) = whats(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, _) = char('<')(i)?;
	let (i, _) = char('-')(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, kind) = table(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, _) = char('<')(i)?;
	let (i, _) = char('-')(i)?;
	let (i, _) = mightbespace(i)?;
	let (i, from) = whats(i)?;
	Ok((i, (kind, from, with)))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn relate_statement_in() {
		let sql = "RELATE person->like->animal";
		let res = relate(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("RELATE person -> like -> animal", format!("{}", out))
	}

	#[test]
	fn relate_statement_out() {
		let sql = "RELATE animal<-like<-person";
		let res = relate(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("RELATE person -> like -> animal", format!("{}", out))
	}

	#[test]
	fn relate_statement_thing() {
		let sql = "RELATE person:tobie->like->person:jaime";
		let res = relate(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("RELATE person:tobie -> like -> person:jaime", format!("{}", out))
	}
}
