use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use geo::{Point};
use geo::algorithm::vincenty_distance::VincentyDistance;
use juniper::{FieldResult, RootNode};
use uom::si::f64::Length;
use uom::si::length::{mile, meter};
use diesel::deserialize::Queryable;
use diesel::prelude::*;
use diesel::dsl::insert_into;

use ::db_schema::vote_opportunities;
use ::db::{DbConnection};

type DB = diesel::pg::Pg;

enum Opportunity {
	Vote(Vote)
}

#[derive(GraphQLObject)]
#[graphql(description = "A voting opportunity")]
struct Vote {
	id: i32,
	title: String,
	location: Location,
	description: String,
	date: DateTime<Utc>,
	tags: Vec<Tag>
}

impl Queryable<vote_opportunities::SqlType, DB> for Vote {
	type Row = (i32, String, Option<String>, f64, f64, String, NaiveDateTime, Vec<Tag>);

	fn build(row: Self::Row) -> Self {
		println!("{}", row.6);
		Vote {
			id: row.0,
			title: row.1,
			location: Location {
				name: row.2,
				point: Point::new(row.3, row.4)
			},
			description: row.5,
			date: DateTime::from_utc(row.6, Utc),
			tags: row.7
		}
	}
}

#[derive(GraphQLInputObject)]
#[derive(Insertable)]
#[graphql(description = "A new voting opportunity")]
#[table_name="vote_opportunities"]
struct VoteInput {
	title: String,
	location_name: Option<String>,
	lat: f64,
	lng: f64,
	description: String,
	date: NaiveDateTime,
	tags: Vec<Tag>
}

type Tag = String;

#[derive(GraphQLObject)]
#[graphql(description = "A user")]
struct User {
	id: i32,
	email: String,
	subscriptions: Vec<Subscription>
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A user")]
struct UserInput {
	email: String,
	subscriptions: Vec<SubscriptionInput>
}

#[derive(GraphQLObject)]
#[graphql(description = "A subscription")]
struct Subscription {
	area: Area
}

#[derive(GraphQLInputObject)]
struct SubscriptionInput {
	area: AreaInput
}


#[derive(GraphQLObject)]
struct Area {
	center: Location,
	radius: f64
}

#[derive(GraphQLInputObject)]
struct AreaInput {
	center: LocationInput,
	radius: f64
}

struct Location {
	name: Option<String>,
	point: Point<f64>
}

impl Location {
	fn from_input(input: LocationInput) -> Self {
		Self {
			name: input.name,
			point: Point::new(input.lat, input.lng)
		}
	}
}

#[derive(GraphQLInputObject)]
struct LocationInput {
	name: Option<String>,
	lat: f64,
	lng: f64
}

graphql_object!(Location: () |&self| {
	field name() -> Option<&str> {
		match self.name {
			Some(ref name) => Some(name.as_str()),
			None => None
		}
	}

	field point() -> Vec<f64> {
		vec![self.point.lng(), self.point.lat()]
	}
});

pub struct Context {
	pub db: DbConnection
}

impl juniper::Context for Context {}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {
	field apiVersion() -> &str {
		"1.0"
	}

	field votes(&executor) -> FieldResult<Vec<Vote>> {
		use ::db_schema::vote_opportunities;

		let opportunities = vote_opportunities::table
			.filter(vote_opportunities::columns::date.ge(Utc::now().naive_utc()))
			.load::<Vote>(&executor.context().db)
			.expect("error loading opportunities");

		Ok(opportunities)
	}

	field opportunitiesNearMe(&executor, lat: f64, lng: f64, radius: f64) -> FieldResult<Vec<Vote>> {

		let radius_in_miles = Length::new::<mile>(radius);
		let radius_in_meters = radius_in_miles.get::<meter>();

		use ::db_schema::vote_opportunities;

		let opportunities = vote_opportunities::table
			.filter(vote_opportunities::columns::date.ge(Utc::now().naive_utc()))
			.load::<Vote>(&executor.context().db)
			.expect("error loading opportunities");

		let p = Point::new(lat, lng);

		return Ok(
			opportunities.into_iter().filter(|v| {
				if let Ok(distance) = p.vincenty_distance(&v.location.point) {
					distance <= radius_in_meters
				} else {
					false
				}
			}).collect()
		)
	}

	field vote(&executor, id: i32) -> FieldResult<Vote> {
		use ::db_schema::vote_opportunities;

		let opp = vote_opportunities::table
			.filter(vote_opportunities::columns::id.eq(id))
			.first::<Vote>(&executor.context().db)
			.expect("error loading opportunities");

		Ok(opp)
	}
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
	field createVote(&executor, new_vote: VoteInput) -> FieldResult<Vote> {

		use ::db_schema::vote_opportunities;
		let vote = insert_into(vote_opportunities::table)
			.values(new_vote)
			.get_result::<Vote>(&executor.context().db)
			.expect("error adding opportunity");

		Ok(vote)
	}

	field deleteVote(&executor, id: i32) -> FieldResult<Vote> {
		use ::db_schema::vote_opportunities;
		let vote = diesel::delete(vote_opportunities::table)
			.filter(vote_opportunities::columns::id.eq(id))
			.get_result::<Vote>(&executor.context().db)
			.expect("error loading opportunities");

		Ok(vote)
	}
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
	Schema::new(QueryRoot {}, MutationRoot {})
}
