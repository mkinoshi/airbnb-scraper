use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::models::*;
use crate::schema::searches::dsl::*;

pub struct Db;

impl Db {
    pub fn establish_connection() -> PgConnection {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }

    pub fn load_all_searches(connection: &PgConnection) -> Vec<Search> {
        searches
            .load::<Search>(connection)
            .expect("Error loading searches")
    }
}
