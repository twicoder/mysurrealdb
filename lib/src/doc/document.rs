use crate::dbs::Level;
use crate::dbs::Options;
use crate::dbs::Transaction;
use crate::dbs::Workable;
use crate::err::Error;
use crate::sql::statements::define::DefineEventStatement;
use crate::sql::statements::define::DefineFieldStatement;
use crate::sql::statements::define::DefineIndexStatement;
use crate::sql::statements::define::DefineTableStatement;
use crate::sql::thing::Thing;
use crate::sql::value::Value;
use std::borrow::Cow;

pub struct Document<'a> {
	pub(super) id: Option<Thing>,
	pub(super) extras: Workable,
	pub(super) current: Cow<'a, Value>,
	pub(super) initial: Cow<'a, Value>,
}

impl<'a> From<&Document<'a>> for Vec<u8> {
	fn from(val: &Document<'a>) -> Vec<u8> {
		val.current.as_ref().into()
	}
}

impl<'a> Document<'a> {
	pub fn new(id: Option<Thing>, val: &'a Value, ext: Workable) -> Self {
		Document {
			id,
			extras: ext,
			current: Cow::Borrowed(val),
			initial: Cow::Borrowed(val),
		}
	}
}

impl<'a> Document<'a> {
	// Check if document has changed
	pub fn changed(&self) -> bool {
		self.initial != self.current
	}
	// Check if document has changed
	pub fn is_new(&self) -> bool {
		self.initial.is_none()
	}
	// Get the table for this document
	pub async fn tb(
		&self,
		opt: &Options,
		txn: &Transaction,
	) -> Result<DefineTableStatement, Error> {
		// Get the record id
		let id = self.id.as_ref().unwrap();
		// Get the table definition
		let tb = txn.clone().lock().await.get_tb(opt.ns(), opt.db(), &id.tb).await;
		// Return the table or attempt to define it
		match tb {
			Ok(tb) => Ok(tb),
			Err(e) => match opt.auth.check(Level::Db) {
				true => txn.clone().lock().await.add_tb(opt.ns(), opt.db(), &id.tb).await,
				false => Err(e),
			},
		}
	}
	// Get the events for this document
	pub async fn ev(
		&self,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Vec<DefineEventStatement>, Error> {
		// Get the record id
		let id = self.id.as_ref().unwrap();
		// Get the table definition
		txn.clone().lock().await.all_ev(opt.ns(), opt.db(), &id.tb).await
	}
	// Get the fields for this document
	pub async fn fd(
		&self,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Vec<DefineFieldStatement>, Error> {
		// Get the record id
		let id = self.id.as_ref().unwrap();
		// Get the table definition
		txn.clone().lock().await.all_fd(opt.ns(), opt.db(), &id.tb).await
	}
	// Get the indexes for this document
	pub async fn ix(
		&self,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Vec<DefineIndexStatement>, Error> {
		// Get the record id
		let id = self.id.as_ref().unwrap();
		// Get the table definition
		txn.clone().lock().await.all_ix(opt.ns(), opt.db(), &id.tb).await
	}
}
