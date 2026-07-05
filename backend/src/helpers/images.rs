use actix_multipart::Multipart;
use futures_util::StreamExt;
use image::ImageFormat;
use std::io::Cursor;

/// Reads the first file from a multipart payload, enforcing a maximum file size limit.
pub async fn read_first_file(mut payload: Multipart, max_size: usize) -> Result<Vec<u8>, &'static str> {
    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let mut bytes = Vec::new();
                while let Some(chunk_result) = field.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if bytes.len() + chunk.len() > max_size {
                                return Err("File size limit exceeded");
                            }
                            bytes.extend_from_slice(&chunk);
                        }
                        Err(_) => return Err("Error reading file chunk"),
                    }
                }
                if bytes.is_empty() {
                    return Err("Uploaded file is empty");
                }
                return Ok(bytes);
            }
            Err(_) => return Err("Failed to parse multipart request"),
        }
    }
    Err("No file uploaded")
}

/// Decodes an image, resizes it as a thumbnail (keeping aspect ratio), and encodes it to WebP format.
pub fn resize_and_encode_webp(bytes: &[u8], width: u32, height: u32) -> Result<Vec<u8>, &'static str> {
    let img = image::load_from_memory(bytes)
        .map_err(|_| "Invalid image format. Supported formats: JPEG, PNG, WebP, GIF, BMP")?;

    let resized = img.thumbnail(width, height);

    let mut webp_bytes = Vec::new();
    resized.write_to(&mut Cursor::new(&mut webp_bytes), ImageFormat::WebP)
        .map_err(|_| "Failed to encode image to WebP")?;

    Ok(webp_bytes)
}

/// Extracts all S3 object keys for wiki article media from a markdown content.
pub fn extract_wiki_image_keys(content: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let bucket_name = std::env::var("MINIO_BUCKET_WIKI_MEDIA")
        .expect("MINIO_BUCKET_WIKI_MEDIA must be set");
    let pattern = format!("{}/", bucket_name);
    let mut search_idx = 0;
    while let Some(start_idx) = content[search_idx..].find(&pattern) {
        let actual_start = search_idx + start_idx + pattern.len();
        let end_idx = content[actual_start..].find(|c: char| {
            !c.is_alphanumeric() && c != '_' && c != '-' && c != '.'
        }).unwrap_or(content[actual_start..].len());
        
        let key = &content[actual_start..actual_start + end_idx];
        if !key.is_empty() {
            keys.push(key.to_string());
        }
        search_idx = actual_start + end_idx;
    }
    keys
}

/// Deletes images from S3 that were present in `old_content` but are missing in `new_content`.
pub async fn cleanup_unused_images(
    old_content: &str,
    new_content: &str,
    s3_client: &aws_sdk_s3::Client,
) {
    let old_keys = extract_wiki_image_keys(old_content);
    let new_keys = extract_wiki_image_keys(new_content);

    let bucket_name = std::env::var("MINIO_BUCKET_WIKI_MEDIA")
        .expect("MINIO_BUCKET_WIKI_MEDIA must be set");
    for key in old_keys {
        if !new_keys.contains(&key) {
            let _ = s3_client.delete_object()
                .bucket(&bucket_name)
                .key(&key)
                .send()
                .await;
        }
    }
}

/// Deletes all S3 images referenced in the markdown content.
pub async fn delete_all_images(
    content: &str,
    s3_client: &aws_sdk_s3::Client,
) {
    let keys = extract_wiki_image_keys(content);
    let bucket_name = std::env::var("MINIO_BUCKET_WIKI_MEDIA")
        .expect("MINIO_BUCKET_WIKI_MEDIA must be set");
    for key in keys {
        let _ = s3_client.delete_object()
            .bucket(&bucket_name)
            .key(&key)
            .send()
            .await;
    }
}

pub struct MultipartFile {
    pub bytes: Vec<u8>,
    pub filename: String,
    pub content_type: String,
}

/// Reads the first file from a multipart payload, extracting its bytes, original filename, and MIME type.
pub async fn read_multipart_file(
    mut payload: Multipart,
    max_size: usize,
) -> Result<MultipartFile, &'static str> {
    while let Some(item) = payload.next().await {
        match item {
            Ok(mut field) => {
                let content_disposition = field.content_disposition();
                let filename = content_disposition
                    .and_then(|cd| cd.get_filename())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "file".to_string());
                
                let content_type = field
                    .content_type()
                    .map(|mime| mime.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let mut bytes = Vec::new();
                while let Some(chunk_result) = field.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            if bytes.len() + chunk.len() > max_size {
                                return Err("File size limit exceeded");
                            }
                            bytes.extend_from_slice(&chunk);
                        }
                        Err(_) => return Err("Error reading file chunk"),
                    }
                }
                if bytes.is_empty() {
                    return Err("Uploaded file is empty");
                }
                return Ok(MultipartFile {
                    bytes,
                    filename,
                    content_type,
                });
            }
            Err(_) => return Err("Failed to parse multipart request"),
        }
    }
    Err("No file uploaded")
}

/// Compresses a video file using ffmpeg to standard MP4 H.264 format if ffmpeg is available.
pub async fn compress_video(bytes: &[u8]) -> Result<(Vec<u8>, String), String> {
    let temp_dir = std::path::Path::new("temp_uploads");
    if !temp_dir.exists() {
        std::fs::create_dir_all(temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    }

    let id = uuid::Uuid::new_v4();
    let input_path = temp_dir.join(format!("input_{}.mp4", id));
    let output_path = temp_dir.join(format!("output_{}.mp4", id));

    // Write original bytes to input file
    std::fs::write(&input_path, bytes).map_err(|e| format!("Failed to write temp input file: {}", e))?;

    // Run ffmpeg
    let status = tokio::task::spawn_blocking({
        let input_path = input_path.clone();
        let output_path = output_path.clone();
        move || {
            std::process::Command::new("ffmpeg")
                .arg("-y")
                .arg("-i")
                .arg(&input_path)
                .arg("-vcodec")
                .arg("libx264")
                .arg("-crf")
                .arg("28")
                .arg("-preset")
                .arg("fast")
                .arg("-vf")
                .arg("scale='min(1280,iw)':-2")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-acodec")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg(&output_path)
                .output()
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
    .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let _ = std::fs::remove_file(&input_path);
        let _ = std::fs::remove_file(&output_path);
        return Err(format!("ffmpeg process failed: {}", stderr));
    }

    // Read compressed bytes
    let compressed_bytes = std::fs::read(&output_path).map_err(|e| format!("Failed to read temp output file: {}", e))?;

    // Clean up
    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok((compressed_bytes, "video/mp4".to_string()))
}

