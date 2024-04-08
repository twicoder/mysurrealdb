use crate::sql::common::commas;
use crate::sql::common::escape;
use crate::sql::common::val_char;
use crate::sql::error::IResult;
use crate::sql::ident::ident_raw;
use nom::multi::separated_list1;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str;

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Tables(pub Vec<Table>);

impl fmt::Display for Tables {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.0.iter().map(|ref v| format!("{}", v)).collect::<Vec<_>>().join(", "))
	}
}

pub fn tables(i: &str) -> IResult<&str, Tables> {
	let (i, v) = separated_list1(commas, table)(i)?;
	Ok((i, Tables(v)))
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Table {
	pub name: String,
}

impl From<String> for Table {
	fn from(v: String) -> Self {
		Table {
			name: v,
		}
	}
}

impl fmt::Display for Table {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", escape(&self.name, &val_char, "`"))
	}
}

pub fn table(i: &str) -> IResult<&str, Table> {
	let (i, v) = ident_raw(i)?;
	Ok((
		i,
		Table {
			name: v,
		},
	))
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn table_normal() {
		let sql = "test";
		let res = table(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("test", format!("{}", out));
		assert_eq!(
			out,
			Table {
				name: String::from("test"),
			}
		);
	}

	#[test]
	fn table_quoted_backtick() {
		let sql = "`test`";
		let res = table(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("test", format!("{}", out));
		assert_eq!(
			out,
			Table {
				name: String::from("test"),
			}
		);
	}

	#[test]
	fn table_quoted_brackets() {
		let sql = "⟨test⟩";
		let res = table(sql);
		assert!(res.is_ok());
		let out = res.unwrap().1;
		assert_eq!("test", format!("{}", out));
		assert_eq!(
			out,
			Table {
				name: String::from("test"),
			}
		);
	}
}
