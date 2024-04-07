use crate::dbs::Executor;
use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::err::Error;
use crate::sql::idiom::Idiom;
use crate::sql::number::Number;
use crate::sql::value::Value;

impl Value {
	pub async fn increment(
		&mut self,
		ctx: &Runtime,
		opt: &Options,
		exe: &Executor<'_>,
		path: &Idiom,
		val: Value,
	) -> Result<(), Error> {
		match self.get(ctx, opt, exe, path).await? {
			Value::Number(v) => match val {
				Value::Number(x) => self.set(ctx, opt, exe, path, Value::from(v + x)).await,
				_ => Ok(()),
			},
			Value::Array(v) => match val {
				Value::Array(x) => self.set(ctx, opt, exe, path, Value::from(v + x)).await,
				x => self.set(ctx, opt, exe, path, Value::from(v + x)).await,
			},
			Value::None => match val {
				Value::Number(x) => {
					self.set(ctx, opt, exe, path, Value::from(Number::from(0) + x)).await
				}
				Value::Array(x) => self.set(ctx, opt, exe, path, Value::from(x)).await,
				x => self.set(ctx, opt, exe, path, Value::from(vec![x])).await,
			},
			_ => Ok(()),
		}
	}
}

#[cfg(test)]
mod tests {

	use super::*;
	use crate::dbs::test::mock;
	use crate::sql::test::Parse;

	#[tokio::test]
	async fn inc_none() {
		let (ctx, opt, exe) = mock();
		let idi = Idiom::parse("other");
		let mut val = Value::parse("{ test: 100 }");
		let res = Value::parse("{ test: 100, other: +10 }");
		val.increment(&ctx, &opt, &exe, &idi, Value::from(10)).await.unwrap();
		assert_eq!(res, val);
	}

	#[tokio::test]
	async fn inc_number() {
		let (ctx, opt, exe) = mock();
		let idi = Idiom::parse("test");
		let mut val = Value::parse("{ test: 100 }");
		let res = Value::parse("{ test: 110 }");
		val.increment(&ctx, &opt, &exe, &idi, Value::from(10)).await.unwrap();
		assert_eq!(res, val);
	}

	#[tokio::test]
	async fn inc_array_number() {
		let (ctx, opt, exe) = mock();
		let idi = Idiom::parse("test[1]");
		let mut val = Value::parse("{ test: [100, 200, 300] }");
		let res = Value::parse("{ test: [100, 210, 300] }");
		val.increment(&ctx, &opt, &exe, &idi, Value::from(10)).await.unwrap();
		assert_eq!(res, val);
	}

	#[tokio::test]
	async fn inc_array_value() {
		let (ctx, opt, exe) = mock();
		let idi = Idiom::parse("test");
		let mut val = Value::parse("{ test: [100, 200, 300] }");
		let res = Value::parse("{ test: [100, 200, 300] }");
		val.increment(&ctx, &opt, &exe, &idi, Value::from(200)).await.unwrap();
		assert_eq!(res, val);
	}

	#[tokio::test]
	async fn inc_array_array() {
		let (ctx, opt, exe) = mock();
		let idi = Idiom::parse("test");
		let mut val = Value::parse("{ test: [100, 200, 300] }");
		let res = Value::parse("{ test: [100, 200, 300, 400, 500] }");
		val.increment(&ctx, &opt, &exe, &idi, Value::parse("[100, 300, 400, 500]"))
			.await
			.unwrap();
		assert_eq!(res, val);
	}
}
