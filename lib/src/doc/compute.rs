use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Statement;
use crate::dbs::Transaction;
use crate::doc::Document;
use crate::err::Error;
use crate::sql::thing::Thing;
use crate::sql::value::Value;

impl<'a> Document<'a> {
	pub async fn compute(
		ctx: Runtime,
		opt: Options,
		txn: Transaction,
		stm: Statement<'_>,
		thg: Option<Thing>,
		val: Value,
	) -> Result<Value, Error> {
		// Setup a new document
		let mut doc = Document::new(thg, &val);
		// Process the statement
		match stm {
			Statement::Select(_) => doc.select(&ctx, &opt, &txn, &stm).await,
			Statement::Create(_) => doc.create(&ctx, &opt, &txn, &stm).await,
			Statement::Update(_) => doc.update(&ctx, &opt, &txn, &stm).await,
			Statement::Relate(_) => doc.relate(&ctx, &opt, &txn, &stm).await,
			Statement::Delete(_) => doc.delete(&ctx, &opt, &txn, &stm).await,
			Statement::Insert(_) => doc.insert(&ctx, &opt, &txn, &stm).await,
			_ => unreachable!(),
		}
	}
}
