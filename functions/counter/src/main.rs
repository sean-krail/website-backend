use aws_sdk_dynamodb::{
    types::{AttributeValue, ReturnValue},
    Client as DynamoDbClient, Error as OtherError,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::env;
use tracing::info;

const COUNTER_NAME: &str = "likes";

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
    let aws_sdk_config = aws_config::load_from_env().await;
    // Create the DynamoDB client
    let ddb_client = DynamoDbClient::new(&aws_sdk_config);
    // Get table name from environment
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    run(service_fn(|event: Request| async {
        handle_request(event, &ddb_client, &table_name).await
    }))
    .await
}

async fn handle_request(
    event: Request,
    ddb_client: &DynamoDbClient,
    table_name: &str,
) -> Result<Response<Body>, Error> {
    info!("Received request: {:?}", event);

    // Insert into the table
    let new_count = increment_counter(ddb_client, table_name).await?;

    //Send back a 200 - success
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(new_count.into())
        .map_err(Box::new)?;
    Ok(resp)
}

pub async fn increment_counter(
    ddb_client: &DynamoDbClient,
    table_name: &str,
) -> Result<String, OtherError> {
    let request = ddb_client
        .update_item()
        .table_name(table_name)
        .key("id", AttributeValue::S(COUNTER_NAME.to_string()))
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
