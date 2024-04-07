mod backup;
mod export;
mod import;
mod log;
mod start;
mod version;

use clap::{App, AppSettings, Arg};

fn auth_valid(v: &str) -> Result<(), String> {
	if v.contains(":") {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid user:pass value separated by a colon, \
		or use the --auth-user and --auth-pass flags\
	",
	))
}

fn file_valid(v: &str) -> Result<(), String> {
	if v.len() > 0 {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid path to a SQL file\
	",
	))
}

fn path_valid(v: &str) -> Result<(), String> {
	if v == "memory" {
		return Ok(());
	}
	if v.starts_with("file://") {
		return Ok(());
	}
	if v.starts_with("tikv://") {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid database path paramater\
	",
	))
}

fn conn_valid(v: &str) -> Result<(), String> {
	if v.starts_with("https://") {
		return Ok(());
	}
	if v.starts_with("http://") {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid database connection string\
	",
	))
}

fn from_valid(v: &str) -> Result<(), String> {
	if v.starts_with("https://") {
		return Ok(());
	}
	if v.starts_with("http://") {
		return Ok(());
	}
	if v.ends_with(".db") {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid database connection string, \
		or specify the path to a database file\
	",
	))
}

fn into_valid(v: &str) -> Result<(), String> {
	if v.starts_with("https://") {
		return Ok(());
	}
	if v.starts_with("http://") {
		return Ok(());
	}
	if v.ends_with(".db") {
		return Ok(());
	}
	Err(String::from(
		"\
		Provide a valid database connection string, \
		or specify the path to a database file\
	",
	))
}

fn key_valid(v: &str) -> Result<(), String> {
	match v.len() {
		16 => Ok(()),
		24 => Ok(()),
		32 => Ok(()),
		_ => Err(String::from(
			"\
			For AES-128 encryption use a 16 bit key, \
			for AES-192 encryption use a 24 bit key, \
			and for AES-256 encryption use a 32 bit key\
		",
		)),
	}
}

