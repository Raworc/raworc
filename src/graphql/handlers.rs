use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::Extension, http::HeaderMap};

use crate::graphql::schema::{Mutation, Query};

pub type RaworcSchema = Schema<Query, Mutation, EmptySubscription>;

// GraphQL handler
pub async fn graphql_handler(
    schema: Extension<RaworcSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner().data(headers)).await.into()
}
