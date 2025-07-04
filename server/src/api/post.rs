use juniper::{FieldResult, graphql_interface};

use crate::{
    api::{Challenge, Context, NodeId, NodeValue, Response, User},
    models::{self, post::FromPost},
};

#[allow(async_fn_in_trait)] // juniper relies on `async` being present in the fn def for proper codegen
#[graphql_interface]
#[graphql(for = [Challenge, Response], impl = NodeValue, context = Context)]
pub trait Post: Sized {
    type ModelType: FromPost;

    #[graphql(skip)]
    async fn fetch_from_db<'c, 'a>(&'a self, ctx: &'c Context) -> FieldResult<&'a Self::ModelType>
    where
        Self::ModelType: 'a;

    #[graphql(skip)]
    fn new(db_id: i32) -> Self;

    #[graphql(skip)]
    fn from_model(model: Self::ModelType) -> Self;

    async fn id<'c>(&self, ctx: &'c Context) -> FieldResult<&NodeId>;

    async fn title<'c>(&self, ctx: &'c Context) -> FieldResult<&String>;

    async fn body<'c>(&self, ctx: &'c Context) -> FieldResult<&String>;

    async fn created_at<'c>(&self, ctx: &'c Context) -> FieldResult<chrono::DateTime<chrono::Utc>>;

    async fn updated_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<Option<chrono::DateTime<chrono::Utc>>>;

    async fn deleted_at<'c>(
        &self,
        ctx: &'c Context,
    ) -> FieldResult<Option<chrono::DateTime<chrono::Utc>>>;

    async fn author<'c>(&self, ctx: &'c Context) -> FieldResult<User>;
}

impl PostValue {
    pub async fn try_from_db_id<'c>(db_id: i32, ctx: &'c Context) -> FieldResult<Self> {
        let mut cnx = ctx.db.get().await?;
        let post = ctx.loader.posts.try_load(db_id).await??;
        if let Ok(m) = models::Challenge::from_post(&mut cnx, &post).await {
            return Ok(Self::from(Challenge::from_model(m)));
        } else if let Ok(m) = models::Response::from_post(&mut cnx, &post).await {
            return Ok(Self::from(Response::from_model(m)));
        }
        Err(format!("unknown post: {db_id}").into())
    }
}
