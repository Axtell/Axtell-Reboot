use async_lock::OnceCell;
use juniper::{FieldResult, graphql_object};

use crate::{
    api::{Context, NodeId, NodeValue, User, post::PostValue},
    models, relayify,
};

pub struct Comment {
    db_id: i32,
    db_model: OnceCell<models::Comment>,
}

#[graphql_object]
#[graphql(impl = [NodeValue], context = Context)]
impl Comment {
    #[graphql(skip)]
    async fn fetch_from_db<'c>(&self, ctx: &'c Context) -> FieldResult<&models::Comment> {
        self.db_model
            .get_or_try_init(async || {
                let mut cnx = ctx.db.get().await?;
                let model = models::Comment::find(&mut cnx, self.db_id).await?;
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
    pub(super) fn from_model(model: models::Comment) -> Self {
        let res = Self {
            db_id: model.id,
            db_model: OnceCell::new(),
        };
        res.db_model.set_blocking(model).unwrap();
        res
    }

    pub fn id(&self) -> NodeId {
        NodeId::from(("comments", self.db_id))
    }

    pub async fn body<'c>(&self, ctx: &'c Context) -> FieldResult<&String> {
        Ok(&self.fetch_from_db(ctx).await?.body)
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

    pub async fn author<'c>(&self, ctx: &'c Context) -> FieldResult<User> {
        let user_id = self.fetch_from_db(ctx).await?.user_id;
        Ok(ctx.loader.users.try_load(user_id).await??.into())
    }

    pub async fn post<'c>(&self, ctx: &'c Context) -> FieldResult<PostValue> {
        let post_id = self.fetch_from_db(ctx).await?.post_id;
        PostValue::try_from_db_id(post_id, ctx).await
    }
}

impl From<models::Comment> for Comment {
    fn from(value: models::Comment) -> Self {
        Self::from_model(value)
    }
}

relayify!(
    Comment,
    (User, async |cnx, user_id, after, before, limit| {
        models::Comment::filter_by_user(cnx, user_id, after, before, limit).await
    }),
    (
        Challenge,
        async |cnx, challenge_id, after, before, limit| {
            models::Comment::filter_by_challenge(cnx, challenge_id, after, before, limit).await
        }
    ),
    (Response, async |cnx, response_id, after, before, limit| {
        models::Comment::filter_by_response(cnx, response_id, after, before, limit).await
    })
);
