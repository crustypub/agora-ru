use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client as S3Client;
use std::env;

pub async fn setup_s3_client() -> S3Client {
    let endpoint = env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set");
    
    let access_key = env::var("S3_ACCESS_KEY").expect("S3_ACCESS_KEY must be set");
    let secret_key = env::var("S3_SECRET_KEY").expect("S3_SECRET_KEY must be set");
    let region = env::var("S3_REGION").expect("S3_REGION must be set");

    let credentials = Credentials::new(
        access_key,
        secret_key,
        None,
        None,
        "StaticCredentials"
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(aws_config::Region::new(region))
        .endpoint_url(endpoint)
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    let client = S3Client::from_conf(s3_config);

    // Verify bucket exists, if not, panic
    let bucket_name = String::from("avatars");
    if let Err(e) = client.head_bucket().bucket(&bucket_name).send().await {
        panic!("S3 bucket '{}' is not available or does not exist: {:?}", bucket_name, e);
    } else {
        println!("Connected to S3 successfully. Bucket '{}' exists.", bucket_name);
    }

    client
}
