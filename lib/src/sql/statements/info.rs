use crate::dbs::Level;
use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Transaction;
use crate::err::Error;
use crate::sql::comment::shouldbespace;
use crate::sql::error::IResult;
use crate::sql::ident::ident_raw;
use crate::sql::object::Object;
use crate::sql::value::Value;
use derive::Store;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Store)]
pub enum InfoStatement {
	Kv,
	Ns,
	Db,
	Sc(String),
	Tb(String),
}

impl InfoStatement {
	pub(crate) async fn compute(
		&self,
		_ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
		_doc: Option<&Value>,
	) -> Result<Value, Error> {
		// Allowed to run?
		match self {
			InfoStatement::Kv => {
				// Allowed to run?
				opt.check(Level::Kv)?;
				// Clone transaction
				let run = txn.clone();
				// Claim transaction
				let mut run = run.lock().await;
				// Create the result set
				let mut res = Object::default();
				// Process the statement
				let mut tmp = Object::default();
				for v in run.all_ns().await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("ns".to_owned(), tmp.into());
				// Ok all good
				Value::from(res).ok()
			}
			InfoStatement::Ns => {
				// Allowed to run?
				opt.check(Level::Ns)?;
				// Clone transaction
				let run = txn.clone();
				// Claim transaction
				let mut run = run.lock().await;
				// Create the result set
				let mut res = Object::default();
				// Process the databases
				let mut tmp = Object::default();
				for v in run.all_db(opt.ns()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("db".to_owned(), tmp.into());
				// Process the tokens
				let mut tmp = Object::default();
				for v in run.all_nt(opt.ns()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("nt".to_owned(), tmp.into());
				// Process the logins
				let mut tmp = Object::default();
				for v in run.all_nl(opt.ns()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("nl".to_owned(), tmp.into());
				// Ok all good
				Value::from(res).ok()
			}
			InfoStatement::Db => {
				// Allowed to run?
				opt.check(Level::Db)?;
				// Clone transaction
				let run = txn.clone();
				// Claim transaction
				let mut run = run.lock().await;
				// Create the result set
				let mut res = Object::default();
				// Process the tables
				let mut tmp = Object::default();
				for v in run.all_tb(opt.ns(), opt.db()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("tb".to_owned(), tmp.into());
				// Process the scopes
				let mut tmp = Object::default();
				for v in run.all_sc(opt.ns(), opt.db()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("sc".to_owned(), tmp.into());
				// Process the tokens
				let mut tmp = Object::default();
				for v in run.all_nt(opt.ns()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("nt".to_owned(), tmp.into());
				// Process the logins
				let mut tmp = Object::default();
				for v in run.all_nl(opt.ns()).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("nl".to_owned(), tmp.into());
				// Ok all good
				Value::from(res).ok()
			}
			InfoStatement::Sc(sc) => {
				// Allowed to run?
				opt.check(Level::Db)?;
				// Clone transaction
				let run = txn.clone();
				// Claim transaction
				let mut run = run.lock().await;
				// Create the result set
				let mut res = Object::default();
				// Process the tokens
				let mut tmp = Object::default();
				for v in run.all_st(opt.ns(), opt.db(), sc).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("st".to_owned(), tmp.into());
				// Ok all good
				Value::from(res).ok()
			}
			InfoStatement::Tb(tb) => {
				// Allowed to run?
				opt.check(Level::Db)?;
				// Clone transaction
				let run = txn.clone();
				// Claim transaction
				let mut run = run.lock().await;
				// Create the result set
				let mut res = Object::default();
				// Process the events
				let mut tmp = Object::default();
				for v in run.all_ev(opt.ns(), opt.db(), tb).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("ev".to_owned(), tmp.into());
				// Process the fields
				let mut tmp = Object::default();
				for v in run.all_fd(opt.ns(), opt.db(), tb).await? {
					tmp.insert(v.name.to_string(), v.to_string().into());
				}
				res.insert("fd".to_owned(), tmp.into());
				// Process the indexs
				let mut tmp = Object::default();
				for v in run.all_ix(opt.ns(), opt.db(), tb).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("ix".to_owned(), tmp.into());
				// Process the tables
				let mut tmp = Object::default();
				for v in run.all_ft(opt.ns(), opt.db(), tb).await? {
					tmp.insert(v.name.to_owned(), v.to_string().into());
				}
				res.insert("ft".to_owned(), tmp.into());
				// Ok all good
				Value::from(res).ok()
			}
		}
	}
}

impl fmt::Display for InfoStatement {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			InfoStatement::Kv => write!(f, "INFO FOR KV"),
			InfoStatement::Ns => write!(f, "INFO FOR NAMESPACE"),
			InfoStatement::Db => write!(f, "INFO FOR DATABASE"),
			InfoStatement::Sc(ref s) => write!(f, "INFO FOR SCOPE {}", s),
			InfoStatement::Tb(ref t) => write!(f, "INFO FOR TABLE {}", t),
		}
	}
}

pub fn info(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = tag_no_case("INFO")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, _) = tag_no_case("FOR")(i)?;
	let (i, _) = shouldbespace(i)?;
	alt((kv, ns, db, sc, tb))(i)
}

fn kv(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = tag_no_case("KV")(i)?;
	Ok((i, InfoStatement::Kv))
}

fn ns(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = alt((tag_no_case("NAMESPACE"), tag_no_case("NS")))(i)?;
	Ok((i, InfoStatement::Ns))
}

fn db(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = alt((tag_no_case("DATABASE"), tag_no_case("DB")))(i)?;
	Ok((i, InfoStatement::Db))
}

fn sc(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = alt((tag_no_case("SCOPE"), tag_no_case("SC")))(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, scope) = ident_raw(i)?;
	Ok((i, InfoStatement::Sc(scope)))
}

fn tb(i: &str) -> IResult<&str, InfoStatement> {
	let (i, _) = alt((tag_no_case("TABLE"), tag_no_case("TB")))(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, table) = ident_raw(i)?;
	Ok((i, InfoStatement::Tb(table)))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn info_query_ns() {
		let sql = "INFO FOR NAMESPACE";
		let res = info(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, InfoStatement::Ns);
		assert_eq!("INFO FOR NAMESPACE", format!("{}", out));
	}

	#[test]
	fn info_query_db() {
		let sql = "INFO FOR DATABASE";
		let res = info(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, InfoStatement::Db);
		assert_eq!("INFO FOR DATABASE", format!("{}", out));
	}

	#[test]
	fn info_query_sc() {
		let sql = "INFO FOR SCOPE test";
		let res = info(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, InfoStatement::Sc(String::from("test")));
		assert_eq!("INFO FOR SCOPE test", format!("{}", out));
	}

	#[test]
	fn info_query_tb() {
		let sql = "INFO FOR TABLE test";
		let res = info(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(out, InfoStatement::Tb(String::from("test")));
		assert_eq!("INFO FOR TABLE test", format!("{}", out));
	}
}
