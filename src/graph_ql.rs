use crate::http::request::format_query;
use juniper::{
    execute, execute_sync, Context, EmptyMutation, EmptySubscription, GraphQLType,
    GraphQLTypeAsync, RootNode, ScalarValue, Variables,
};
use ncms_core::errors::http::{ValueError, ValueErrors};
use serde_json::Value;

/// GraphQL 実行
async fn execute_query<'a, QueryT, MutationT, SubscriptionT, S>(
    query: Value,
    schema: &'a RootNode<'a, QueryT, MutationT, SubscriptionT, S>,
) -> Result<Value, Value>
where
    QueryT: GraphQLTypeAsync<S>,
    QueryT::TypeInfo: Sync,
    QueryT::Context: Sync,
    MutationT: GraphQLTypeAsync<S, Context = QueryT::Context>,
    MutationT::TypeInfo: Sync,
    SubscriptionT: GraphQLType<S, Context = QueryT::Context> + Sync,
    SubscriptionT::TypeInfo: Sync,
    S: ScalarValue + Send + Sync,
{
    let query = format_query(query)?;

    // let schema = Schema::new(QueryRoot, EmptyMutation::new(), EmptySubscription::new());
    // println!("query1: {}", query2);
    // let query2 = "query { humans(i: 0) { name } }";
    // println!("query2: {}", query2);

    let result = execute_sync::<S, QueryT, MutationT, SubscriptionT>(
        &query,
        None,
        &schema,
        &Variables::default(),
        &(),
    );

    // println!("{:?}", result);

    let result = match result {
        Ok((result, _)) => serde_json::to_value(result).expect("fatal error"),
        Err(err) => {
            println!("{}", err);

            let msg = err.to_string();
            let field_error = ValueError {
                property: Some(msg),
                ..Default::default()
            };
            let field_errors = ValueErrors::new(vec![field_error]);

            field_errors.to_value()
        }
    };

    Ok(result)
}
