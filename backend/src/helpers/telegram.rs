use actix_web::web;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH}; // Добавили для работы со временем

use crate::handlers::auth::TelegramAuthParams;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_tg_hash(params: &web::Json<TelegramAuthParams>, bot_token: &String) -> bool {
    // Проверка актуальности (24 часа)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;

    let day_in_seconds = 24 * 60 * 60; // 86400

    // Проверяем, что данные не старше суток и не из "будущего" (с запасом в 5 мин)
    if params.auth_date > now + 300 || now - params.auth_date > day_in_seconds {
        return false;
    }
    // ----------------------------------------------------

    // 1. Собираем пары key=value для всех полей, кроме hash
    let mut data_parts: Vec<String> = Vec::new();

    data_parts.push(format!("auth_date={}", params.auth_date));
    data_parts.push(format!("first_name={}", params.first_name));
    data_parts.push(format!("id={}", params.id));

    if let Some(ref last_name) = params.last_name {
        data_parts.push(format!("last_name={}", last_name));
    }

    if let Some(ref photo_url) = params.photo_url {
        data_parts.push(format!("photo_url={}", photo_url));
    }

    if let Some(ref username) = params.username {
        data_parts.push(format!("username={}", username));
    }

    for (key, value) in &params.extra {
        let val_str = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        data_parts.push(format!("{}={}", key, val_str));
    }

    // 2. Сортируем по алфавиту и объединяем через \n
    data_parts.sort();
    let data_check_string = data_parts.join("\n");

    // 3. secret_key = SHA-256(bot_token)
    let secret_key = Sha256::digest(bot_token.as_bytes());

    // 4. HMAC-SHA-256(secret_key, data_check_string)
    let mut mac =
        HmacSha256::new_from_slice(&secret_key).expect("HMAC error");
    mac.update(data_check_string.as_bytes());
    let result = mac.finalize().into_bytes();

    // 5. Сравниваем с переданным hash
    let computed_hash = hex::encode(result);

    computed_hash == params.hash
}