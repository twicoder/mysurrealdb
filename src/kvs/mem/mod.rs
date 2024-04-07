use crate::err::Error;
use crate::kvs::Key;
use crate::kvs::Val;
use std::ops::Range;

pub struct Datastore {
	db: echodb::Db<Key, Val>,
}

pub struct Transaction {
	// Is the transaction complete?
	ok: bool,
	// Is the transaction read+write?
	rw: bool,
	// The distributed datastore transaction
	tx: echodb::Tx<Key, Val>,
}

impl Datastore {
	// Open a new database
	pub fn new() -> Result<Datastore, Error> {
		Ok(Datastore {
			db: echodb::db::new(),
		})
	}
	// Start a new transaction
	pub async fn transaction(&self, write: bool, _: bool) -> Result<Transaction, Error> {
		match self.db.begin(write).await {
			Ok(tx) => Ok(Transaction {
				ok: false,
				rw: write,
				tx,
			}),
			Err(_) => Err(Error::TxError),
		}
	}
}

impl Transaction {
	// Check if closed
	pub fn closed(&self) -> bool {
		self.ok
	}
	// Cancel a transaction
	pub fn cancel(&mut self) -> Result<(), Error> {
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Mark this transaction as done
		self.ok = true;
		// Cancel this transaction
		self.tx.cancel()?;
		// Continue
		Ok(())
	}
	// Commit a transaction
	pub fn commit(&mut self) -> Result<(), Error> {
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Check to see if transaction is writable
		if self.rw == false {
			return Err(Error::TxReadonlyError);
		}
		// Mark this transaction as done
		self.ok = true;
		// Cancel this transaction
		self.tx.commit()?;
		// Continue
		Ok(())
	}
	// Delete a key
	pub fn del<K>(&mut self, key: K) -> Result<(), Error>
	where
		K: Into<Key>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Check to see if transaction is writable
		if self.rw == false {
			return Err(Error::TxReadonlyError);
		}
		// Remove the key
		let res = self.tx.del(key.into())?;
		// Return result
		Ok(res)
	}
	// Check if a key exists
	pub fn exi<K>(&mut self, key: K) -> Result<bool, Error>
	where
		K: Into<Key>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Check the key
		let res = self.tx.exi(key.into())?;
		// Return result
		Ok(res)
	}
	// Fetch a key from the database
	pub fn get<K>(&mut self, key: K) -> Result<Option<Val>, Error>
	where
		K: Into<Key>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Get the key
		let res = self.tx.get(key.into())?;
		// Return result
		Ok(res)
	}
	// Insert or update a key in the database
	pub fn set<K, V>(&mut self, key: K, val: V) -> Result<(), Error>
	where
		K: Into<Key>,
		V: Into<Val>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Check to see if transaction is writable
		if self.rw == false {
			return Err(Error::TxReadonlyError);
		}
		// Set the key
		self.tx.set(key.into(), val.into())?;
		// Return result
		Ok(())
	}
	// Insert a key if it doesn't exist in the database
	pub fn put<K, V>(&mut self, key: K, val: V) -> Result<(), Error>
	where
		K: Into<Key>,
		V: Into<Val>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Check to see if transaction is writable
		if self.rw == false {
			return Err(Error::TxReadonlyError);
		}
		// Set the key
		self.tx.put(key.into(), val.into())?;
		// Return result
		Ok(())
	}
	// Retrieve a range of keys from the databases
	pub fn scan<K>(&mut self, rng: Range<K>, limit: u32) -> Result<Vec<(Key, Val)>, Error>
	where
		K: Into<Key>,
	{
		// Check to see if transaction is closed
		if self.ok == true {
			return Err(Error::TxFinishedError);
		}
		// Convert the range to bytes
		let rng: Range<Key> = Range {
			start: rng.start.into(),
			end: rng.end.into(),
		};
		// Scan the keys
		let res = self.tx.scan(rng, limit)?;
		// Return result
		Ok(res)
	}
}