pub fn init() {
	let setup = App::new("SurrealDB command-line interface and server")
		.setting(AppSettings::DisableVersionFlag)
		.setting(AppSettings::ArgRequiredElseHelp)
		.arg(
			Arg::new("verbose")
				.short('v')
				.long("verbose")
				.takes_value(false)
				.multiple_occurrences(true)
				.help("Specify the log output verbosity"),
		);

	let setup = setup.subcommand(
		App::new("start")
			.display_order(1)
			.about("Start the database server")
			.arg(
				Arg::new("path")
					.index(1)
					.required(false)
					.validator(path_valid)
					.default_value("memory")
					.help("Database path used for storing data"),
			)
			.arg(
				Arg::new("auth")
					.short('a')
					.long("auth")
					.forbid_empty_values(true)
					.validator(auth_valid)
					.default_value("root:root")
					.help("Master database authentication details"),
			)
			.arg(
				Arg::new("auth-user")
					.short('u')
					.long("auth-user")
					.forbid_empty_values(true)
					.default_value("root")
					.help("The master username for the database"),
			)
			.arg(
				Arg::new("auth-pass")
					.short('p')
					.long("auth-pass")
					.forbid_empty_values(true)
					.default_value("root")
					.help("The master password for the database"),
			)
			.arg(
				Arg::new("auth-addr")
					.long("auth-addr")
					.number_of_values(1)
					.forbid_empty_values(true)
					.multiple_occurrences(true)
					.default_value("127.0.0.1/32")
					.help("The allowed networks for master authentication"),
			)
			.arg(
				Arg::new("bind")
					.short('b')
					.long("bind")
					.forbid_empty_values(true)
					.default_value("0.0.0.0:3000")
					.help("The hostname or ip address to listen for connections on"),
			)
			.arg(
				Arg::new("key")
					.short('k')
					.long("key")
					.takes_value(true)
					.forbid_empty_values(true)
					.validator(key_valid)
					.help("Encryption key to use for on-disk encryption"),
			)
			.arg(
				Arg::new("kvs-ca")
					.long("kvs-ca")
					.takes_value(true)
					.forbid_empty_values(true)
					.help("Path to the CA file used when connecting to the remote KV store"),
			)
			.arg(
				Arg::new("kvs-crt")
					.long("kvs-crt")
					.takes_value(true)
					.forbid_empty_values(true)
					.help(
						"Path to the certificate file used when connecting to the remote KV store",
					),
			)
			.arg(
				Arg::new("kvs-key")
					.long("kvs-key")
					.takes_value(true)
					.forbid_empty_values(true)
					.help(
						"Path to the private key file used when connecting to the remote KV store",
					),
			)
			.arg(
				Arg::new("web-crt")
					.long("web-crt")
					.takes_value(true)
					.forbid_empty_values(true)
					.help("Path to the certificate file for encrypted client connections"),
			)
			.arg(
				Arg::new("web-key")
					.long("web-key")
					.takes_value(true)
					.forbid_empty_values(true)
					.help("Path to the private key file for encrypted client connections"),
			),
	);

	let setup = setup.subcommand(
		App::new("backup")
			.display_order(2)
			.about("Backup data to or from an existing database")
			.arg(
				Arg::new("from")
					.index(1)
					.required(true)
					.validator(from_valid)
					.help("Path to the remote database or file from which to export"),
			)
			.arg(
				Arg::new("into")
					.index(2)
					.required(true)
					.validator(into_valid)
					.help("Path to the remote database or file into which to import"),
			)
			.arg(
				Arg::new("user")
					.short('u')
					.long("user")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication username to use when connecting"),
			)
			.arg(
				Arg::new("pass")
					.short('p')
					.long("pass")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication password to use when connecting"),
			),
	);

	let setup = setup.subcommand(
		App::new("import")
			.display_order(3)
			.about("Import a SQL script into an existing database")
			.arg(
				Arg::new("file")
					.index(1)
					.required(true)
					.validator(file_valid)
					.help("Path to the sql file to import"),
			)
			.arg(
				Arg::new("ns")
					.long("ns")
					.required(true)
					.forbid_empty_values(true)
					.help("The namespace to import the data into"),
			)
			.arg(
				Arg::new("db")
					.long("db")
					.required(true)
					.forbid_empty_values(true)
					.help("The database to import the data into"),
			)
			.arg(
				Arg::new("conn")
					.short('c')
					.long("conn")
					.forbid_empty_values(true)
					.validator(conn_valid)
					.default_value("https://surreal.io")
					.help("Remote database server url to connect to"),
			)
			.arg(
				Arg::new("user")
					.short('u')
					.long("user")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication username to use when connecting"),
			)
			.arg(
				Arg::new("pass")
					.short('p')
					.long("pass")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication password to use when connecting"),
			),
	);

	let setup = setup.subcommand(
		App::new("export")
			.display_order(4)
			.about("Export an existing database into a SQL script")
			.arg(
				Arg::new("file")
					.index(1)
					.required(true)
					.validator(file_valid)
					.help("Path to the sql file to export"),
			)
			.arg(
				Arg::new("ns")
					.long("ns")
					.required(true)
					.forbid_empty_values(true)
					.help("The namespace to export the data from"),
			)
			.arg(
				Arg::new("db")
					.long("db")
					.required(true)
					.forbid_empty_values(true)
					.help("The database to export the data from"),
			)
			.arg(
				Arg::new("conn")
					.short('c')
					.long("conn")
					.forbid_empty_values(true)
					.validator(conn_valid)
					.default_value("https://surreal.io")
					.help("Remote database server url to connect to"),
			)
			.arg(
				Arg::new("user")
					.short('u')
					.long("user")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication username to use when connecting"),
			)
			.arg(
				Arg::new("pass")
					.short('p')
					.long("pass")
					.forbid_empty_values(true)
					.default_value("root")
					.help("Database authentication password to use when connecting"),
			),
	);

	let setup = setup.subcommand(
		App::new("version")
			.display_order(5)
			.about("Output the command-line tool version information"),
	);

	let matches = setup.get_matches();

	let verbose = matches.occurrences_of("verbose") as usize;

	log::init(verbose);

	let output = match matches.subcommand() {
		Some(("start", m)) => start::init(m),
		Some(("backup", m)) => backup::init(m),
		Some(("import", m)) => import::init(m),
		Some(("export", m)) => export::init(m),
		Some(("version", m)) => version::init(m),
		_ => Ok(()),
	};

	match output {
		Err(e) => {
			error!("{}", e);
			return ();
		}
		Ok(_) => {}
	};
}
