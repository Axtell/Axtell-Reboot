use std::sync::LazyLock;

use anyhow::Result;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use dotenvy::dotenv;

pub mod repo;
pub use repo::{Loader, Repository};

pub type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

fn build_pool() -> Result<DbPool> {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new({
        dotenv()?;
        dotenvy::var("DATABASE_URL")?
    });
    Ok(Pool::builder(config).build()?)
}

pub static DB_POOL: LazyLock<DbPool> = LazyLock::new(|| build_pool().unwrap());
