use crate::sql::common::is_hex;
use crate::sql::error::IResult;
use crate::sql::serde::is_internal_serialization;
use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::character::complete::char;
use nom::combinator::recognize;
use nom::sequence::delimited;
use nom::sequence::tuple;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;
use std::str;

const SINGLE: char = '\'';
const DOUBLE: char = '"';

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize)]
pub struct Uuid(pub uuid::Uuid);

impl From<&str> for Uuid {
	fn from(s: &str) -> Self {
		match uuid::Uuid::try_parse(s) {
			Ok(v) => Uuid(v),
			_ => Uuid::default(),
		}
	}
}

impl From<String> for Uuid {
	fn from(s: String) -> Self {
		match uuid::Uuid::try_parse(&s) {
			Ok(v) => Uuid(v),
			_ => Uuid::default(),
		}
	}
}

impl Deref for Uuid {
	type Target = uuid::Uuid;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Uuid {
	pub fn new() -> Self {
		Uuid(uuid::Uuid::new_v4())
	}
	pub fn to_raw(&self) -> String {
		self.0.to_string()
	}
}

impl fmt::Display for Uuid {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "\"{}\"", self.0)
	}
}

impl Serialize for Uuid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if is_internal_serialization() {
			serializer.serialize_newtype_struct("Uuid", &self.0)
		} else {
			serializer.serialize_some(&self.0)
		}
	}
}

pub fn uuid(i: &str) -> IResult<&str, Uuid> {
	alt((
		delimited(char(DOUBLE), uuid_raw, char(DOUBLE)),
		delimited(char(SINGLE), uuid_raw, char(SINGLE)),
	))(i)
}

fn uuid_raw(i: &str) -> IResult<&str, Uuid> {
	let (i, v) = recognize(tuple((
		take_while_m_n(8, 8, is_hex),
		char('-'),
		take_while_m_n(4, 4, is_hex),
		char('-'),
		alt((char('1'), char('2'), char('3'), char('4'))),
		take_while_m_n(3, 3, is_hex),
		char('-'),
		take_while_m_n(4, 4, is_hex),
		char('-'),
		take_while_m_n(12, 12, is_hex),
	)))(i)?;
	Ok((i, Uuid::from(v)))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn uuid_v1() {
		let sql = "e72bee20-f49b-11ec-b939-0242ac120002";
		let res = uuid_raw(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("\"e72bee20-f49b-11ec-b939-0242ac120002\"", format!("{}", out));
		assert_eq!(out, Uuid::from("e72bee20-f49b-11ec-b939-0242ac120002"));
	}

	#[test]
	fn uuid_v4() {
		let sql = "b19bc00b-aa98-486c-ae37-c8e1c54295b1";
		let res = uuid_raw(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("\"b19bc00b-aa98-486c-ae37-c8e1c54295b1\"", format!("{}", out));
		assert_eq!(out, Uuid::from("b19bc00b-aa98-486c-ae37-c8e1c54295b1"));
	}
}
