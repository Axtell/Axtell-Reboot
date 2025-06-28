use crate::models::user::User;
use crate::schema::posts;
use diesel::{
    AsChangeset, Identifiable, Insertable, QueryResult, Queryable, Selectable,
    prelude::Associations,
};
use diesel_async::AsyncPgConnection;

#[derive(
    Queryable, Selectable, Identifiable, AsChangeset, Insertable, Associations, Debug, PartialEq,
)]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,

    pub user_id: i32,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl Post {
    pub async fn find(cnx: &mut AsyncPgConnection, db_id: i32) -> QueryResult<Self> {
        use diesel::QueryDsl;
        use diesel_async::RunQueryDsl;

        posts::table.find(db_id).first(cnx).await
    }
}

pub trait FromPost: Sized {
    async fn from_post(cnx: &mut AsyncPgConnection, post: &Post) -> QueryResult<Self>;
}
