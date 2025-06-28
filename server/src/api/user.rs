use async_lock::OnceCell;
use juniper::{FieldResult, graphql_object};

use crate::{
    api::{
        Context, NodeId, NodeValue, challenge::ChallengeConnection, comment::CommentConnection,
        response::ResponseConnection,
    },
    models,
};

pub struct User {
    db_id: i32,
    db_model: OnceCell<models::User>,
}

#[graphql_object]
#[graphql(impl = [NodeValue], context = Context)]
impl User {
    #[graphql(skip)]
    async fn fetch_from_db<'c>(&self, ctx: &'c Context) -> FieldResult<&models::User> {
        self.db_model
            .get_or_try_init(async || {
                use crate::schema::users::dsl::*;
                use diesel::QueryDsl;
                use diesel_async::RunQueryDsl;
                let mut cnx = ctx.db.get().await?;
                let model = users.find(self.db_id).first(&mut cnx).await?;
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

    pub fn id(&self) -> NodeId {
        NodeId::from(("users", self.db_id))
    }

    pub async fn name<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.name)
    }

    pub async fn profile<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.profile)
    }

    pub async fn created_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<chrono::DateTime<chrono::Utc>> {
        Ok(self.fetch_from_db(ctx).await?.created_at.and_utc())
    }

    pub async fn updated_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(self
            .fetch_from_db(ctx)
            .await?
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
            .deleted_at
            .map(|dt| dt.and_utc()))
    }

    #[graphql(skip)]
    pub(super) fn from_model(model: models::User) -> Self {
        let res = Self {
            db_id: model.id,
            db_model: OnceCell::new(),
        };
        res.db_model.set_blocking(model).unwrap();
        res
    }

    pub async fn challenges<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<ChallengeConnection> {
        ChallengeConnection::try_from_user(self.db_id, ctx, first, after, last, before).await
    }

    pub async fn responses<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<ResponseConnection> {
        ResponseConnection::try_from_user(self.db_id, ctx, first, after, last, before).await
    }

    pub async fn comments<'c>(
        &self,
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<CommentConnection> {
        CommentConnection::try_from_user(self.db_id, ctx, first, after, last, before).await
    }
}

impl From<models::User> for User {
    fn from(value: models::User) -> Self {
        Self::from_model(value)
    }
}
