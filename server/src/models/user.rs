use crate::schema::users;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Insertable, Debug, PartialEq, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,

    pub profile: String,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}
