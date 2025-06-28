use async_lock::OnceCell;
use juniper::{FieldResult, graphql_object};

use crate::{
    api::{
        Context, NodeId, NodeValue, comment::CommentConnection, post::PostValue,
        response::ResponseConnection, user::User,
    },
    models, relayify,
};

pub struct ChallengeType {
    id: NodeId,
    name: String,
    description: String,
}

#[graphql_object]
#[graphql(impl = [NodeValue], context = Context)]
impl ChallengeType {
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }
}

impl From<models::ChallengeType> for ChallengeType {
    fn from(value: models::ChallengeType) -> Self {
        Self {
            id: NodeId::from(("challenge_types", value.id)),
            name: value.name,
            description: value.description,
        }
    }
}

pub struct Challenge {
    db_id: i32,
    db_model: OnceCell<models::Challenge>,
}

#[graphql_object]
#[graphql(impl = [NodeValue, PostValue], context = Context)]
impl Challenge {
    #[graphql(skip)]
    async fn fetch_from_db<'c>(&self, ctx: &'c Context) -> FieldResult<&models::Challenge> {
        self.db_model
            .get_or_try_init(async || {
                let mut cnx = ctx.db.get().await?;
                let model = models::Challenge::find(&mut cnx, self.db_id).await?;
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
    pub(super) fn from_model(model: models::Challenge) -> Self {
        let res = Self {
            db_id: model.post.id,
            db_model: OnceCell::new(),
        };
        res.db_model.set_blocking(model).unwrap();
        res
    }

    pub fn id(&self) -> NodeId {
        NodeId::from(("challenges", self.db_id))
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
        Ok(User::new(user_id))
    }

    pub async fn challenge_type<'c>(&self, ctx: &'c Context) -> FieldResult<ChallengeType> {
        Ok(self.fetch_from_db(ctx).await?.challenge_type.clone().into())
    }

    pub async fn responses<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<ResponseConnection> {
        ResponseConnection::try_from_challenge(self.db_id, ctx, first, after, last, before).await
    }

    pub async fn comments<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<CommentConnection> {
        CommentConnection::try_from_challenge(self.db_id, ctx, first, after, last, before).await
    }
}

relayify!(
    Challenge,
    (User, async |cnx, user_id, after, before, limit| {
        models::Challenge::filter_by_user(cnx, user_id, after, before, limit).await
    })
);

impl From<models::Challenge> for Challenge {
    fn from(value: models::Challenge) -> Self {
        Self::from_model(value)
    }
}
