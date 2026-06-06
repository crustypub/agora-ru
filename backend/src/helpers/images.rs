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
