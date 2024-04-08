use crate::err::Error;
use crate::net::head;
use crate::net::output;
use crate::net::session;
use crate::net::DB;
use bytes::Bytes;
use serde::Deserialize;
use std::str;
use surrealdb::sql::Value;
use surrealdb::Session;
use warp::path;
use warp::Filter;

const MAX: u64 = 1024 * 16; // 16 KiB

#[derive(Default, Deserialize, Debug, Clone)]
struct Query {
	pub limit: Option<String>,
	pub start: Option<String>,
}

pub fn config() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	// ------------------------------
	// Routes for OPTIONS
	// ------------------------------

	let base = warp::path("key");
	// Set opts method
	let opts = base.and(warp::options()).map(warp::reply);

	// ------------------------------
	// Routes for a table
	// ------------------------------

	// Set select method
	let select = warp::any()
		.and(warp::get())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String).and(warp::path::end()))
		.and(warp::query())
		.and_then(select_all);
	// Set create method
	let create = warp::any()
		.and(warp::post())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String).and(warp::path::end()))
		.and(warp::body::content_length_limit(MAX))
		.and(warp::body::bytes())
		.and_then(create_all);
	// Set delete method
	let delete = warp::any()
		.and(warp::delete())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String).and(warp::path::end()))
		.and_then(delete_all);
	// Specify route
	let all = select.or(create).or(delete);

	// ------------------------------
	// Routes for a thing
	// ------------------------------

	// Set select method
	let select = warp::any()
		.and(warp::get())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String / String).and(warp::path::end()))
		.and_then(select_one);
	// Set create method
	let create = warp::any()
		.and(warp::post())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String / String).and(warp::path::end()))
		.and(warp::body::content_length_limit(MAX))
		.and(warp::body::bytes())
		.and_then(create_one);
	// Set update method
	let update = warp::any()
		.and(warp::put())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String / String).and(warp::path::end()))
		.and(warp::body::content_length_limit(MAX))
		.and(warp::body::bytes())
		.and_then(update_one);
	// Set modify method
	let modify = warp::any()
		.and(warp::patch())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String / String).and(warp::path::end()))
		.and(warp::body::content_length_limit(MAX))
		.and(warp::body::bytes())
		.and_then(modify_one);
	// Set delete method
	let delete = warp::any()
		.and(warp::delete())
		.and(session::build())
		.and(warp::header::<String>(http::header::CONTENT_TYPE.as_str()))
		.and(path!("key" / String / String).and(warp::path::end()))
		.and_then(delete_one);
	// Specify route
	let one = select.or(create).or(update).or(modify).or(delete);

	// ------------------------------
	// All routes
	// ------------------------------

	// Specify route
	opts.or(all).or(one).with(head::cors())
}

// ------------------------------
// Routes for a table
// ------------------------------

async fn select_all(
	session: Session,
	output: String,
	table: String,
	query: Query,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let sql = format!(
		"SELECT * FROM type::table($table) LIMIT {l} START {s}",
		l = query.limit.unwrap_or_else(|| String::from("100")),
		s = query.start.unwrap_or_else(|| String::from("0")),
	);
	let vars = map! {
		String::from("table") => Value::from(table),
	};
	match db.execute(sql.as_str(), &session, Some(vars)).await {
		Ok(ref res) => match output.as_ref() {
			"application/json" => Ok(output::json(res)),
			"application/cbor" => Ok(output::cbor(res)),
			"application/msgpack" => Ok(output::pack(&res)),
			_ => Err(warp::reject::not_found()),
		},
		Err(err) => Err(warp::reject::custom(Error::from(err))),
	}
}

async fn create_all(
	session: Session,
	output: String,
	table: String,
	body: Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let data = str::from_utf8(&body).unwrap();
	match surrealdb::sql::json(data) {
		Ok(data) => {
			let sql = "CREATE type::table($table) CONTENT $data";
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("data") => data,
			};
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match output.as_ref() {
					"application/json" => Ok(output::json(&res)),
					"application/cbor" => Ok(output::cbor(&res)),
					"application/msgpack" => Ok(output::pack(&res)),
					_ => Err(warp::reject::not_found()),
				},
				Err(err) => Err(warp::reject::custom(Error::from(err))),
			}
		}
		Err(_) => Err(warp::reject::custom(Error::Request)),
	}
}

