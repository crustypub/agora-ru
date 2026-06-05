use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client as S3Client;
use std::env;

pub async fn setup_s3_client() -> S3Client {
    let endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://minio:9000".to_string());
    
    // We check S3_ACCESS_KEY first, then fallback to MINIO_ROOT_USER
    let access_key = env::var("S3_ACCESS_KEY")
        .or_else(|_| env::var("MINIO_ROOT_USER"))
        .expect("S3_ACCESS_KEY or MINIO_ROOT_USER must be set");
        
    let secret_key = env::var("S3_SECRET_KEY")
        .or_else(|_| env::var("MINIO_ROOT_PASSWORD"))
        .expect("S3_SECRET_KEY or MINIO_ROOT_PASSWORD must be set");
        
    let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

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
    let bucket_name = env::var("S3_BUCKET_AVATARS").unwrap_or_else(|_| "avatars".to_string());
    if let Err(e) = client.head_bucket().bucket(&bucket_name).send().await {
        panic!("S3 bucket '{}' is not available or does not exist: {:?}", bucket_name, e);
    } else {
        println!("Connected to S3 successfully. Bucket '{}' exists.", bucket_name);
    }

    client
}
