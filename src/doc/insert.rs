use crate::dbs::Executor;
use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Statement;
use crate::doc::Document;
use crate::err::Error;
use crate::sql::value::Value;

impl Document {
	pub async fn insert(
		&mut self,
		_ctx: &Runtime,
		_opt: &Options<'_>,
		_exe: &Executor<'_>,
		_stm: &Statement<'_>,
	) -> Result<Value, Error> {
		todo!()
	}
}
