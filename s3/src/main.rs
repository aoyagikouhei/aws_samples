use aws_config::Region;
use aws_sdk_s3::{
    config::Credentials,
    operation::{list_objects_v2::ListObjectsV2Output, put_object::PutObjectOutput},
    primitives::ByteStream,
    Client,
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{Stream, TryStreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bucket_name = "my-bucket";
    let credentials_provider = Credentials::new("admin123", "admin123", None, None, "example");
    let config = aws_sdk_s3::Config::builder()
        .behavior_version_latest()
        .credentials_provider(credentials_provider)
        .region(Region::new("ap-northeast-1"))
        .force_path_style(true)
        .endpoint_url("http://localhost:9000")
        .build();
    let client = Client::from_conf(config);
    for i in 1..1100 {
        let _ = put_object(&client, bucket_name, &i.to_string(), i.to_string()).await?;
    }
    let mut stream = get_stream(&client, bucket_name, None);
    while let Some(item) = stream.try_next().await? {
        println!("{}", item.contents().len());
        for line in item.contents() {
            println!("{:?}", line.key());
        }
    }
    Ok(())
}

async fn put_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
    data: String,
) -> anyhow::Result<PutObjectOutput> {
    client
        .put_object()
        .set_bucket(Some(bucket_name.to_owned()))
        .set_key(Some(key.to_owned()))
        .set_body(Some(ByteStream::from(data.into_bytes())))
        .send()
        .await
        .map_err(|e| e.into())
}

fn get_stream(
    client: &Client,
    bucket_name: &str,
    prefix: Option<String>,
) -> impl Stream<Item = anyhow::Result<ListObjectsV2Output>> {
    client
        .list_objects_v2()
        .bucket(bucket_name)
        .set_prefix(prefix)
        .into_paginator()
        .send()
        .into_stream_03x()
        .err_into()
}
