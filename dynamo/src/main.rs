use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;
use dynamo::Client;
use futures_util::TryStreamExt;

pub mod dynamo;

const TABLE_NAME: &str = "test_table";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new().await;
    let _ = client.create_table(TABLE_NAME, "id").await?;
    for i in 1..1100 {
        let mut map = HashMap::new();
        map.insert("id".to_string(), AttributeValue::S(i.to_string()));
        map.insert("value".to_string(), AttributeValue::S(i.to_string()));
        let _ = client.create(TABLE_NAME, map).await?;
    }
    let mut stream = client.get_stream(TABLE_NAME, None, None);
    while let Some(item) = stream.try_next().await? {
        println!("{:?}", item);
    }
    let _ = client.delete_table(TABLE_NAME).await?;
    Ok(())
}