use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnValue},
    Client as DynamoDbClient,
};
use lambda_http::{
    http::{Method, StatusCode},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};
use lambda_runtime::tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure tracing
    // Required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // Disable printing the name of the module in every log line
        .with_target(false)
        // This needs to be set to false, otherwise ANSI color codes will show up in a confusing manner in CloudWatch logs
        .with_ansi(false)
        // Disabling time is handy because CloudWatch will add the ingestion time
        .without_time()
        .init();

    // Get config from environment
    let aws_sdk_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    // Create the DynamoDB client
    let ddb_client = DynamoDbClient::new(&aws_sdk_config);
    // Get table name from environment
    let table_name = std::env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");
    // Get CORS origin from environment
    let cors_origin = std::env::var("CORS_ORIGIN").expect("Missing CORS_ORIGIN env var");

    run(ServiceBuilder::new()
        .layer(
            CorsLayer::new()
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_origin(AllowOrigin::exact(cors_origin.parse().unwrap())),
            // Uncomment below for development
            // .allow_origin(AllowOrigin::any()),
        )
        .service(service_fn(|event: Request| async {
            handle_request(event, &ddb_client, &table_name).await
        })))
    .await
}

async fn handle_request(
    event: Request,
    ddb_client: &DynamoDbClient,
    table_name: &str,
) -> Result<Response<Body>, Error> {
    info!("Received {} {}", event.method(), event.uri().path());

    let path_parameters = event.path_parameters();
    let counter_id = match path_parameters.first("counter") {
        Some(id) => id,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Missing counter path parameter".into())
                .map_err(Box::new)?);
        }
    };

    let resp: String;
    if event.method() == Method::GET {
        resp = get_count(ddb_client, table_name, counter_id).await?;
    } else if event.method() == Method::POST {
        resp = increment_counter(ddb_client, table_name, counter_id).await?;
    } else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Unsupported operation".into())
            .map_err(Box::new)?);
    }

    //Send back a 200 - success
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .body(resp.into())
        .map_err(Box::new)?;
    Ok(resp)
}

pub async fn get_count(
    ddb_client: &DynamoDbClient,
    table_name: &str,
    counter_id: &str,
) -> Result<String, Error> {
    info!("Getting item from DynamoDB");

    let resp = ddb_client
        .get_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(counter_id.to_string()))
        .projection_expression("#count")
        .expression_attribute_names("#count", "count")
        .send()
        .await?;

    let item = match resp.item() {
        Some(item) => item,
        None => return Ok("0".to_string()),
    };

    let count = item
        .get("count")
        .ok_or("Missing 'count' attribute in DynamoDB item")?
        .as_n()
        .map_err(|_| "DynamoDB 'count' attribute is not a number")?
        .to_string();

    Ok(count)
}

pub async fn increment_counter(
    ddb_client: &DynamoDbClient,
    table_name: &str,
    counter_id: &str,
) -> Result<String, Error> {
    info!("Updating item in DynamoDB");

    let resp = ddb_client
        .update_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(counter_id.to_string()))
        .update_expression("ADD #count :incr")
        .expression_attribute_names("#count", "count")
        .expression_attribute_values(":incr", AttributeValue::N(1.to_string()))
        .return_values(ReturnValue::UpdatedNew)
        .send()
        .await?;

    let count = resp
        .attributes()
        .ok_or("Missing attributes in DynamoDB update response")?
        .get("count")
        .ok_or("Missing 'count' attribute in DynamoDB update response")?
        .as_n()
        .map_err(|_| "DynamoDB 'count' attribute is not a number")?
        .to_string();

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dummy_ddb_client() -> DynamoDbClient {
        let config = aws_sdk_dynamodb::Config::builder()
            .region(aws_sdk_dynamodb::config::Region::new("us-east-1"))
            .credentials_provider(aws_sdk_dynamodb::config::Credentials::new(
                "dummy", "dummy", None, None, "dummy",
            ))
            .endpoint_url("http://localhost:18888")
            .behavior_version(aws_sdk_dynamodb::config::BehaviorVersion::latest())
            .build();
        DynamoDbClient::from_conf(config)
    }

    #[tokio::test]
    async fn test_unsupported_method_returns_400() {
        let ddb_client = make_dummy_ddb_client();
        let event: Request = lambda_http::http::Request::<()>::builder()
            .method(Method::DELETE)
            .uri("https://example.com/count/my-counter")
            .body(Body::Empty)
            .unwrap();

        let response = handle_request(event, &ddb_client, "test-table")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_missing_counter_path_param_returns_400() {
        let ddb_client = make_dummy_ddb_client();
        let event: Request = lambda_http::http::Request::<()>::builder()
            .method(Method::GET)
            .uri("https://example.com/count/")
            .body(Body::Empty)
            .unwrap();

        let response = handle_request(event, &ddb_client, "test-table")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
