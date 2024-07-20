use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnValue},
    Client as DynamoDbClient, Error as OtherError,
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

    run(ServiceBuilder::new()
        .layer(
            CorsLayer::new()
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_origin(AllowOrigin::exact("https://seankrail.dev".parse().unwrap())),
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
    info!("Received request: {:?}", event);

    let path_parameters = event.path_parameters();
    let counter_id = path_parameters.first("counter").unwrap();

    let resp: String;
    if event.method() == Method::GET {
        resp = get_count(ddb_client, table_name, counter_id).await?;
    } else if event.method() == Method::POST {
        resp = increment_counter(ddb_client, table_name, counter_id).await?;
    } else {
        let resp = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Unsupported operation".into())
            .map_err(Box::new)?;
        return Ok(resp);
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
) -> Result<String, OtherError> {
    let request = ddb_client
        .get_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(counter_id.to_string()))
        .projection_expression("#count")
        .expression_attribute_names("#count", "count");

    info!("Getting item from DynamoDB");

    let resp = request.send().await?;

    info!("Got item from DynamoDB {:?}", resp);

    Ok(resp
        .item()
        .unwrap()
        .get("count")
        .unwrap()
        .as_n()
        .unwrap()
        .to_string())
}

pub async fn increment_counter(
    ddb_client: &DynamoDbClient,
    table_name: &str,
    counter_id: &str,
) -> Result<String, OtherError> {
    let request = ddb_client
        .update_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(counter_id.to_string()))
        .update_expression("ADD #count :incr")
        .expression_attribute_names("#count", "count")
        .expression_attribute_values(":incr", AttributeValue::N(1.to_string()))
        .return_values(ReturnValue::UpdatedNew);

    info!("Updating item in DynamoDB");

    let resp = request.send().await?;

    info!("Updated item in DynamoDB {:?}", resp);

    Ok(resp
        .attributes()
        .unwrap()
        .get("count")
        .unwrap()
        .as_n()
        .unwrap()
        .to_string())
}
