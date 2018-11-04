use diesel::Connection;
use dotenv::dotenv;
use std::env;

pub type DbConnection = diesel::pg::PgConnection;

pub fn establish_connection() -> DbConnection {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	DbConnection::establish(&database_url).expect("Error connecting to db")
}
