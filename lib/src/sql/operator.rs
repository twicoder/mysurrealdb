use crate::sql::comment::mightbespace;
use crate::sql::comment::shouldbespace;
use crate::sql::error::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::char;
use nom::combinator::map;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Operator {
	Or,  // ||
	And, // &&
	//
	Add, // +
	Sub, // -
	Mul, // *
	Div, // /
	Inc, // +=
	Dec, // -=
	//
	Exact, // ==
	//
	Equal,    // =
	NotEqual, // !=
	AllEqual, // *=
	AnyEqual, // ?=
	//
	Like,    // ~
	NotLike, // !~
	AllLike, // *~
	AnyLike, // ?~
	//
	LessThan,        // <
	LessThanOrEqual, // <=
	MoreThan,        // >
	MoreThanOrEqual, // >=
	//
	Contain,     // ∋
	NotContain,  // ∌
	ContainAll,  // ⊇
	ContainAny,  // ⊃
	ContainNone, // ⊅
	Inside,      // ∈
	NotInside,   // ∉
	AllInside,   // ⊆
	AnyInside,   // ⊂
	NoneInside,  // ⊄
	Intersects,  // ∩
}

impl Default for Operator {
	fn default() -> Operator {
		Operator::Equal
	}
}

impl Operator {
	#[inline]
	pub fn precedence(&self) -> u8 {
		match self {
			Operator::Or => 1,
			Operator::And => 2,
			Operator::Sub => 4,
			Operator::Add => 5,
			Operator::Mul => 6,
			Operator::Div => 7,
			_ => 3,
		}
	}
}

impl fmt::Display for Operator {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Operator::Or => write!(f, "OR"),
			Operator::And => write!(f, "AND"),
			Operator::Add => write!(f, "+"),
			Operator::Sub => write!(f, "-"),
			Operator::Mul => write!(f, "*"),
			Operator::Div => write!(f, "/"),
			Operator::Inc => write!(f, "+="),
			Operator::Dec => write!(f, "-="),
			Operator::Exact => write!(f, "=="),
			Operator::Equal => write!(f, "="),
			Operator::NotEqual => write!(f, "!="),
			Operator::AllEqual => write!(f, "*="),
			Operator::AnyEqual => write!(f, "?="),
			Operator::Like => write!(f, "~"),
			Operator::NotLike => write!(f, "!~"),
			Operator::AllLike => write!(f, "*~"),
			Operator::AnyLike => write!(f, "?~"),
			Operator::LessThan => write!(f, "<"),
			Operator::LessThanOrEqual => write!(f, "<="),
			Operator::MoreThan => write!(f, ">"),
			Operator::MoreThanOrEqual => write!(f, ">="),
			Operator::Contain => write!(f, "CONTAINS"),
			Operator::NotContain => write!(f, "CONTAINS NOT"),
			Operator::ContainAll => write!(f, "CONTAINS ALL"),
			Operator::ContainAny => write!(f, "CONTAINS ANY"),
			Operator::ContainNone => write!(f, "CONTAINS NONE"),
			Operator::Inside => write!(f, "INSIDE"),
			Operator::NotInside => write!(f, "NOT INSIDE"),
			Operator::AllInside => write!(f, "ALL INSIDE"),
			Operator::AnyInside => write!(f, "ANY INSIDE"),
			Operator::NoneInside => write!(f, "NONE INSIDE"),
			Operator::Intersects => write!(f, "INTERSECTS"),
		}
	}
}

pub fn assigner(i: &str) -> IResult<&str, Operator> {
	alt((
		map(char('='), |_| Operator::Equal),
		map(tag("+="), |_| Operator::Inc),
		map(tag("-="), |_| Operator::Dec),
	))(i)
}

pub fn operator(i: &str) -> IResult<&str, Operator> {
	alt((symbols, phrases))(i)
}

pub fn symbols(i: &str) -> IResult<&str, Operator> {
	let (i, _) = mightbespace(i)?;
	let (i, v) = alt((
		alt((
			map(tag("=="), |_| Operator::Exact),
			map(tag("!="), |_| Operator::NotEqual),
			map(tag("*="), |_| Operator::AllEqual),
			map(tag("?="), |_| Operator::AnyEqual),
			map(char('='), |_| Operator::Equal),
		)),
		alt((
			map(tag("!~"), |_| Operator::NotLike),
			map(tag("*~"), |_| Operator::AllLike),
			map(tag("?~"), |_| Operator::AnyLike),
			map(char('~'), |_| Operator::Like),
		)),
		alt((
			map(tag("<="), |_| Operator::LessThanOrEqual),
			map(char('<'), |_| Operator::LessThan),
			map(tag(">="), |_| Operator::MoreThanOrEqual),
			map(char('>'), |_| Operator::MoreThan),
		)),
		alt((
			map(char('+'), |_| Operator::Add),
			map(char('-'), |_| Operator::Sub),
			map(char('*'), |_| Operator::Mul),
			map(char('×'), |_| Operator::Mul),
			map(char('∙'), |_| Operator::Mul),
			map(char('/'), |_| Operator::Div),
			map(char('÷'), |_| Operator::Div),
		)),
		alt((
			map(char('∋'), |_| Operator::Contain),
			map(char('∌'), |_| Operator::NotContain),
			map(char('∈'), |_| Operator::Inside),
			map(char('∉'), |_| Operator::NotInside),
			map(char('⊇'), |_| Operator::ContainAll),
			map(char('⊃'), |_| Operator::ContainAny),
			map(char('⊅'), |_| Operator::ContainNone),
			map(char('⊆'), |_| Operator::AllInside),
			map(char('⊂'), |_| Operator::AnyInside),
			map(char('⊄'), |_| Operator::NoneInside),
		)),
	))(i)?;
	let (i, _) = mightbespace(i)?;
	Ok((i, v))
}

pub fn phrases(i: &str) -> IResult<&str, Operator> {
	let (i, _) = shouldbespace(i)?;
	let (i, v) = alt((
		alt((
			map(tag_no_case("&&"), |_| Operator::And),
			map(tag_no_case("AND"), |_| Operator::And),
			map(tag_no_case("||"), |_| Operator::Or),
			map(tag_no_case("OR"), |_| Operator::Or),
		)),
		alt((
			map(tag_no_case("IS NOT"), |_| Operator::NotEqual),
			map(tag_no_case("IS"), |_| Operator::Equal),
		)),
		alt((
			map(tag_no_case("CONTAINS ALL"), |_| Operator::ContainAll),
			map(tag_no_case("CONTAINS ANY"), |_| Operator::ContainAny),
			map(tag_no_case("CONTAINS NONE"), |_| Operator::ContainNone),
			map(tag_no_case("CONTAINS NOT"), |_| Operator::NotContain),
			map(tag_no_case("CONTAINS"), |_| Operator::Contain),
			map(tag_no_case("ALL INSIDE"), |_| Operator::AllInside),
			map(tag_no_case("ANY INSIDE"), |_| Operator::AnyInside),
			map(tag_no_case("NONE INSIDE"), |_| Operator::NoneInside),
			map(tag_no_case("NOT INSIDE"), |_| Operator::NotInside),
			map(tag_no_case("INSIDE"), |_| Operator::Inside),
			map(tag_no_case("OUTSIDE"), |_| Operator::NotInside),
			map(tag_no_case("INTERSECTS"), |_| Operator::Intersects),
		)),
	))(i)?;
	let (i, _) = shouldbespace(i)?;
	Ok((i, v))
}
