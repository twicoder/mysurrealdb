#![allow(clippy::derive_ord_xor_partial_ord)]

use crate::ctx::Context;
use crate::dbs::Options;
use crate::dbs::Transaction;
use crate::err::Error;
use crate::sql::array::{array, Array};
use crate::sql::common::commas;
use crate::sql::datetime::{datetime, Datetime};
use crate::sql::duration::{duration, Duration};
use crate::sql::error::IResult;
use crate::sql::expression::{expression, Expression};
use crate::sql::function::{function, Function};
use crate::sql::geometry::{geometry, Geometry};
use crate::sql::id::Id;
use crate::sql::idiom::{idiom, Idiom};
use crate::sql::kind::Kind;
use crate::sql::model::{model, Model};
use crate::sql::number::{number, Number};
use crate::sql::object::{object, Object};
use crate::sql::operation::Operation;
use crate::sql::param::{param, Param};
use crate::sql::part::Part;
use crate::sql::regex::{regex, Regex};
use crate::sql::strand::{strand, Strand};
use crate::sql::subquery::{subquery, Subquery};
use crate::sql::table::{table, Table};
use crate::sql::thing::{thing, Thing};
use async_recursion::async_recursion;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use derive::Store;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use geo::Point;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom::multi::separated_list1;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;
use std::ops;
use std::ops::Deref;
use std::str::FromStr;

static MATCHER: Lazy<SkimMatcherV2> = Lazy::new(|| SkimMatcherV2::default().ignore_case());

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Values(pub Vec<Value>);

impl Deref for Values {
	type Target = Vec<Value>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl IntoIterator for Values {
	type Item = Value;
	type IntoIter = std::vec::IntoIter<Self::Item>;
	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl fmt::Display for Values {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0.iter().map(|ref v| format!("{}", v)).collect::<Vec<_>>().join(", "))
	}
}

pub fn values(i: &str) -> IResult<&str, Values> {
	let (i, v) = separated_list1(commas, value)(i)?;
	Ok((i, Values(v)))
}

pub fn selects(i: &str) -> IResult<&str, Values> {
	let (i, v) = separated_list1(commas, select)(i)?;
	Ok((i, Values(v)))
}

pub fn whats(i: &str) -> IResult<&str, Values> {
	let (i, v) = separated_list1(commas, what)(i)?;
	Ok((i, Values(v)))
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Store)]
pub enum Value {
	None,
	Void,
	Null,
	False,
	True,
	Number(Number),
	Strand(Strand),
	Duration(Duration),
	Datetime(Datetime),
	Array(Array),
	Object(Object),
	Geometry(Geometry),
	// ---
	Param(Param),
	Idiom(Idiom),
	Table(Table),
	Thing(Thing),
	Model(Model),
	Regex(Regex),
	Function(Box<Function>),
	Subquery(Box<Subquery>),
	Expression(Box<Expression>),
}

impl Eq for Value {}

impl Ord for Value {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).unwrap_or(Ordering::Equal)
	}
}

impl Default for Value {
	fn default() -> Value {
		Value::None
	}
}

impl From<bool> for Value {
	fn from(v: bool) -> Self {
		match v {
			true => Value::True,
			false => Value::False,
		}
	}
}

impl From<Param> for Value {
	fn from(v: Param) -> Self {
		Value::Param(v)
	}
}

impl From<Idiom> for Value {
	fn from(v: Idiom) -> Self {
		Value::Idiom(v)
	}
}

impl From<Table> for Value {
	fn from(v: Table) -> Self {
		Value::Table(v)
	}
}

impl From<Thing> for Value {
	fn from(v: Thing) -> Self {
		Value::Thing(v)
	}
}

impl From<Regex> for Value {
	fn from(v: Regex) -> Self {
		Value::Regex(v)
	}
}

impl From<Array> for Value {
	fn from(v: Array) -> Self {
		Value::Array(v)
	}
}

impl From<Object> for Value {
	fn from(v: Object) -> Self {
		Value::Object(v)
	}
}

impl From<Number> for Value {
	fn from(v: Number) -> Self {
		Value::Number(v)
	}
}

impl From<Strand> for Value {
	fn from(v: Strand) -> Self {
		Value::Strand(v)
	}
}

impl From<Geometry> for Value {
	fn from(v: Geometry) -> Self {
		Value::Geometry(v)
	}
}

impl From<Datetime> for Value {
	fn from(v: Datetime) -> Self {
		Value::Datetime(v)
	}
}

impl From<Duration> for Value {
	fn from(v: Duration) -> Self {
		Value::Duration(v)
	}
}

impl From<Function> for Value {
	fn from(v: Function) -> Self {
		Value::Function(Box::new(v))
	}
}

impl From<Expression> for Value {
	fn from(v: Expression) -> Self {
		Value::Expression(Box::new(v))
	}
}

impl From<Box<Expression>> for Value {
	fn from(v: Box<Expression>) -> Self {
		Value::Expression(v)
	}
}

