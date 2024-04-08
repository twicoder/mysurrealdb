use crate::sql::error::IResult;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Algorithm {
	EdDSA,
	Es256,
	Es384,
	Es512,
	Hs256,
	Hs384,
	Hs512,
	Ps256,
	Ps384,
	Ps512,
	Rs256,
	Rs384,
	Rs512,
}

impl Default for Algorithm {
	fn default() -> Algorithm {
		Algorithm::Hs512
	}
}

impl fmt::Display for Algorithm {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Algorithm::EdDSA => write!(f, "EDDSA"),
			Algorithm::Es256 => write!(f, "ES256"),
			Algorithm::Es384 => write!(f, "ES384"),
			Algorithm::Es512 => write!(f, "ES512"),
			Algorithm::Hs256 => write!(f, "HS256"),
			Algorithm::Hs384 => write!(f, "HS384"),
			Algorithm::Hs512 => write!(f, "HS512"),
			Algorithm::Ps256 => write!(f, "PS256"),
			Algorithm::Ps384 => write!(f, "PS384"),
			Algorithm::Ps512 => write!(f, "PS512"),
			Algorithm::Rs256 => write!(f, "RS256"),
			Algorithm::Rs384 => write!(f, "RS384"),
			Algorithm::Rs512 => write!(f, "RS512"),
		}
	}
}

pub fn algorithm(i: &str) -> IResult<&str, Algorithm> {
	alt((
		map(tag("EDDSA"), |_| Algorithm::EdDSA),
		map(tag("ES256"), |_| Algorithm::Es256),
		map(tag("ES384"), |_| Algorithm::Es384),
		map(tag("ES512"), |_| Algorithm::Es512),
		map(tag("HS256"), |_| Algorithm::Hs256),
		map(tag("HS384"), |_| Algorithm::Hs384),
		map(tag("HS512"), |_| Algorithm::Hs512),
		map(tag("PS256"), |_| Algorithm::Ps256),
		map(tag("PS384"), |_| Algorithm::Ps384),
		map(tag("PS512"), |_| Algorithm::Ps512),
		map(tag("RS256"), |_| Algorithm::Rs256),
		map(tag("RS384"), |_| Algorithm::Rs384),
		map(tag("RS512"), |_| Algorithm::Rs512),
	))(i)
}
