use crate::dbs::Options;
use crate::dbs::Runtime;
use crate::dbs::Statement;
use crate::dbs::Transaction;
use crate::doc::Document;
use crate::err::Error;

impl<'a> Document<'a> {
	pub async fn store(
		&self,
		_ctx: &Runtime,
		opt: &Options,
		txn: &Transaction,
		_stm: &Statement,
	) -> Result<(), Error> {
		// Check if forced
		if !opt.force && !self.changed() {
			return Ok(());
		}
		// Check if the table is a view
		if self.tb(opt, txn).await?.drop {
			return Ok(());
		}
		// Clone transaction
		let run = txn.clone();
		// Claim transaction
		let mut run = run.lock().await;
		// Get the record id
		let rid = self.id.as_ref().unwrap();
		// Store the record data
		let key = crate::key::thing::new(opt.ns(), opt.db(), &rid.tb, &rid.id);
		run.set(key, self).await?;
		// Carry on
		Ok(())
	}
}
