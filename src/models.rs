use crate::schema::searches;
use diesel::pg::data_types::PgTimestamp;
use diesel::{AsChangeset, Identifiable, Queryable};
use uuid::Uuid;

#[derive(Queryable, Identifiable, Debug, PartialEq)]
#[table_name = "searches"]
pub struct Search {
    pub id: Uuid,
    pub url: Option<String>,
    pub result_url: Option<String>,
    pub email: Option<String>,
    pub created_at: Option<PgTimestamp>,
    pub updated_at: Option<PgTimestamp>,
}

#[derive(AsChangeset, Identifiable)]
#[table_name = "searches"]
pub struct SeachForm {
    id: Uuid,
    result_url: String,
}
