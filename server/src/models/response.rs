use crate::models::post::Post;
use crate::models::{challenge::Challenge, post::FromPost};
use crate::schema::responses;
use diesel::{
    AsChangeset, Associations, Identifiable, Insertable, QueryResult, Queryable, Selectable,
};
use diesel_async::AsyncPgConnection;

#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    Associations,
    Debug,
    PartialEq,
    Clone,
)]
#[diesel(belongs_to(Post))]
#[diesel(belongs_to(Challenge))]
#[diesel(table_name = responses)]
#[diesel(primary_key(post_id))]
pub struct ResponseData {
    pub post_id: i32,
    pub challenge_id: i32,
    pub code: String,
}

#[derive(Queryable, Selectable, Debug, PartialEq, Clone)]
pub struct Response {
    #[diesel(embed)]
    pub post: Post,
    #[diesel(embed)]
    pub data: ResponseData,
}

impl Response {
    pub async fn find(cnx: &mut AsyncPgConnection, id: i32) -> QueryResult<Self> {
        use crate::schema::posts;
        use diesel::{QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        responses::table
            .find(id)
            .inner_join(posts::table)
            .select(Self::as_select())
            .first(cnx)
            .await
    }

    pub async fn filter_by_user(
        cnx: &mut AsyncPgConnection,
        user_id: i32,
        after: Option<i32>,
        before: Option<i32>,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use crate::schema::{posts, responses};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        responses::table
            .inner_join(posts::table)
            .filter(posts::dsl::user_id.eq(user_id))
            .filter(posts::dsl::id.gt(after.unwrap_or_default()))
            .filter(posts::dsl::id.lt(before.unwrap_or(i32::MAX)))
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
        use crate::schema::{posts, responses};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        responses::table
            .inner_join(posts::table)
            .filter(responses::dsl::challenge_id.eq(challenge_id))
            .filter(posts::dsl::id.gt(after.unwrap_or_default()))
            .filter(posts::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }
}

impl FromPost for Response {
    async fn from_post(cnx: &mut AsyncPgConnection, post: &Post) -> QueryResult<Self> {
        Self::find(cnx, post.id).await
    }
}
