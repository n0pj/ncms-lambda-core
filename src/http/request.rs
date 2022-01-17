use ncms_core::errors;
use ncms_core::errors::http::{ValueError, ValueErrors};
use regex::Regex;
use serde_json::{to_value, Value};

/// Lambda に GET パラメーターを渡した場合 queryStringParameters に入る。
/// そこから GET パラメーターを取得する
pub fn get_query(event: Value) -> Result<Value, Value> {
    // println!("{:?}", event);

    // Lambda では queryStringParameters の中に GET パラメーターが入る
    let query_string_parameters = match event.get("queryStringParameters") {
        Some(event) => event.clone(),
        None => {
            let field_error = ValueError {
                property: Some(
                    errors::validation::CANNOT_FIND_QUERY_STRING_PARAMETERS
                        .message
                        .to_owned(),
                ),
                ..Default::default()
            };
            let field_errors = ValueErrors::new(vec![field_error]);

            return Err(to_value(field_errors).expect("fatal error"));
        }
    };

    // query に GraphQL query を入れるため、ここから取得
    match query_string_parameters.get("query") {
        Some(query) => Ok(query.clone()),
        None => {
            let field_error = ValueError {
                property: Some(errors::validation::CANNOT_FIND_QUERY.message.to_owned()),
                ..Default::default()
            };
            let field_errors = ValueErrors::new(vec![field_error]);

            return Err(to_value(field_errors).expect("fatal error"));
        }
    }
}

/// GET で送られてきたものは "" で囲まれてしまうため、 "" を解除する
/// "query { humans(i: 0) { name } }" -> query { humans(i: 0) { name } }
pub fn format_query(query: Value) -> Result<String, Value> {
    let query = match serde_json::to_string(&query) {
        Ok(result) => result,
        Err(_) => {
            let field_error = ValueError {
                property: Some(errors::validation::CANNOT_FIND_QUERY.message.to_owned()),
                ..Default::default()
            };
            let field_errors = ValueErrors::new(vec![field_error]);

            return Err(field_errors.to_value());
        }
    };
    // let query = query.to_string();
    let re = Regex::new(r#""(.*)""#).unwrap();
    let caps = re.captures(&query).unwrap();
    let query = caps.get(1).map_or("", |m| m.as_str());

    Ok(query.to_owned())
}
