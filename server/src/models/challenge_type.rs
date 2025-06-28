use crate::schema::challenge_types;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Insertable, Debug, PartialEq, Clone)]
#[diesel(table_name = challenge_types)]
pub struct ChallengeType {
    pub id: i16,
    pub name: String,
    pub description: String,
}
