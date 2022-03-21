use regex::Regex;
use serde_json::Value;
use std::io::{Error, ErrorKind};

/// Lambda に GET パラメーターを渡した場合 queryStringParameters に入る。
/// そこから GET パラメーターを取得する
pub fn get_query(event: &Value) -> Result<Value, Error> {
    // println!("{:?}", event);

    // Lambda では queryStringParameters の中に GET パラメーターが入る
    let query_string_parameters = match event.get("queryStringParameters") {
        Some(event) => event.clone(),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "queryStringParameters is not found",
            ))
        }
    };

    // query に GraphQL query を入れるため、ここから取得
    match query_string_parameters.get("query") {
        Some(query) => Ok(query.clone()),
        None => Err(Error::new(ErrorKind::InvalidInput, "query is not found")),
    }
}

/// Lambda に POST パラメーターを渡した場合 bodyParameters に入る。
/// そこから POST パラメーターを取得する
pub fn get_body_query(event: &Value) -> Result<Value, Error> {
    // println!("{:?}", event);

    // Lambda では bodyParameters の中に GET パラメーターが入る
    let body_parameters = match event.get("bodyParameters") {
        Some(event) => event.clone(),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "bodyParameters is not found",
            ))
        }
    };

    // query に GraphQL query を入れるため、ここから取得
    match body_parameters.get("query") {
        Some(query) => Ok(query.clone()),
        None => Err(Error::new(ErrorKind::InvalidInput, "query is not found")),
    }
}

/// 指定のパラメーターがあるかどうかを確認し、あればそのパラメーターを返す
pub fn find_param(event: &Value, param: &str) -> Result<Value, Error> {
    match event.get(param) {
        Some(event) => Ok(event.clone()),
        None => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{} is not found", param),
        )),
    }
}

/// GET で送られてきたものは "" で囲まれてしまうため、 "" を解除する
/// "query { humans(i: 0) { name } }" -> query { humans(i: 0) { name } }
pub fn format_query(query: &Value) -> Result<String, Error> {
    let query = match serde_json::to_string(query) {
        Ok(result) => result,
        Err(_) => return Err(Error::new(ErrorKind::InvalidInput, "query is not found")),
    };
    // let query = query.to_string();
    let re = Regex::new(r#""(.*)""#).unwrap();
    let caps = re.captures(&query).unwrap();
    let query = caps.get(1).map_or("", |m| m.as_str());

    Ok(query.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_query() {
        let event = json!({
            "queryStringParameters": {
                "query": "query { humans(i: 0) { name } }"
            }
        });
        let query = get_query(&event).unwrap();
        let query = format_query(&query).unwrap();
        assert_eq!(query, "query { humans(i: 0) { name } }");
    }

    #[test]
    fn test_find_param() {
        let event = json!({
            "queryStringParameters": {
                "query": "query { humans(i: 0) { name } }"
            }
        });
        let query = get_query(&event).unwrap();
        let query = format_query(&query).unwrap();
        assert_eq!(query, "query { humans(i: 0) { name } }");
    }
}
