use async_lock::OnceCell;
use juniper::{FieldResult, graphql_object};

use crate::{
    api::{
        Context, NodeId, NodeValue, challenge::Challenge, comment::CommentConnection,
        post::PostValue, user::User,
    },
    models, relayify,
};

pub struct Response {
    db_id: i32,
    db_model: OnceCell<models::Response>,
}

#[graphql_object]
#[graphql(impl = [NodeValue, PostValue], context = Context)]
impl Response {
    #[graphql(skip)]
    async fn fetch_from_db<'c>(&self, ctx: &'c Context) -> FieldResult<&models::Response> {
        self.db_model
            .get_or_try_init(async || {
                let mut cnx = ctx.db.get().await?;
                let model = models::Response::find(&mut cnx, self.db_id).await?;
                Ok(model)
            })
            .await
    }

    #[graphql(skip)]
    pub fn new(db_id: i32) -> Self {
        Self {
            db_id,
            db_model: OnceCell::new(),
        }
    }

    #[graphql(skip)]
    pub(super) fn from_model(model: models::Response) -> Self {
        let res = Self {
            db_id: model.post.id,
            db_model: OnceCell::new(),
        };
        res.db_model.set_blocking(model).unwrap();
        res
    }

    pub fn id(&self) -> NodeId {
        NodeId::from(("responses", self.db_id))
    }

    pub async fn title<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.post.title)
    }

    pub async fn body<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.post.body)
    }

    pub async fn created_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<chrono::DateTime<chrono::Utc>> {
        Ok(self.fetch_from_db(ctx).await?.post.created_at.and_utc())
    }

    pub async fn updated_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(self
            .fetch_from_db(ctx)
            .await?
            .post
            .updated_at
            .map(|dt| dt.and_utc()))
    }

    pub async fn deleted_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(self
            .fetch_from_db(ctx)
            .await?
            .post
            .deleted_at
            .map(|dt| dt.and_utc()))
    }

    pub async fn author<'c>(&self, ctx: &'c Context) -> FieldResult<User> {
        let user_id = self.fetch_from_db(ctx).await?.post.user_id;
        Ok(ctx.loader.users.try_load(user_id).await??.into())
    }

    pub async fn code<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.data.code)
    }

    pub async fn challenge<'c>(&self, ctx: &'c Context) -> FieldResult<Challenge> {
        let challenge_id = self.fetch_from_db(ctx).await?.data.challenge_id;
        Ok(ctx.loader.challenges.try_load(challenge_id).await??.into())
    }

    pub async fn comments<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<CommentConnection> {
        CommentConnection::try_from_response(self.db_id, ctx, first, after, last, before).await
    }
}

relayify!(
    Response,
    (
        Challenge,
        async |cnx, challenge_id, after, before, limit| {
            models::Response::filter_by_challenge(cnx, challenge_id, after, before, limit).await
        }
    ),
    (User, async |cnx, user_id, after, before, limit| {
        models::Response::filter_by_user(cnx, user_id, after, before, limit).await
    })
);

impl From<models::Response> for Response {
    fn from(value: models::Response) -> Self {
        Self::from_model(value)
    }
}
