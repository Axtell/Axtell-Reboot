use juniper::{FieldResult, GraphQLObject};

use crate::api::NodeId;

// code adapted from https://github.com/Mego/juniper-relay

/// To return objects inside a connection, they must
/// implement this trait.
pub trait RelayConnectionNode {
    /// Returns the cursor associated with this node.
    fn cursor(&self) -> NodeId;

    /// Returns the type name connections
    /// over these nodes should have in the
    /// API. E.g. `"FooConnection"`.
    fn connection_type_name() -> &'static str;

    /// Returns the type name edges containing
    /// these nodes should have in the API.
    /// E.g. `"FooConnectionEdge"`.
    fn edge_type_name() -> &'static str;
}

pub trait RelayConnectionEdge<N>
where
    N: RelayConnectionNode,
{
    fn node(&self) -> &N;
    fn cursor(&self) -> NodeId;
}

#[derive(Debug, Default, GraphQLObject)]
#[graphql(name = "PageInfo")]
#[doc(hidden)]
pub struct RelayConnectionPageInfo {
    has_previous_page: bool,
    has_next_page: bool,
    start_cursor: Option<NodeId>,
    end_cursor: Option<NodeId>,
}

fn check_ge_zero(val: i32) -> Result<i32, &'static str> {
    if val < 0 {
        Err("Pagination argument must be positive")
    } else {
        Ok(val)
    }
}

pub fn relay_connection_closure_args(
    first: Option<i32>,
    after: Option<NodeId>,
    before: Option<NodeId>,
) -> (Option<i32>, Option<i32>, Option<i64>) {
    (
        after.map(|n| n.1),
        before.map(|n| n.1),
        first.map(|l| (l + 1) as i64),
    )
}

pub fn build_connection<N, E, B>(
    first: Option<i32>,
    last: Option<i32>,
    nodes: Vec<N>,
    build_edge: B,
) -> FieldResult<(RelayConnectionPageInfo, Vec<E>)>
where
    N: RelayConnectionNode,
    E: RelayConnectionEdge<N>,
    B: Fn(N) -> E,
{
    let edges_len: i32 = nodes.len().try_into()?;

    let first = first.map(check_ge_zero).transpose()?;
    let last = last.map(check_ge_zero).transpose()?;

    let has_previous_page = if let Some(last) = last {
        edges_len > last
    } else {
        false
    };
    let has_next_page = if let Some(first) = first {
        edges_len > first
    } else {
        false
    };

    let first = first.unwrap_or(edges_len);
    let last = last.unwrap_or(edges_len);

    let len_after_take = i32::min(edges_len, first);
    let skip = i32::max(0, len_after_take - last);

    let edges: Vec<E> = nodes
        .into_iter()
        .take(first.try_into()?)
        .skip(skip.try_into()?)
        .map(build_edge)
        .collect();

    Ok((
        RelayConnectionPageInfo {
            has_previous_page,
            has_next_page,
            start_cursor: edges.first().map(|edge| edge.cursor().clone()),
            end_cursor: edges.last().map(|edge| edge.cursor().clone()),
        },
        edges,
    ))
}

#[macro_export]
macro_rules! relayify {
    ($node:ident, $(($foreign:ident, $get_nodes:expr)),+) => {
        paste::paste! {
            impl crate::api::relay::RelayConnectionNode for $node {
                fn cursor(&self) -> NodeId {
                    self.id()
                }

                fn connection_type_name() -> &'static str {
                    concat!(stringify!($node), "Connection")
                }

                fn edge_type_name() -> &'static str {
                    concat!(stringify!($node), "ConnectionEdge")
                }
            }


            pub struct [<$node ConnectionEdge>] {
                node: $node,
            }

            #[graphql_object]
            #[graphql(context = Context)]
            impl [<$node ConnectionEdge>] {
                #[graphql(skip)]
                pub fn new(node: $node) -> Self {
                    Self { node }
                }

                pub fn node(&self) -> &$node {
                    &self.node
                }

                pub fn cursor(&self) -> NodeId {
                    self.node.id()
                }
            }

            impl crate::api::relay::RelayConnectionEdge<$node> for [<$node ConnectionEdge>] {
                fn node(&self) -> &$node {
                    &self.node
                }

                fn cursor(&self) -> NodeId {
                    self.cursor()
                }
            }

            #[derive(juniper::GraphQLObject)]
            #[graphql(context = Context)]
            pub struct [<$node Connection>] {
                edges: Vec<[<$node ConnectionEdge>]>,
                page_info: crate::api::relay::RelayConnectionPageInfo,
            }

            impl [<$node Connection>] {
                pub fn new(
                    edges: Vec<[<$node ConnectionEdge>]>,
                    page_info: crate::api::relay::RelayConnectionPageInfo
                ) -> Self {
                    Self { edges, page_info }
                }

                $(
                    pub async fn [<try_from_ $foreign:lower>]<'c>(
                        foreign_id: i32,
                        ctx: &'c Context,
                        first: Option<i32>,
                        after: Option<NodeId>,
                        last: Option<i32>,
                        before: Option<NodeId>,
                    ) -> FieldResult<Self> {
                        let (after, before, limit) = crate::api::relay::relay_connection_closure_args(first, after, before);
                        let mut cnx = ctx.db.get().await?;
                        let node_getter = $get_nodes;
                        let nodes = node_getter(&mut cnx, foreign_id, after, before, limit).await?;
                        let (page_info, edges) = crate::api::relay::build_connection(
                            first,
                            last,
                            nodes.into_iter().map($node::from).collect(),
                            [<$node ConnectionEdge>]::new,
                        )?;
                        Ok(Self::new(edges, page_info))
                    }
                )+
            }
        }
    };
}
