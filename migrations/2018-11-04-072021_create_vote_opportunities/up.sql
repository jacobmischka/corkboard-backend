CREATE TABLE vote_opportunities (
	id serial PRIMARY KEY NOT NULL,
	title text NOT NULL,
	location_name text,
	lat double precision NOT NULL,
	lng double precision NOT NULL,
	description text NOT NULL,
	date timestamp NOT NULL,
	tags text[] NOT NULL
)
