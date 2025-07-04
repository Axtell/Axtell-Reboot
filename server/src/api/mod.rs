use std::{fmt::Display, str::FromStr};

use crate::{
    api::{
        challenge::{ChallengeConnection, ChallengeConnectionEdge},
        post::PostValue,
        relay::{build_connection, relay_connection_closure_args},
    },
    db::{DB_POOL, DbPool, Loader, Repository},
    models::{self, SchemaModel, SchemaTable},
};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use juniper::{
    DefaultScalarValue, EmptyMutation, EmptySubscription, FieldResult, GraphQLScalar, ID,
    InputValue, ParseScalarResult, ParseScalarValue, RootNode, ScalarToken, ScalarValue, Value,
    graphql_interface, graphql_object,
};

pub mod challenge;
pub mod comment;
pub mod post;
pub mod response;
pub mod user;

pub mod relay;

pub use challenge::{Challenge, ChallengeType};
pub use comment::Comment;
pub use post::Post;
pub use response::Response;
pub use user::User;

#[derive(GraphQLScalar, Clone, Debug, PartialEq, Eq)]
#[graphql(with = Self)]
pub struct NodeId(pub String, pub i32);

impl NodeId {
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        BASE64_URL_SAFE
            .encode(format!("{}:{}", self.0, self.1))
            .into()
    }

    fn from_input<S: ScalarValue>(input: &InputValue<S>) -> Result<Self, String> {
        input
            .as_string_value()
            .ok_or_else(|| format!("Expected `String`, found {input}"))
            .and_then(|str| {
                let (table, id) = decode_id(&ID::new(str)).map_err(|e| e.message().to_string())?;
                Ok(Self(table, id))
            })
    }

    fn parse_token<S: ScalarValue>(t: ScalarToken<'_>) -> ParseScalarResult<S> {
        <String as ParseScalarValue<S>>::from_str(t)
    }
}

impl<T, U> From<(T, U)> for NodeId
where
    T: Into<String>,
    U: Into<i32>,
{
    fn from(value: (T, U)) -> Self {
        Self(value.0.into(), value.1.into())
    }
}

impl FromStr for NodeId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        decode_id(&ID::new(s))
            .map(Self::from)
            .map_err(|e| e.message().to_string())
    }
}

impl ToString for NodeId {
    fn to_string(&self) -> String {
        self.to_output::<DefaultScalarValue>()
            .as_string_value()
            .unwrap()
            .to_string()
    }
}

#[graphql_interface]
#[graphql(for = [User, Challenge, ChallengeType, Response, Comment, PostValue], context = Context)]
pub struct Node {
    id: NodeId,
}

impl From<SchemaModel> for NodeValue {
    fn from(value: SchemaModel) -> Self {
        match value {
            SchemaModel::Challenge(m) => NodeValueEnum::Challenge(Challenge::from(m)),
            SchemaModel::User(m) => NodeValueEnum::User(User::from(m)),
            SchemaModel::ChallengeType(m) => NodeValueEnum::ChallengeType(ChallengeType::from(m)),
            SchemaModel::Comment(m) => NodeValueEnum::Comment(Comment::from(m)),
            SchemaModel::Response(m) => NodeValueEnum::Response(Response::from(m)),
        }
    }
}

pub struct Context {
    pub loader: Loader,
    pub db: DbPool,
}

impl Context {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            loader: Loader::new(&DB_POOL),
            db: DB_POOL.clone(),
        })
    }
}

impl juniper::Context for Context {}

#[derive(Debug)]
pub struct IDFormatError(String);

impl Display for IDFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid ID: {}", self.0)
    }
}

impl std::error::Error for IDFormatError {}

pub struct Query;

#[graphql_object]
#[graphql(context = Context)]
impl Query {
    fn api_version() -> &'static str {
        "0.1"
    }

    async fn node<'c>(id: NodeId, ctx: &'c Context) -> FieldResult<Option<NodeValue>> {
        let NodeId(table_name, db_id) = id;
        let table = SchemaTable::from_str(&table_name)?;
        let mut cnx = ctx.db.get().await?;
        let model = table.find(db_id, &mut cnx).await;
        Ok(model.ok().map(NodeValue::from))
    }

    async fn newest_challenges<'c>(
        ctx: &'c Context,
        first: Option<i32>,
        after: Option<NodeId>,
        last: Option<i32>,
        before: Option<NodeId>,
    ) -> FieldResult<ChallengeConnection> {
        let (after, before, limit) = relay_connection_closure_args(first, after, before);
        let mut cnx = ctx.db.get().await?;
        let nodes = models::Challenge::newest(&mut cnx, after, before, limit).await?;
        let (page_info, edges) = build_connection(
            first,
            last,
            nodes.into_iter().map(Challenge::from).collect(),
            ChallengeConnectionEdge::new,
        )?;
        Ok(ChallengeConnection::new(edges, page_info))
    }
}

fn decode_id(id: &ID) -> FieldResult<(String, i32)> {
    let raw_id = String::from_utf8(BASE64_URL_SAFE.decode(id.as_bytes())?)?;
    let (table_name, db_id_str) = raw_id
        .split_once(":")
        .ok_or(IDFormatError(id.to_string()))?;
    Ok((table_name.to_string(), db_id_str.parse()?))
}

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}