async fn delete_all(
	session: Session,
	output: String,
	table: String,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let sql = "DELETE type::table($table)";
	let vars = map! {
		String::from("table") => Value::from(table),
	};
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match output.as_ref() {
			"application/json" => Ok(output::json(&res)),
			"application/cbor" => Ok(output::cbor(&res)),
			"application/msgpack" => Ok(output::pack(&res)),
			_ => Err(warp::reject::not_found()),
		},
		Err(err) => Err(warp::reject::custom(Error::from(err))),
	}
}

// ------------------------------
// Routes for a thing
// ------------------------------

async fn select_one(
	session: Session,
	output: String,
	table: String,
	id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let sql = "SELECT * FROM type::thing($table, $id)";
	let vars = map! {
		String::from("table") => Value::from(table),
		String::from("id") => Value::from(id),
	};
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match output.as_ref() {
			"application/json" => Ok(output::json(&res)),
			"application/cbor" => Ok(output::cbor(&res)),
			"application/msgpack" => Ok(output::pack(&res)),
			_ => Err(warp::reject::not_found()),
		},
		Err(err) => Err(warp::reject::custom(Error::from(err))),
	}
}

async fn create_one(
	session: Session,
	output: String,
	table: String,
	id: String,
	body: Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let data = str::from_utf8(&body).unwrap();
	match surrealdb::sql::json(data) {
		Ok(data) => {
			let sql = "CREATE type::thing($table, $id) CONTENT $data";
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => Value::from(id),
				String::from("data") => data,
			};
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match output.as_ref() {
					"application/json" => Ok(output::json(&res)),
					"application/cbor" => Ok(output::cbor(&res)),
					"application/msgpack" => Ok(output::pack(&res)),
					_ => Err(warp::reject::not_found()),
				},
				Err(err) => Err(warp::reject::custom(Error::from(err))),
			}
		}
		Err(_) => Err(warp::reject::custom(Error::Request)),
	}
}

async fn update_one(
	session: Session,
	output: String,
	table: String,
	id: String,
	body: Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let data = str::from_utf8(&body).unwrap();
	match surrealdb::sql::json(data) {
		Ok(data) => {
			let sql = "UPDATE type::thing($table, $id) CONTENT $data";
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => Value::from(id),
				String::from("data") => data,
			};
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match output.as_ref() {
					"application/json" => Ok(output::json(&res)),
					"application/cbor" => Ok(output::cbor(&res)),
					"application/msgpack" => Ok(output::pack(&res)),
					_ => Err(warp::reject::not_found()),
				},
				Err(err) => Err(warp::reject::custom(Error::from(err))),
			}
		}
		Err(_) => Err(warp::reject::custom(Error::Request)),
	}
}

async fn modify_one(
	session: Session,
	output: String,
	table: String,
	id: String,
	body: Bytes,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let data = str::from_utf8(&body).unwrap();
	match surrealdb::sql::json(data) {
		Ok(data) => {
			let sql = "UPDATE type::thing($table, $id) MERGE $data";
			let vars = map! {
				String::from("table") => Value::from(table),
				String::from("id") => Value::from(id),
				String::from("data") => data,
			};
			match db.execute(sql, &session, Some(vars)).await {
				Ok(res) => match output.as_ref() {
					"application/json" => Ok(output::json(&res)),
					"application/cbor" => Ok(output::cbor(&res)),
					"application/msgpack" => Ok(output::pack(&res)),
					_ => Err(warp::reject::not_found()),
				},
				Err(err) => Err(warp::reject::custom(Error::from(err))),
			}
		}
		Err(_) => Err(warp::reject::custom(Error::Request)),
	}
}

async fn delete_one(
	session: Session,
	output: String,
	table: String,
	id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
	let db = DB.get().unwrap();
	let sql = "DELETE type::thing($table, $id)";
	let vars = map! {
		String::from("table") => Value::from(table),
		String::from("id") => Value::from(id),
	};
	match db.execute(sql, &session, Some(vars)).await {
		Ok(res) => match output.as_ref() {
			"application/json" => Ok(output::json(&res)),
			"application/cbor" => Ok(output::cbor(&res)),
			"application/msgpack" => Ok(output::pack(&res)),
			_ => Err(warp::reject::not_found()),
		},
		Err(err) => Err(warp::reject::custom(Error::from(err))),
	}
}
