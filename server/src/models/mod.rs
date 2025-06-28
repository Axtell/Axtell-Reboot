pub mod challenge;
pub mod challenge_type;
pub mod comment;
pub mod post;
pub mod response;
pub mod user;

use std::{fmt::Display, str::FromStr};

pub use challenge::Challenge;
pub use challenge_type::ChallengeType;
pub use comment::Comment;
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;
pub use post::Post;
pub use response::Response;
pub use user::User;

use crate::schema;

pub enum SchemaModel {
    Challenge(Challenge),
    Comment(Comment),
    ChallengeType(ChallengeType),
    Response(Response),
    User(User),
}

impl From<Challenge> for SchemaModel {
    fn from(value: Challenge) -> Self {
        Self::Challenge(value)
    }
}

impl From<Comment> for SchemaModel {
    fn from(value: Comment) -> Self {
        Self::Comment(value)
    }
}

impl From<ChallengeType> for SchemaModel {
    fn from(value: ChallengeType) -> Self {
        Self::ChallengeType(value)
    }
}

impl From<Response> for SchemaModel {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}

impl From<User> for SchemaModel {
    fn from(value: User) -> Self {
        Self::User(value)
    }
}

pub enum SchemaTable {
    Challenges,
    Comments(schema::comments::table),
    ChallengeTypes(schema::challenge_types::table),
    Responses,
    Users(schema::users::table),
}

impl SchemaTable {
    pub async fn find(
        &self,
        primary_key: i32,
        cnx: &mut AsyncPgConnection,
    ) -> QueryResult<SchemaModel> {
        use diesel::QueryDsl;
        use diesel_async::RunQueryDsl;

        Ok(match self {
            Self::Challenges => Challenge::find(cnx, primary_key).await?.into(),
            Self::Responses => Response::find(cnx, primary_key).await?.into(),
            Self::Comments(t) => t.find(primary_key).first::<Comment>(cnx).await?.into(),
            Self::ChallengeTypes(t) => t
                .find(primary_key as i16)
                .first::<ChallengeType>(cnx)
                .await?
                .into(),
            Self::Users(t) => t.find(primary_key).first::<User>(cnx).await?.into(),
        })
    }
}

#[derive(Debug)]
pub struct InvalidTableNameError(String);

impl Display for InvalidTableNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid table name: {}", self.0)
    }
}

impl std::error::Error for InvalidTableNameError {}

impl FromStr for SchemaTable {
    type Err = InvalidTableNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "challenges" => Ok(Self::Challenges),
            "responses" => Ok(Self::Responses),
            "challenge_types" => Ok(Self::ChallengeTypes(schema::challenge_types::table)),
            "comments" => Ok(Self::Comments(schema::comments::table)),
            "users" => Ok(Self::Users(schema::users::table)),
            _ => Err(InvalidTableNameError(s.to_owned())),
        }
    }
}
