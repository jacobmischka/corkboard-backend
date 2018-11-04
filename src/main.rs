extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate geo;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate juniper;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate uom;

use actix::prelude::*;
use actix_web::{
	http,
	middleware,
	server,
	App,
	AsyncResponder,
	Error,
	FutureResponse,
	HttpRequest,
	HttpResponse,
	Json,
	State
};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

mod db;
mod db_schema;
mod schema;

use schema::{Schema, create_schema};
use db::{DbConnection, establish_connection};

struct AppState {
	executor: Addr<GraphQLExecutor>
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
	type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
	schema: std::sync::Arc<Schema>
}

impl GraphQLExecutor {
	fn new(schema: std::sync::Arc<Schema>) -> Self {
		Self { schema }
	}
}

impl Actor for GraphQLExecutor {
	type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
	type Result = Result<String, Error>;

	fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
		let res = msg.0.execute(&self.schema, &schema::Context {
			db: establish_connection()
		});
		let res_text = serde_json::to_string(&res)?;
		Ok(res_text)
	}
}

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
	let html = graphiql_source("http://127.0.0.1:8000/graphql");
	Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))
}

fn graphql(
	(st, data): (State<AppState>, Json<GraphQLData>)
) -> FutureResponse<HttpResponse> {
	st.executor
		.send(data.0)
		.from_err()
		.and_then(|res| match res {
			Ok(user) => Ok(
				HttpResponse::Ok()
					.content_type("application/json")
					.body(user)
			),
			Err(err) => {
				eprintln!("{:?}", err);
				Ok(HttpResponse::InternalServerError().into())
			}
		})
		.responder()
}


fn main() {
	::std::env::set_var("RUST_LOG", "debug");
	env_logger::init();
	let sys = actix::System::new("corkboard-backend");

	let schema = std::sync::Arc::new(create_schema());
	let addr = SyncArbiter::start(3, move || GraphQLExecutor::new(schema.clone()));

	server::new(move || {
		App::with_state(AppState { executor: addr.clone() })
			.middleware(middleware::Logger::default())
			.resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
			.resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
	}).bind("127.0.0.1:8000")
	.unwrap()
	.start();

	println!("Started on localhost:8000");

	let _ = sys.run();
}
