use crate::sql::array::{array, Array};
use crate::sql::comment::mightbespace;
use crate::sql::comment::shouldbespace;
use crate::sql::common::commas;
use crate::sql::idiom::{idiom, Idiom};
use crate::sql::object::{object, Object};
use crate::sql::operator::{assigner, Operator};
use crate::sql::value::{value, Value};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::multi::separated_list1;
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Data {
	EmptyExpression,
	SetExpression(Vec<(Idiom, Operator, Value)>),
	DiffExpression(Array),
	MergeExpression(Object),
	ReplaceExpression(Value),
	ContentExpression(Value),
	SingleExpression(Value),
	ValuesExpression(Vec<Vec<(Idiom, Value)>>),
	UpdateExpression(Vec<(Idiom, Operator, Value)>),
}

impl Default for Data {
	fn default() -> Data {
		Data::EmptyExpression
	}
}

impl fmt::Display for Data {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Data::EmptyExpression => write!(f, ""),
			Data::SetExpression(v) => write!(
				f,
				"SET {}",
				v.iter()
					.map(|(l, o, r)| format!("{} {} {}", l, o, r))
					.collect::<Vec<_>>()
					.join(", ")
			),
			Data::DiffExpression(v) => write!(f, "DIFF {}", v),
			Data::MergeExpression(v) => write!(f, "MERGE {}", v),
			Data::ReplaceExpression(v) => write!(f, "REPLACE {}", v),
			Data::ContentExpression(v) => write!(f, "CONTENT {}", v),
			Data::SingleExpression(v) => write!(f, "{}", v),
			Data::ValuesExpression(v) => write!(
				f,
				"({}) VALUES {}",
				v.first()
					.unwrap()
					.iter()
					.map(|v| format!("{}", v.0))
					.collect::<Vec<_>>()
					.join(", "),
				v.iter()
					.map(|v| format!(
						"({})",
						v.iter().map(|v| format!("{}", v.1)).collect::<Vec<_>>().join(", ")
					))
					.collect::<Vec<_>>()
					.join(", ")
			),
			Data::UpdateExpression(v) => write!(
				f,
				"ON DUPLICATE KEY UPDATE {}",
				v.iter()
					.map(|(l, o, r)| format!("{} {} {}", l, o, r))
					.collect::<Vec<_>>()
					.join(", ")
			),
		}
	}
}

pub fn data(i: &str) -> IResult<&str, Data> {
	alt((set, diff, merge, replace, content))(i)
}

fn set(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("SET")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = separated_list1(commas, |i| {
		let (i, l) = idiom(i)?;
		let (i, _) = mightbespace(i)?;
		let (i, o) = assigner(i)?;
		let (i, _) = mightbespace(i)?;
		let (i, r) = value(i)?;
		Ok((i, (l, o, r)))
	})(i)?;
	Ok((i, Data::SetExpression(v)))
}

fn diff(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("DIFF")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = array(i)?;
	Ok((i, Data::DiffExpression(v)))
}

fn merge(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("MERGE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = object(i)?;
	Ok((i, Data::MergeExpression(v)))
}

fn replace(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("REPLACE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = value(i)?;
	Ok((i, Data::ReplaceExpression(v)))
}

fn content(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("CONTENT")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = value(i)?;
	Ok((i, Data::ContentExpression(v)))
}

pub fn single(i: &str) -> IResult<&str, Data> {
	let (i, v) = value(i)?;
	Ok((i, Data::SingleExpression(v)))
}

pub fn values(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("(")(i)?;
	let (i, fields) = separated_list1(commas, idiom)(i)?;
	let (i, _) = tag_no_case(")")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, _) = tag_no_case("VALUES")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, values) = separated_list1(commas, |i| {
		let (i, _) = tag_no_case("(")(i)?;
		let (i, v) = separated_list1(commas, value)(i)?;
		let (i, _) = tag_no_case(")")(i)?;
		Ok((i, v))
	})(i)?;
	Ok((
		i,
		Data::ValuesExpression(
			values
				.into_iter()
				.map(|row| fields.iter().cloned().zip(row.into_iter()).collect())
				.collect(),
		),
	))
}

pub fn update(i: &str) -> IResult<&str, Data> {
	let (i, _) = tag_no_case("ON DUPLICATE KEY UPDATE")(i)?;
	let (i, _) = shouldbespace(i)?;
	let (i, v) = separated_list1(commas, |i| {
		let (i, l) = idiom(i)?;
		let (i, _) = mightbespace(i)?;
		let (i, o) = assigner(i)?;
		let (i, _) = mightbespace(i)?;
		let (i, r) = value(i)?;
		Ok((i, (l, o, r)))
	})(i)?;
	Ok((i, Data::UpdateExpression(v)))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn set_statement() {
		let sql = "SET field = true";
		let res = data(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("SET field = true", format!("{}", out));
	}

	#[test]
	fn set_statement_multiple() {
		let sql = "SET field = true, other.field = false";
		let res = data(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("SET field = true, other.field = false", format!("{}", out));
	}

	#[test]
	fn diff_statement() {
		let sql = "DIFF [{ field: true }]";
		let res = data(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("DIFF [{ field: true }]", format!("{}", out));
	}

	#[test]
	fn merge_statement() {
		let sql = "MERGE { field: true }";
		let res = data(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("MERGE { field: true }", format!("{}", out));
	}

	#[test]
	fn content_statement() {
		let sql = "CONTENT { field: true }";
		let res = data(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("CONTENT { field: true }", format!("{}", out));
	}

	#[test]
	fn values_statement() {
		let sql = "(one, two, three) VALUES ($param, true, [1, 2, 3]), ($param, false, [4, 5, 6])";
		let res = values(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!(
			"(one, two, three) VALUES ($param, true, [1, 2, 3]), ($param, false, [4, 5, 6])",
			format!("{}", out)
		);
	}

	#[test]
	fn update_statement() {
		let sql = "ON DUPLICATE KEY UPDATE field = true, other.field = false";
		let res = update(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("ON DUPLICATE KEY UPDATE field = true, other.field = false", format!("{}", out));
	}
}
