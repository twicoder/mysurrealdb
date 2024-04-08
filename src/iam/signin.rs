use crate::cnf::SERVER_NAME;
use crate::dbs::DB;
use crate::err::Error;
use crate::iam::token::{Claims, HEADER};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey};
use surrealdb::sql::Object;
use surrealdb::Session;

pub async fn signin(vars: Object) -> Result<String, Error> {
	// Parse the speficied variables
	let ns = vars.get("NS").or_else(|| vars.get("ns"));
	let db = vars.get("DB").or_else(|| vars.get("db"));
	let sc = vars.get("SC").or_else(|| vars.get("sc"));
	// Check if the paramaters exist
	match (ns, db, sc) {
		(Some(ns), Some(db), Some(sc)) => {
			// Process the provided values
			let ns = ns.to_strand().as_string();
			let db = db.to_strand().as_string();
			let sc = sc.to_strand().as_string();
			// Attempt to signin to specified scope
			let res = super::signin::sc(ns, db, sc, vars).await?;
			// Return the result to the client
			Ok(res)
		}
		(Some(ns), Some(db), None) => {
			// Get the provided user and pass
			let user = vars.get("user");
			let pass = vars.get("pass");
			// Validate the user and pass
			match (user, pass) {
				// There is a username and password
				(Some(user), Some(pass)) => {
					// Process the provided values
					let ns = ns.to_strand().as_string();
					let db = db.to_strand().as_string();
					let user = user.to_strand().as_string();
					let pass = pass.to_strand().as_string();
					// Attempt to signin to database
					let res = super::signin::db(ns, db, user, pass).await?;
					// Return the result to the client
					Ok(res)
				}
				// There is no username or password
				_ => Err(Error::InvalidAuth),
			}
		}
		(Some(ns), None, None) => {
			// Get the provided user and pass
			let user = vars.get("user");
			let pass = vars.get("pass");
			// Validate the user and pass
			match (user, pass) {
				// There is a username and password
				(Some(user), Some(pass)) => {
					// Process the provided values
					let ns = ns.to_strand().as_string();
					let user = user.to_strand().as_string();
					let pass = pass.to_strand().as_string();
					// Attempt to signin to namespace
					let res = super::signin::ns(ns, user, pass).await?;
					// Return the result to the client
					Ok(res)
				}
				// There is no username or password
				_ => Err(Error::InvalidAuth),
			}
		}
		_ => Err(Error::InvalidAuth),
	}
}

pub async fn sc(ns: String, db: String, sc: String, vars: Object) -> Result<String, Error> {
	// Get a database reference
	let kvs = DB.get().unwrap();
	// Create a new readonly transaction
	let mut tx = kvs.transaction(false, false).await?;
	// Check if the supplied NS Login exists
	match tx.get_sc(&ns, &db, &sc).await {
		Ok(sv) => {
			match sv.signin {
				// This scope allows signin
				Some(val) => {
					// Setup the query params
					let vars = Some(vars.0);
					// Setup the query session
					let sess = Session::for_db(&ns, &db);
					// Compute the value with the params
					match kvs.compute(val, &sess, vars).await {
						// The signin value succeeded
						Ok(val) => match val.rid() {
							// There is a record returned
							Some(rid) => {
								// Create the authentication key
								let key = EncodingKey::from_secret(sv.code.as_ref());
								// Create the authentication claim
								let val = Claims {
									iss: SERVER_NAME.to_owned(),
									iat: Utc::now().timestamp(),
									nbf: Utc::now().timestamp(),
									exp: match sv.session {
										Some(v) => Utc::now() + Duration::from_std(v.0).unwrap(),
										_ => Utc::now() + Duration::hours(1),
									}
									.timestamp(),
									ns: Some(ns),
									db: Some(db),
									sc: Some(sc),
									id: Some(rid.to_raw()),
									..Claims::default()
								};
								// Create the authentication token
								match encode(&*HEADER, &val, &key) {
									// The auth token was created successfully
									Ok(tk) => Ok(tk),
									// There was an error creating the token
									_ => Err(Error::InvalidAuth),
								}
							}
							// No record was returned
							_ => Err(Error::InvalidAuth),
						},
						// The signin query failed
						_ => Err(Error::InvalidAuth),
					}
				}
				// This scope does not allow signin
				_ => Err(Error::InvalidAuth),
			}
		}
		// The scope does not exists
		_ => Err(Error::InvalidAuth),
	}
}

pub async fn db(ns: String, db: String, user: String, pass: String) -> Result<String, Error> {
	// Get a database reference
	let kvs = DB.get().unwrap();
	// Create a new readonly transaction
	let mut tx = kvs.transaction(false, false).await?;
	// Check if the supplied DB Login exists
	match tx.get_dl(&ns, &db, &user).await {
		Ok(dl) => {
			// Compute the hash and verify the password
			let hash = PasswordHash::new(&dl.hash).unwrap();
			// Attempt to verify the password using Argon2
			match Argon2::default().verify_password(pass.as_ref(), &hash) {
				Ok(_) => {
					// Create the authentication key
					let key = EncodingKey::from_secret(dl.code.as_ref());
					// Create the authentication claim
					let val = Claims {
						iss: SERVER_NAME.to_owned(),
						iat: Utc::now().timestamp(),
						nbf: Utc::now().timestamp(),
						exp: (Utc::now() + Duration::hours(1)).timestamp(),
						ns: Some(ns),
						db: Some(db),
						id: Some(user),
						..Claims::default()
					};
					// Create the authentication token
					match encode(&*HEADER, &val, &key) {
						// The auth token was created successfully
						Ok(tk) => Ok(tk),
						// There was an error creating the token
						_ => Err(Error::InvalidAuth),
					}
				}
				// The password did not verify
				_ => Err(Error::InvalidAuth),
			}
		}
		// The specified user login does not exist
		_ => Err(Error::InvalidAuth),
	}
}

pub async fn ns(ns: String, user: String, pass: String) -> Result<String, Error> {
	// Get a database reference
	let kvs = DB.get().unwrap();
	// Create a new readonly transaction
	let mut tx = kvs.transaction(false, false).await?;
	// Check if the supplied NS Login exists
	match tx.get_nl(&ns, &user).await {
		Ok(nl) => {
			// Compute the hash and verify the password
			let hash = PasswordHash::new(&nl.hash).unwrap();
			// Attempt to verify the password using Argon2
			match Argon2::default().verify_password(pass.as_ref(), &hash) {
				Ok(_) => {
					// Create the authentication key
					let key = EncodingKey::from_secret(nl.code.as_ref());
					// Create the authentication claim
					let val = Claims {
						iss: SERVER_NAME.to_owned(),
						iat: Utc::now().timestamp(),
						nbf: Utc::now().timestamp(),
						exp: (Utc::now() + Duration::hours(1)).timestamp(),
						ns: Some(ns),
						id: Some(user),
						..Claims::default()
					};
					// Create the authentication token
					match encode(&*HEADER, &val, &key) {
						// The auth token was created successfully
						Ok(tk) => Ok(tk),
						// There was an error creating the token
						_ => Err(Error::InvalidAuth),
					}
				}
				// The password did not verify
				_ => Err(Error::InvalidAuth),
			}
		}
		// The specified user login does not exist
		_ => Err(Error::InvalidAuth),
	}
}
