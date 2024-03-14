use std::collections::HashMap;

use aws_config::Region;
use aws_sdk_dynamodb::{
    operation::{
        create_table::CreateTableOutput, delete_table::DeleteTableOutput, put_item::PutItemOutput,
    },
    types::{
        AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
        ScalarAttributeType,
    },
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{Stream, TryStreamExt};

pub struct Client {
    client: aws_sdk_dynamodb::client::Client,
}

impl Client {
    pub async fn new() -> Self {
        std::env::set_var("AWS_ACCESS_KEY_ID", "xxx");
        std::env::set_var("AWS_SECRET_ACCESS_KEY", "xxx");
        std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");
        let config = aws_config::load_from_env().await;
        let builder = aws_sdk_dynamodb::config::Builder::from(&config)
            .endpoint_url("http://localhost:8000")
            .region(Some(Region::from_static("us-east-1")));
        Self {
            client: aws_sdk_dynamodb::Client::from_conf(builder.build()),
        }
    }

    pub async fn create_table(
        &self,
        table_name: &str,
        key: &str,
    ) -> anyhow::Result<CreateTableOutput> {
        let ad = AttributeDefinition::builder()
            .attribute_name(key)
            .attribute_type(ScalarAttributeType::S)
            .build()?;

        let ks = KeySchemaElement::builder()
            .attribute_name(key)
            .key_type(KeyType::Hash)
            .build()?;

        let pt = ProvisionedThroughput::builder()
            .read_capacity_units(1)
            .write_capacity_units(1)
            .build()?;

        self.client
            .create_table()
            .table_name(table_name)
            .set_key_schema(Some(vec![ks]))
            .set_attribute_definitions(Some(vec![ad]))
            .set_provisioned_throughput(Some(pt))
            .send()
            .await
            .map_err(|e| e.into())
    }

    pub async fn delete_table(&self, table_name: &str) -> anyhow::Result<DeleteTableOutput> {
        self.client
            .delete_table()
            .table_name(table_name)
            .send()
            .await
            .map_err(|e| e.into())
    }

    pub async fn create(
        &self,
        table_name: &str,
        data: HashMap<String, AttributeValue>,
    ) -> anyhow::Result<PutItemOutput> {
        self.client
            .put_item()
            .table_name(table_name)
            .set_item(Some(data))
            .send()
            .await
            .map_err(|e| e.into())
    }

    pub fn get_stream(
        &self,
        table_name: &str,
        filter_expression: Option<String>,
        expression_value: Option<HashMap<String, AttributeValue>>,
    ) -> impl Stream<Item = Result<HashMap<String, AttributeValue>, anyhow::Error>> {
        self.client
            .scan()
            .table_name(table_name)
            .set_filter_expression(filter_expression)
            .set_expression_attribute_values(expression_value)
            .into_paginator()
            .items()
            .send()
            .into_stream_03x()
            .err_into()
    }
}
