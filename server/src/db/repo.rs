use std::{collections::HashMap, future, sync::Arc};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};

use futures::TryStreamExt;

use crate::{
    db::DbPool,
    models::{Challenge, ChallengeType, Comment, Post, Response, User},
};

#[derive(Clone)]
pub struct Repository {
    db_pool: DbPool,
}

impl Repository {
    pub fn new(db_pool: &DbPool) -> Self {
        Self {
            db_pool: db_pool.clone(),
        }
    }

    pub fn pool(&self) -> DbPool {
        self.db_pool.clone()
    }

    pub async fn load_users_by_ids(&self, ids: &[i32]) -> anyhow::Result<HashMap<i32, User>> {
        use crate::schema::users::dsl::*;
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(users
            .filter(id.eq_any(ids))
            .load_stream::<User>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }

    pub async fn load_posts_by_ids(&self, ids: &[i32]) -> anyhow::Result<HashMap<i32, Post>> {
        use crate::schema::posts::dsl::*;
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(posts
            .filter(id.eq_any(ids))
            .load_stream::<Post>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }

    pub async fn load_challenges_by_ids(
        &self,
        ids: &[i32],
    ) -> anyhow::Result<HashMap<i32, Challenge>> {
        use crate::schema::{challenge_types, challenges, posts};
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(challenges::table
            .inner_join(posts::table)
            .inner_join(challenge_types::table)
            .filter(posts::dsl::id.eq_any(ids))
            .select(Challenge::as_select())
            .load_stream::<Challenge>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.post.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }

    pub async fn load_responses_by_ids(
        &self,
        ids: &[i32],
    ) -> anyhow::Result<HashMap<i32, Response>> {
        use crate::schema::{posts, responses};
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(responses::table
            .inner_join(posts::table)
            .filter(posts::dsl::id.eq_any(ids))
            .select(Response::as_select())
            .load_stream::<Response>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.post.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }

    pub async fn load_comments_by_ids(&self, ids: &[i32]) -> anyhow::Result<HashMap<i32, Comment>> {
        use crate::schema::comments::dsl::*;
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(comments
            .filter(id.eq_any(ids))
            .load_stream::<Comment>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }

    pub async fn load_challenge_types_by_ids(
        &self,
        ids: &[i16],
    ) -> anyhow::Result<HashMap<i16, ChallengeType>> {
        use crate::schema::challenge_types::dsl::*;
        use diesel_async::RunQueryDsl;
        let mut cnx = self.db_pool.get().await?;
        Ok(challenge_types
            .filter(id.eq_any(ids))
            .load_stream::<ChallengeType>(&mut cnx)
            .await?
            .try_fold(HashMap::new(), |mut acc, item| {
                acc.insert(item.id, item);
                future::ready(Ok(acc))
            })
            .await?)
    }
}

macro_rules! make_loader {
    (ChallengeType) => {
        paste::paste! {
            pub struct ChallengeTypeBatcher {
                repo: Repository,
            }

            impl ChallengeTypeBatcher {
                pub fn new(db_pool: &DbPool) -> Self {
                    Self {
                        repo: Repository::new(db_pool),
                    }
                }
            }

            impl dataloader::BatchFn<i16, Result<ChallengeType, Arc<anyhow::Error>>> for ChallengeTypeBatcher {
                async fn load(&mut self, keys: &[i16]) -> HashMap<i16, Result<ChallengeType, Arc<anyhow::Error>>> {
                    match self.repo.load_challenge_types_by_ids(keys).await {
                        Ok(models) => models
                            .into_iter()
                            .map(|(id, model)| (id, Ok(model)))
                            .collect(),
                        Err(e) => {
                            let e = Arc::new(e);
                            keys.iter().map(|k| (k.clone(), Err(e.clone()))).collect()
                        }
                    }
                }
            }

            pub type ChallengeTypeLoader = dataloader::cached::Loader<i16, Result<ChallengeType, Arc<anyhow::Error>>, ChallengeTypeBatcher>;
        }

    };
    ($model:ident) => {
        paste::paste! {
            pub struct [<$model Batcher>] {
                repo: Repository,
            }

            impl [<$model Batcher>] {
                pub fn new(db_pool: &DbPool) -> Self {
                    Self {
                        repo: Repository::new(db_pool),
                    }
                }
            }

            impl dataloader::BatchFn<i32, Result<$model, Arc<anyhow::Error>>> for [<$model Batcher>] {
                async fn load(&mut self, keys: &[i32]) -> HashMap<i32, Result<$model, Arc<anyhow::Error>>> {
                    match self.repo.[<load_ $model:snake:lower s _by_ids>](keys).await {
                        Ok(models) => models
                            .into_iter()
                            .map(|(id, model)| (id, Ok(model)))
                            .collect(),
                        Err(e) => {
                            let e = Arc::new(e);
                            keys.iter().map(|k| (k.clone(), Err(e.clone()))).collect()
                        }
                    }
                }
            }

            pub type [<$model Loader>] = dataloader::cached::Loader<i32, Result<$model, Arc<anyhow::Error>>, [<$model Batcher>]>;
        }

    };
}

make_loader!(User);
make_loader!(Post);
make_loader!(Challenge);
make_loader!(Response);
make_loader!(Comment);
make_loader!(ChallengeType);

pub struct Loader {
    pub users: UserLoader,
    pub posts: PostLoader,
    pub challenges: ChallengeLoader,
    pub responses: ResponseLoader,
    pub comments: CommentLoader,
    pub challenge_type: ChallengeTypeLoader,
}

impl Loader {
    pub fn new(db_pool: &DbPool) -> Self {
        Self {
            users: UserLoader::new(UserBatcher::new(db_pool)),
            posts: PostLoader::new(PostBatcher::new(db_pool)),
            challenges: ChallengeLoader::new(ChallengeBatcher::new(db_pool)),
            challenge_type: ChallengeTypeLoader::new(ChallengeTypeBatcher::new(db_pool)),
            responses: ResponseLoader::new(ResponseBatcher::new(db_pool)),
            comments: CommentLoader::new(CommentBatcher::new(db_pool)),
        }
    }
}
