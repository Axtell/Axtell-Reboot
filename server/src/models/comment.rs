use crate::models::post::Post;
use crate::models::user::User;
use crate::schema::comments;
use diesel::{
    AsChangeset, Identifiable, Insertable, QueryResult, Queryable, Selectable,
    prelude::Associations,
};
use diesel_async::AsyncPgConnection;

#[derive(
    Queryable, Selectable, Identifiable, AsChangeset, Insertable, Associations, Debug, PartialEq,
)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Post))]
#[diesel(table_name = comments)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub body: String,

    pub user_id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl Comment {
    pub async fn find(cnx: &mut AsyncPgConnection, db_id: i32) -> QueryResult<Self> {
        use diesel::QueryDsl;
        use diesel_async::RunQueryDsl;

        comments::table.find(db_id).first(cnx).await
    }

    pub async fn filter_by_user(
        cnx: &mut AsyncPgConnection,
        user_id: i32,
        after: Option<i32>,
        before: Option<i32>,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        comments::table
            .filter(comments::dsl::user_id.eq(user_id))
            .filter(comments::dsl::id.gt(after.unwrap_or_default()))
            .filter(comments::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }

    pub async fn filter_by_challenge(
        cnx: &mut AsyncPgConnection,
        challenge_id: i32,
        after: Option<i32>,
        before: Option<i32>,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use crate::schema::{challenges, posts};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        comments::table
            .inner_join(posts::table.inner_join(challenges::table))
            .filter(comments::dsl::post_id.eq(challenge_id))
            .filter(comments::dsl::id.gt(after.unwrap_or_default()))
            .filter(comments::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }

    pub async fn filter_by_response(
        cnx: &mut AsyncPgConnection,
        response_id: i32,
        after: Option<i32>,
        before: Option<i32>,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use crate::schema::{posts, responses};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        comments::table
            .inner_join(posts::table.inner_join(responses::table))
            .filter(comments::dsl::post_id.eq(response_id))
            .filter(comments::dsl::id.gt(after.unwrap_or_default()))
            .filter(comments::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }
}