impl From<i8> for Value {
	fn from(v: i8) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<i16> for Value {
	fn from(v: i16) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<i32> for Value {
	fn from(v: i32) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<i64> for Value {
	fn from(v: i64) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<isize> for Value {
	fn from(v: isize) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<u8> for Value {
	fn from(v: u8) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<u16> for Value {
	fn from(v: u16) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<u32> for Value {
	fn from(v: u32) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<u64> for Value {
	fn from(v: u64) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<usize> for Value {
	fn from(v: usize) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<f32> for Value {
	fn from(v: f32) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<f64> for Value {
	fn from(v: f64) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<BigDecimal> for Value {
	fn from(v: BigDecimal) -> Self {
		Value::Number(Number::from(v))
	}
}

impl From<String> for Value {
	fn from(v: String) -> Self {
		Value::Strand(Strand::from(v))
	}
}

impl<'a> From<&'a str> for Value {
	fn from(v: &str) -> Self {
		Value::Strand(Strand::from(v))
	}
}

impl From<DateTime<Utc>> for Value {
	fn from(v: DateTime<Utc>) -> Self {
		Value::Datetime(Datetime::from(v))
	}
}

impl From<(f64, f64)> for Value {
	fn from(v: (f64, f64)) -> Self {
		Value::Geometry(Geometry::from(v))
	}
}

impl From<[f64; 2]> for Value {
	fn from(v: [f64; 2]) -> Self {
		Value::Geometry(Geometry::from(v))
	}
}

impl From<Point<f64>> for Value {
	fn from(v: Point<f64>) -> Self {
		Value::Geometry(Geometry::from(v))
	}
}

impl From<Operation> for Value {
	fn from(v: Operation) -> Self {
		Value::Object(Object::from(v))
	}
}

impl<'a> From<Vec<&str>> for Value {
	fn from(v: Vec<&str>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<Vec<i32>> for Value {
	fn from(v: Vec<i32>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<Vec<String>> for Value {
	fn from(v: Vec<String>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<Vec<Value>> for Value {
	fn from(v: Vec<Value>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<Vec<Operation>> for Value {
	fn from(v: Vec<Operation>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<Vec<Vec<Value>>> for Value {
	fn from(v: Vec<Vec<Value>>) -> Self {
		Value::Array(Array::from(v))
	}
}

impl From<HashMap<String, Value>> for Value {
	fn from(v: HashMap<String, Value>) -> Self {
		Value::Object(Object::from(v))
	}
}

impl From<BTreeMap<String, Value>> for Value {
	fn from(v: BTreeMap<String, Value>) -> Self {
		Value::Object(Object::from(v))
	}
}

impl From<Option<Value>> for Value {
	fn from(v: Option<Value>) -> Self {
		match v {
			Some(v) => v,
			None => Value::None,
		}
	}
}

impl From<Option<String>> for Value {
	fn from(v: Option<String>) -> Self {
		match v {
			Some(v) => Value::from(v),
			None => Value::None,
		}
	}
}

impl From<Id> for Value {
	fn from(v: Id) -> Self {
		match v {
			Id::Number(v) => v.into(),
			Id::String(v) => Strand::from(v).into(),
		}
	}
}

impl Value {
	// -----------------------------------
	// Initial record value
	// -----------------------------------

	pub fn base() -> Self {
		Value::Object(Object::default())
	}

	// -----------------------------------
	// Builtin types
	// -----------------------------------

	pub fn ok(self) -> Result<Value, Error> {
		Ok(self)
	}

	pub fn output(self) -> Option<Value> {
		match self {
			Value::None => None,
			Value::Void => None,
			_ => Some(self),
		}
	}

	// -----------------------------------
	// Simple value detection
	// -----------------------------------

	pub fn is_none(&self) -> bool {
		matches!(self, Value::None | Value::Void | Value::Null)
	}

	pub fn is_void(&self) -> bool {
		matches!(self, Value::None | Value::Void)
	}

	pub fn is_null(&self) -> bool {
		matches!(self, Value::None | Value::Null)
	}

	pub fn is_some(&self) -> bool {
		!self.is_none()
	}

	pub fn is_true(&self) -> bool {
		match self {
			Value::True => true,
			Value::Strand(v) => v.to_ascii_lowercase() == "true",
			_ => false,
		}
	}

	pub fn is_false(&self) -> bool {
		match self {
			Value::False => true,
			Value::Strand(v) => v.to_ascii_lowercase() == "false",
			_ => false,
		}
	}

	pub fn is_truthy(&self) -> bool {
		match self {
			Value::True => true,
			Value::False => false,
			Value::Thing(_) => true,
			Value::Geometry(_) => true,
			Value::Array(v) => !v.is_empty(),
			Value::Object(v) => !v.is_empty(),
			Value::Strand(v) => !v.is_empty() && v.to_ascii_lowercase() != "false",
			Value::Number(v) => v.is_truthy(),
			Value::Duration(v) => v.as_nanos() > 0,
			Value::Datetime(v) => v.timestamp() > 0,
			_ => false,
		}
	}

	pub fn is_type_record(&self, types: &[Table]) -> bool {
		match self {
			Value::Thing(v) => types.iter().any(|tb| tb.0 == v.tb),
			_ => false,
		}
	}

	pub fn is_type_geometry(&self, types: &[String]) -> bool {
		match self {
			Value::Geometry(Geometry::Point(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "point")
			}
			Value::Geometry(Geometry::Line(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "line")
			}
			Value::Geometry(Geometry::Polygon(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "polygon")
			}
			Value::Geometry(Geometry::MultiPoint(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "multipoint")
			}
			Value::Geometry(Geometry::MultiLine(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "multiline")
			}
			Value::Geometry(Geometry::MultiPolygon(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "multipolygon")
			}
			Value::Geometry(Geometry::Collection(_)) => {
				types.iter().any(|t| &t[..] == "feature" || &t[..] == "collection")
			}
			_ => false,
		}
	}

	// -----------------------------------
	// Simple conversion of value
	// -----------------------------------

	pub fn as_int(self) -> i64 {
		match self {
			Value::True => 1,
			Value::Strand(v) => v.parse::<i64>().unwrap_or(0),
			Value::Number(v) => v.as_int(),
			Value::Duration(v) => v.as_secs() as i64,
			Value::Datetime(v) => v.timestamp(),
			_ => 0,
		}
	}

	pub fn as_float(self) -> f64 {
		match self {
			Value::True => 1.0,
			Value::Strand(v) => v.parse::<f64>().unwrap_or(0.0),
			Value::Number(v) => v.as_float(),
			Value::Duration(v) => v.as_secs() as f64,
			Value::Datetime(v) => v.timestamp() as f64,
			_ => 0.0,
		}
	}

	pub fn as_decimal(self) -> BigDecimal {
		match self {
			Value::True => BigDecimal::from(1),
			Value::Number(v) => v.as_decimal(),
			Value::Strand(v) => BigDecimal::from_str(v.as_str()).unwrap_or_default(),
			Value::Duration(v) => v.as_secs().into(),
			Value::Datetime(v) => v.timestamp().into(),
			_ => BigDecimal::default(),
		}
	}

	pub fn as_number(self) -> Number {
		match self {
			Value::True => Number::from(1),
			Value::Number(v) => v,
			Value::Strand(v) => Number::from(v.as_str()),
			Value::Duration(v) => v.as_secs().into(),
			Value::Datetime(v) => v.timestamp().into(),
			_ => Number::default(),
		}
	}

	pub fn as_strand(self) -> Strand {
		match self {
			Value::Strand(v) => v,
			_ => Strand::from(self.to_string()),
		}
	}

	pub fn as_datetime(self) -> Datetime {
		match self {
			Value::Strand(v) => Datetime::from(v.as_str()),
			Value::Datetime(v) => v,
			_ => Datetime::default(),
		}
	}

	pub fn as_duration(self) -> Duration {
		match self {
			Value::Strand(v) => Duration::from(v.as_str()),
			Value::Duration(v) => v,
			_ => Duration::default(),
		}
	}

	pub fn as_string(self) -> String {
		match self {
			Value::Strand(v) => v.as_string(),
			_ => self.to_string(),
		}
	}

	// -----------------------------------
	// Expensive conversion of value
	// -----------------------------------

	pub fn to_number(&self) -> Number {
		match self {
			Value::True => Number::from(1),
			Value::Number(v) => v.clone(),
			Value::Strand(v) => Number::from(v.as_str()),
			Value::Duration(v) => v.as_secs().into(),
			Value::Datetime(v) => v.timestamp().into(),
			_ => Number::default(),
		}
	}

	pub fn to_strand(&self) -> Strand {
		match self {
			Value::Strand(v) => v.clone(),
			_ => Strand::from(self.to_string()),
		}
	}

	pub fn to_datetime(&self) -> Datetime {
		match self {
			Value::Strand(v) => Datetime::from(v.as_str()),
			Value::Datetime(v) => v.clone(),
			_ => Datetime::default(),
		}
	}

	pub fn to_duration(&self) -> Duration {
		match self {
			Value::Strand(v) => Duration::from(v.as_str()),
			Value::Duration(v) => v.clone(),
			_ => Duration::default(),
		}
	}

	pub fn to_idiom(&self) -> Idiom {
		self.to_strand()
			.as_str()
			.trim_start_matches('/')
			.split(&['.', '/'][..])
			.map(Part::from)
			.collect::<Vec<Part>>()
			.into()
	}

	pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
		msgpack::to_vec(&self).map_err(|e| e.into())
	}

	// -----------------------------------
	// Simple conversion of value
	// -----------------------------------

	pub fn make_bool(self) -> Value {
		match self {
			Value::True | Value::False => self,
			_ => self.is_truthy().into(),
		}
	}

	pub fn make_int(self) -> Value {
		match self {
			Value::Number(Number::Int(_)) => self,
			_ => self.as_int().into(),
		}
	}

	pub fn make_float(self) -> Value {
		match self {
			Value::Number(Number::Float(_)) => self,
			_ => self.as_float().into(),
		}
	}

	pub fn make_decimal(self) -> Value {
		match self {
			Value::Number(Number::Decimal(_)) => self,
			_ => self.as_decimal().into(),
		}
	}

	pub fn make_number(self) -> Value {
		match self {
			Value::Number(_) => self,
			_ => self.as_number().into(),
		}
	}

	pub fn make_strand(self) -> Value {
		match self {
			Value::Strand(_) => self,
			_ => self.as_strand().into(),
		}
	}

	pub fn make_datetime(self) -> Value {
		match self {
			Value::Datetime(_) => self,
			_ => self.as_datetime().into(),
		}
	}

	pub fn make_duration(self) -> Value {
		match self {
			Value::Duration(_) => self,
			_ => self.as_duration().into(),
		}
	}

	pub fn convert_to(self, kind: &Kind) -> Value {
		match kind {
			Kind::Any => self,
			Kind::Bool => self.make_bool(),
			Kind::Int => self.make_int(),
			Kind::Float => self.make_float(),
			Kind::Decimal => self.make_decimal(),
			Kind::Number => self.make_number(),
			Kind::String => self.make_strand(),
			Kind::Datetime => self.make_datetime(),
			Kind::Duration => self.make_duration(),
			Kind::Array => match self {
				Value::Array(_) => self,
				_ => Value::None,
			},
			Kind::Object => match self {
				Value::Object(_) => self,
				_ => Value::None,
			},
			Kind::Record(t) => match self.is_type_record(t) {
				true => self,
				_ => Value::None,
			},
			Kind::Geometry(t) => match self.is_type_geometry(t) {
				true => self,
				_ => Value::None,
			},
		}
	}

	// -----------------------------------
	// Value operations
	// -----------------------------------

	pub fn equal(&self, other: &Value) -> bool {
		match self {
			Value::None => other.is_none(),
			Value::Null => other.is_null(),
			Value::Void => other.is_void(),
			Value::True => other.is_true(),
			Value::False => other.is_false(),
			Value::Thing(v) => match other {
				Value::Thing(w) => v == w,
				Value::Regex(w) => match w.regex() {
					Some(ref r) => r.is_match(v.to_string().as_str()),
					None => false,
				},
				_ => false,
			},
			Value::Regex(v) => match other {
				Value::Regex(w) => v == w,
				Value::Number(w) => match v.regex() {
					Some(ref r) => r.is_match(w.to_string().as_str()),
					None => false,
				},
				Value::Strand(w) => match v.regex() {
					Some(ref r) => r.is_match(w.as_str()),
					None => false,
				},
				_ => false,
			},
			Value::Array(v) => match other {
				Value::Array(w) => v == w,
				_ => false,
			},
			Value::Object(v) => match other {
				Value::Object(w) => v == w,
				_ => false,
			},
			Value::Strand(v) => match other {
				Value::Strand(w) => v == w,
				Value::Regex(w) => match w.regex() {
					Some(ref r) => r.is_match(v.as_str()),
					None => false,
				},
				_ => v == &other.to_strand(),
			},
			Value::Number(v) => match other {
				Value::Number(w) => v == w,
				Value::Strand(_) => v == &other.to_number(),
				Value::Regex(w) => match w.regex() {
					Some(ref r) => r.is_match(v.to_string().as_str()),
					None => false,
				},
				_ => false,
			},
			Value::Geometry(v) => match other {
				Value::Geometry(w) => v == w,
				_ => false,
			},
			Value::Duration(v) => match other {
				Value::Duration(w) => v == w,
				Value::Strand(_) => v == &other.to_duration(),
				_ => false,
			},
			Value::Datetime(v) => match other {
				Value::Datetime(w) => v == w,
				Value::Strand(_) => v == &other.to_datetime(),
				_ => false,
			},
			_ => unreachable!(),
		}
	}

	pub fn all_equal(&self, other: &Value) -> bool {
		match self {
			Value::Array(v) => v.iter().all(|v| v.equal(other)),
			_ => self.equal(other),
		}
	}

	pub fn any_equal(&self, other: &Value) -> bool {
		match self {
			Value::Array(v) => v.iter().any(|v| v.equal(other)),
			_ => self.equal(other),
		}
	}

	pub fn fuzzy(&self, other: &Value) -> bool {
		match self {
			Value::Strand(v) => match other {
				Value::Strand(w) => MATCHER.fuzzy_match(v.as_str(), w.as_str()).is_some(),
				_ => MATCHER.fuzzy_match(v.as_str(), other.to_string().as_str()).is_some(),
			},
			_ => self.equal(other),
		}
	}

	pub fn all_fuzzy(&self, other: &Value) -> bool {
		match self {
			Value::Array(v) => v.iter().all(|v| v.fuzzy(other)),
			_ => self.fuzzy(other),
		}
	}

	pub fn any_fuzzy(&self, other: &Value) -> bool {
		match self {
			Value::Array(v) => v.iter().any(|v| v.fuzzy(other)),
			_ => self.fuzzy(other),
		}
	}

	pub fn contains(&self, other: &Value) -> bool {
		match self {
			Value::Array(v) => v.iter().any(|v| v.equal(other)),
			Value::Strand(v) => match other {
				Value::Strand(w) => v.contains(w.as_str()),
				_ => v.contains(&other.to_string().as_str()),
			},
			Value::Geometry(v) => match other {
				Value::Geometry(w) => v.contains(w),
				_ => false,
			},
			_ => false,
		}
	}

	pub fn contains_all(&self, other: &Value) -> bool {
		match other {
			Value::Array(v) => v.iter().all(|v| match self {
				Value::Array(w) => w.iter().any(|w| v.equal(w)),
				Value::Geometry(_) => self.contains(v),
				_ => false,
			}),
			_ => false,
		}
	}

	pub fn contains_any(&self, other: &Value) -> bool {
		match other {
			Value::Array(v) => v.iter().any(|v| match self {
				Value::Array(w) => w.iter().any(|w| v.equal(w)),
				Value::Geometry(_) => self.contains(v),
				_ => false,
			}),
			_ => false,
		}
	}

	pub fn intersects(&self, other: &Value) -> bool {
		match self {
			Value::Geometry(v) => match other {
				Value::Geometry(w) => v.intersects(w),
				_ => false,
			},
			_ => false,
		}
	}

	// -----------------------------------
	// Sorting operations
	// -----------------------------------

	pub fn lexical_cmp(&self, other: &Value) -> Option<Ordering> {
		match (self, other) {
			(Value::Strand(a), Value::Strand(b)) => Some(lexical_sort::lexical_cmp(a, b)),
			_ => self.partial_cmp(other),
		}
	}

	pub fn natural_cmp(&self, other: &Value) -> Option<Ordering> {
		match (self, other) {
			(Value::Strand(a), Value::Strand(b)) => Some(lexical_sort::natural_cmp(a, b)),
			_ => self.partial_cmp(other),
		}
	}

	pub fn natural_lexical_cmp(&self, other: &Value) -> Option<Ordering> {
		match (self, other) {
			(Value::Strand(a), Value::Strand(b)) => Some(lexical_sort::natural_lexical_cmp(a, b)),
			_ => self.partial_cmp(other),
		}
	}
}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Value::None => write!(f, "NONE"),
			Value::Void => write!(f, "VOID"),
			Value::Null => write!(f, "NULL"),
			Value::True => write!(f, "true"),
			Value::False => write!(f, "false"),
			Value::Number(v) => write!(f, "{}", v),
			Value::Strand(v) => write!(f, "{}", v),
			Value::Duration(v) => write!(f, "{}", v),
			Value::Datetime(v) => write!(f, "{}", v),
			Value::Array(v) => write!(f, "{}", v),
			Value::Object(v) => write!(f, "{}", v),
			Value::Geometry(v) => write!(f, "{}", v),
			Value::Param(v) => write!(f, "{}", v),
			Value::Idiom(v) => write!(f, "{}", v),
			Value::Table(v) => write!(f, "{}", v),
			Value::Thing(v) => write!(f, "{}", v),
			Value::Model(v) => write!(f, "{}", v),
			Value::Regex(v) => write!(f, "{}", v),
			Value::Function(v) => write!(f, "{}", v),
			Value::Subquery(v) => write!(f, "{}", v),
			Value::Expression(v) => write!(f, "{}", v),
		}
	}
}

impl Value {
	pub(crate) fn writeable(&self) -> bool {
		match self {
			Value::Array(v) => v.iter().any(|v| v.writeable()),
			Value::Object(v) => v.iter().any(|(_, v)| v.writeable()),
			Value::Function(v) => v.args().iter().any(|v| v.writeable()),
			Value::Subquery(v) => v.writeable(),
			Value::Expression(v) => v.l.writeable() || v.r.writeable(),
			_ => false,
		}
	}

	#[cfg_attr(feature = "parallel", async_recursion)]
	#[cfg_attr(not(feature = "parallel"), async_recursion(?Send))]
	pub(crate) async fn compute(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
		doc: Option<&'async_recursion Value>,
	) -> Result<Value, Error> {
		match self {
			Value::None => Ok(Value::None),
			Value::Void => Ok(Value::Void),
			Value::Null => Ok(Value::Null),
			Value::True => Ok(Value::True),
			Value::False => Ok(Value::False),
			Value::Param(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Idiom(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Array(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Object(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Function(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Subquery(v) => v.compute(ctx, opt, txn, doc).await,
			Value::Expression(v) => v.compute(ctx, opt, txn, doc).await,
			_ => Ok(self.to_owned()),
		}
	}
}

impl Serialize for Value {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if s.is_human_readable() {
			match self {
				Value::None => s.serialize_none(),
				Value::Void => s.serialize_none(),
				Value::Null => s.serialize_none(),
				Value::True => s.serialize_bool(true),
				Value::False => s.serialize_bool(false),
				Value::Thing(v) => s.serialize_some(v),
				Value::Array(v) => s.serialize_some(v),
				Value::Object(v) => s.serialize_some(v),
				Value::Number(v) => s.serialize_some(v),
				Value::Strand(v) => s.serialize_some(v),
				Value::Geometry(v) => s.serialize_some(v),
				Value::Duration(v) => s.serialize_some(v),
				Value::Datetime(v) => s.serialize_some(v),
				_ => s.serialize_none(),
			}
		} else {
			match self {
				Value::None => s.serialize_unit_variant("Value", 0, "None"),
				Value::Void => s.serialize_unit_variant("Value", 1, "Void"),
				Value::Null => s.serialize_unit_variant("Value", 2, "Null"),
				Value::False => s.serialize_unit_variant("Value", 3, "False"),
				Value::True => s.serialize_unit_variant("Value", 4, "True"),
				Value::Number(v) => s.serialize_newtype_variant("Value", 5, "Number", v),
				Value::Strand(v) => s.serialize_newtype_variant("Value", 6, "Strand", v),
				Value::Duration(v) => s.serialize_newtype_variant("Value", 7, "Duration", v),
				Value::Datetime(v) => s.serialize_newtype_variant("Value", 8, "Datetime", v),
				Value::Array(v) => s.serialize_newtype_variant("Value", 9, "Array", v),
				Value::Object(v) => s.serialize_newtype_variant("Value", 10, "Object", v),
				Value::Geometry(v) => s.serialize_newtype_variant("Value", 11, "Geometry", v),
				Value::Param(v) => s.serialize_newtype_variant("Value", 12, "Param", v),
				Value::Idiom(v) => s.serialize_newtype_variant("Value", 13, "Idiom", v),
				Value::Table(v) => s.serialize_newtype_variant("Value", 14, "Table", v),
				Value::Thing(v) => s.serialize_newtype_variant("Value", 15, "Thing", v),
				Value::Model(v) => s.serialize_newtype_variant("Value", 16, "Model", v),
				Value::Regex(v) => s.serialize_newtype_variant("Value", 17, "Regex", v),
				Value::Function(v) => s.serialize_newtype_variant("Value", 18, "Function", v),
				Value::Subquery(v) => s.serialize_newtype_variant("Value", 19, "Subquery", v),
				Value::Expression(v) => s.serialize_newtype_variant("Value", 20, "Expression", v),
			}
		}
	}
}

impl ops::Add for Value {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		match (self, other) {
			(Value::Number(v), Value::Number(w)) => Value::Number(v + w),
			(Value::Strand(v), Value::Strand(w)) => Value::Strand(v + w),
			(Value::Datetime(v), Value::Duration(w)) => Value::Datetime(w + v),
			(Value::Duration(v), Value::Datetime(w)) => Value::Datetime(v + w),
			(Value::Duration(v), Value::Duration(w)) => Value::Duration(v + w),
			(v, w) => Value::from(v.as_number() + w.as_number()),
		}
	}
}

impl ops::Sub for Value {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		match (self, other) {
			(Value::Number(v), Value::Number(w)) => Value::Number(v - w),
			(Value::Datetime(v), Value::Duration(w)) => Value::Datetime(w - v),
			(Value::Duration(v), Value::Datetime(w)) => Value::Datetime(v - w),
			(Value::Duration(v), Value::Duration(w)) => Value::Duration(v - w),
			(v, w) => Value::from(v.as_number() - w.as_number()),
		}
	}
}

impl ops::Mul for Value {
	type Output = Self;
	fn mul(self, other: Self) -> Self {
		match (self, other) {
			(Value::Number(v), Value::Number(w)) => Value::Number(v * w),
			(v, w) => Value::from(v.as_number() * w.as_number()),
		}
	}
}

impl ops::Div for Value {
	type Output = Self;
	fn div(self, other: Self) -> Self {
		match (self, other) {
			(Value::Number(v), Value::Number(w)) => Value::Number(v / w),
			(Value::Datetime(v), Value::Duration(w)) => Value::Datetime(w / v),
			(v, w) => Value::from(v.as_number() / w.as_number()),
		}
	}
}

pub fn value(i: &str) -> IResult<&str, Value> {
	alt((double, single))(i)
}

pub fn double(i: &str) -> IResult<&str, Value> {
	map(expression, |v| Value::Expression(Box::new(v)))(i)
}

pub fn single(i: &str) -> IResult<&str, Value> {
	alt((
		map(tag_no_case("NONE"), |_| Value::None),
		map(tag_no_case("VOID"), |_| Value::Void),
		map(tag_no_case("NULL"), |_| Value::Null),
		map(tag_no_case("true"), |_| Value::True),
		map(tag_no_case("false"), |_| Value::False),
		map(subquery, |v| Value::Subquery(Box::new(v))),
		map(function, |v| Value::Function(Box::new(v))),
		map(datetime, Value::Datetime),
		map(duration, Value::Duration),
		map(geometry, Value::Geometry),
		map(number, Value::Number),
		map(strand, Value::Strand),
		map(object, Value::Object),
		map(array, Value::Array),
		map(param, Value::Param),
		map(regex, Value::Regex),
		map(thing, Value::Thing),
		map(model, Value::Model),
		map(idiom, Value::Idiom),
	))(i)
}

pub fn select(i: &str) -> IResult<&str, Value> {
	alt((
		map(tag_no_case("NONE"), |_| Value::None),
		map(tag_no_case("VOID"), |_| Value::Void),
		map(tag_no_case("NULL"), |_| Value::Null),
		map(tag_no_case("true"), |_| Value::True),
		map(tag_no_case("false"), |_| Value::False),
		map(subquery, |v| Value::Subquery(Box::new(v))),
		map(function, |v| Value::Function(Box::new(v))),
		map(datetime, Value::Datetime),
		map(duration, Value::Duration),
		map(geometry, Value::Geometry),
		map(number, Value::Number),
		map(strand, Value::Strand),
		map(object, Value::Object),
		map(array, Value::Array),
		map(param, Value::Param),
		map(regex, Value::Regex),
		map(thing, Value::Thing),
		map(model, Value::Model),
		map(table, Value::Table),
	))(i)
}

pub fn what(i: &str) -> IResult<&str, Value> {
	alt((
		map(function, |v| Value::Function(Box::new(v))),
		map(param, Value::Param),
		map(model, Value::Model),
		map(thing, Value::Thing),
		map(table, Value::Table),
	))(i)
}

pub fn json(i: &str) -> IResult<&str, Value> {
	alt((
		map(tag_no_case("NULL"), |_| Value::Null),
		map(tag_no_case("true"), |_| Value::True),
		map(tag_no_case("false"), |_| Value::False),
		map(datetime, Value::Datetime),
		map(duration, Value::Duration),
		map(geometry, Value::Geometry),
		map(number, Value::Number),
		map(object, Value::Object),
		map(array, Value::Array),
		map(strand, Value::Strand),
	))(i)
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::sql::test::Parse;

	#[test]
	fn check_none() {
		assert_eq!(true, Value::None.is_none());
		assert_eq!(true, Value::Void.is_none());
		assert_eq!(true, Value::Null.is_none());
		assert_eq!(false, Value::from(1).is_none());
	}

	#[test]
	fn check_void() {
		assert_eq!(true, Value::None.is_void());
		assert_eq!(true, Value::Void.is_void());
		assert_eq!(false, Value::Null.is_void());
		assert_eq!(false, Value::from(1).is_void());
	}

	#[test]
	fn check_null() {
		assert_eq!(true, Value::None.is_null());
		assert_eq!(false, Value::Void.is_null());
		assert_eq!(true, Value::Null.is_null());
		assert_eq!(false, Value::from(1).is_null());
	}

	#[test]
	fn check_true() {
		assert_eq!(false, Value::None.is_true());
		assert_eq!(true, Value::True.is_true());
		assert_eq!(false, Value::False.is_true());
		assert_eq!(false, Value::from(1).is_true());
		assert_eq!(true, Value::from("true").is_true());
		assert_eq!(false, Value::from("false").is_true());
		assert_eq!(false, Value::from("something").is_true());
	}

	#[test]
	fn check_false() {
		assert_eq!(false, Value::None.is_false());
		assert_eq!(false, Value::True.is_false());
		assert_eq!(true, Value::False.is_false());
		assert_eq!(false, Value::from(1).is_false());
		assert_eq!(false, Value::from("true").is_false());
		assert_eq!(true, Value::from("false").is_false());
		assert_eq!(false, Value::from("something").is_false());
	}

	#[test]
	fn convert_bool() {
		assert_eq!(false, Value::None.is_truthy());
		assert_eq!(false, Value::Null.is_truthy());
		assert_eq!(false, Value::Void.is_truthy());
		assert_eq!(true, Value::True.is_truthy());
		assert_eq!(false, Value::False.is_truthy());
		assert_eq!(false, Value::from(0).is_truthy());
		assert_eq!(true, Value::from(1).is_truthy());
		assert_eq!(true, Value::from(-1).is_truthy());
		assert_eq!(true, Value::from(1.1).is_truthy());
		assert_eq!(true, Value::from(-1.1).is_truthy());
		assert_eq!(true, Value::from("true").is_truthy());
		assert_eq!(false, Value::from("false").is_truthy());
		assert_eq!(true, Value::from("falsey").is_truthy());
		assert_eq!(true, Value::from("something").is_truthy());
	}

	#[test]
	fn convert_int() {
		assert_eq!(0, Value::None.as_int());
		assert_eq!(0, Value::Null.as_int());
		assert_eq!(0, Value::Void.as_int());
		assert_eq!(1, Value::True.as_int());
		assert_eq!(0, Value::False.as_int());
		assert_eq!(0, Value::from(0).as_int());
		assert_eq!(1, Value::from(1).as_int());
		assert_eq!(-1, Value::from(-1).as_int());
		assert_eq!(1, Value::from(1.1).as_int());
		assert_eq!(-1, Value::from(-1.1).as_int());
		assert_eq!(3, Value::from("3").as_int());
		assert_eq!(0, Value::from("true").as_int());
		assert_eq!(0, Value::from("false").as_int());
		assert_eq!(0, Value::from("something").as_int());
	}

	#[test]
	fn convert_float() {
		assert_eq!(0.0, Value::None.as_float());
		assert_eq!(0.0, Value::Null.as_float());
		assert_eq!(0.0, Value::Void.as_float());
		assert_eq!(1.0, Value::True.as_float());
		assert_eq!(0.0, Value::False.as_float());
		assert_eq!(0.0, Value::from(0).as_float());
		assert_eq!(1.0, Value::from(1).as_float());
		assert_eq!(-1.0, Value::from(-1).as_float());
		assert_eq!(1.1, Value::from(1.1).as_float());
		assert_eq!(-1.1, Value::from(-1.1).as_float());
		assert_eq!(3.0, Value::from("3").as_float());
		assert_eq!(0.0, Value::from("true").as_float());
		assert_eq!(0.0, Value::from("false").as_float());
		assert_eq!(0.0, Value::from("something").as_float());
	}

	#[test]
	fn convert_number() {
		assert_eq!(Number::from(0), Value::None.as_number());
		assert_eq!(Number::from(0), Value::Null.as_number());
		assert_eq!(Number::from(0), Value::Void.as_number());
		assert_eq!(Number::from(1), Value::True.as_number());
		assert_eq!(Number::from(0), Value::False.as_number());
		assert_eq!(Number::from(0), Value::from(0).as_number());
		assert_eq!(Number::from(1), Value::from(1).as_number());
		assert_eq!(Number::from(-1), Value::from(-1).as_number());
		assert_eq!(Number::from(1.1), Value::from(1.1).as_number());
		assert_eq!(Number::from(-1.1), Value::from(-1.1).as_number());
		assert_eq!(Number::from(3), Value::from("3").as_number());
		assert_eq!(Number::from(0), Value::from("true").as_number());
		assert_eq!(Number::from(0), Value::from("false").as_number());
		assert_eq!(Number::from(0), Value::from("something").as_number());
	}

	#[test]
	fn convert_strand() {
		assert_eq!(Strand::from("NONE"), Value::None.as_strand());
		assert_eq!(Strand::from("NULL"), Value::Null.as_strand());
		assert_eq!(Strand::from("VOID"), Value::Void.as_strand());
		assert_eq!(Strand::from("true"), Value::True.as_strand());
		assert_eq!(Strand::from("false"), Value::False.as_strand());
		assert_eq!(Strand::from("0"), Value::from(0).as_strand());
		assert_eq!(Strand::from("1"), Value::from(1).as_strand());
		assert_eq!(Strand::from("-1"), Value::from(-1).as_strand());
		assert_eq!(Strand::from("1.1"), Value::from(1.1).as_strand());
		assert_eq!(Strand::from("-1.1"), Value::from(-1.1).as_strand());
		assert_eq!(Strand::from("3"), Value::from("3").as_strand());
		assert_eq!(Strand::from("true"), Value::from("true").as_strand());
		assert_eq!(Strand::from("false"), Value::from("false").as_strand());
		assert_eq!(Strand::from("something"), Value::from("something").as_strand());
	}

	#[test]
	fn convert_string() {
		assert_eq!(String::from("NONE"), Value::None.as_string());
		assert_eq!(String::from("NULL"), Value::Null.as_string());
		assert_eq!(String::from("VOID"), Value::Void.as_string());
		assert_eq!(String::from("true"), Value::True.as_string());
		assert_eq!(String::from("false"), Value::False.as_string());
		assert_eq!(String::from("0"), Value::from(0).as_string());
		assert_eq!(String::from("1"), Value::from(1).as_string());
		assert_eq!(String::from("-1"), Value::from(-1).as_string());
		assert_eq!(String::from("1.1"), Value::from(1.1).as_string());
		assert_eq!(String::from("-1.1"), Value::from(-1.1).as_string());
		assert_eq!(String::from("3"), Value::from("3").as_string());
		assert_eq!(String::from("true"), Value::from("true").as_string());
		assert_eq!(String::from("false"), Value::from("false").as_string());
		assert_eq!(String::from("something"), Value::from("something").as_string());
	}

	#[test]
	fn check_size() {
		assert_eq!(88, std::mem::size_of::<Value>());
		assert_eq!(48, std::mem::size_of::<crate::sql::number::Number>());
		assert_eq!(24, std::mem::size_of::<crate::sql::strand::Strand>());
		assert_eq!(16, std::mem::size_of::<crate::sql::duration::Duration>());
		assert_eq!(12, std::mem::size_of::<crate::sql::datetime::Datetime>());
		assert_eq!(24, std::mem::size_of::<crate::sql::array::Array>());
		assert_eq!(24, std::mem::size_of::<crate::sql::object::Object>());
		assert_eq!(56, std::mem::size_of::<crate::sql::geometry::Geometry>());
		assert_eq!(24, std::mem::size_of::<crate::sql::param::Param>());
		assert_eq!(24, std::mem::size_of::<crate::sql::idiom::Idiom>());
		assert_eq!(24, std::mem::size_of::<crate::sql::table::Table>());
		assert_eq!(80, std::mem::size_of::<crate::sql::thing::Thing>());
		assert_eq!(48, std::mem::size_of::<crate::sql::model::Model>());
		assert_eq!(24, std::mem::size_of::<crate::sql::regex::Regex>());
		assert_eq!(8, std::mem::size_of::<Box<crate::sql::function::Function>>());
		assert_eq!(8, std::mem::size_of::<Box<crate::sql::subquery::Subquery>>());
		assert_eq!(8, std::mem::size_of::<Box<crate::sql::expression::Expression>>());
		assert_eq!(96, std::mem::size_of::<Result<Value, Error>>());
	}

	#[test]
	fn serialize_deserialize() {
		let val = Value::parse(
			"{ test: { something: [1, 'two', null, test:tobie, { something: false }] } }",
		);
		let res = Value::parse(
			"{ test: { something: [1, 'two', null, test:tobie, { something: false }] } }",
		);
		let enc: Vec<u8> = val.into();
		let dec: Value = enc.into();
		assert_eq!(res, dec);
	}
}
