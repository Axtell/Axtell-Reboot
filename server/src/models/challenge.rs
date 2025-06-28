use crate::models::post::Post;
use crate::models::{challenge_type::ChallengeType, post::FromPost};
use crate::schema::challenges;
use diesel::{
    AsChangeset, Associations, Identifiable, Insertable, QueryResult, Queryable, Selectable,
};
use diesel_async::AsyncPgConnection;

#[derive(
    Queryable, Selectable, Identifiable, AsChangeset, Insertable, Associations, Debug, PartialEq,
)]
#[diesel(belongs_to(Post))]
#[diesel(belongs_to(ChallengeType))]
#[diesel(table_name = challenges)]
#[diesel(primary_key(post_id))]
pub struct ChallengeData {
    pub post_id: i32,
    pub challenge_type_id: i16,
}

#[derive(Queryable, Selectable, Debug, PartialEq)]
pub struct Challenge {
    #[diesel(embed)]
    pub post: Post,
    #[diesel(embed)]
    pub data: ChallengeData,
    #[diesel(embed)]
    pub challenge_type: ChallengeType,
}

impl Challenge {
    pub async fn find(cnx: &mut AsyncPgConnection, id: i32) -> QueryResult<Self> {
        use crate::schema::{challenge_types, posts};
        use diesel::{QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        challenges::table
            .find(id)
            .inner_join(posts::table)
            .inner_join(challenge_types::table)
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
        use crate::schema::{challenge_types, posts};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        challenges::table
            .inner_join(posts::table)
            .inner_join(challenge_types::table)
            .filter(posts::dsl::user_id.eq(user_id))
            .filter(posts::dsl::id.gt(after.unwrap_or_default()))
            .filter(posts::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }

    pub async fn newest(
        cnx: &mut AsyncPgConnection,
        after: Option<i32>,
        before: Option<i32>,
        limit: Option<i64>,
    ) -> QueryResult<Vec<Self>> {
        use crate::schema::{challenge_types, posts};
        use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
        use diesel_async::RunQueryDsl;

        challenges::table
            .inner_join(posts::table)
            .inner_join(challenge_types::table)
            .filter(posts::dsl::id.gt(after.unwrap_or_default()))
            .filter(posts::dsl::id.lt(before.unwrap_or(i32::MAX)))
            .select(Self::as_select())
            .order_by(posts::dsl::created_at.desc())
            .limit(limit.unwrap_or(25))
            .load(cnx)
            .await
    }
}

impl FromPost for Challenge {
    async fn from_post(cnx: &mut AsyncPgConnection, post: &Post) -> QueryResult<Self> {
        Self::find(cnx, post.id).await
    }
}
