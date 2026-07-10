use std::collections::HashMap;

use actix_web::cookie::{time, Cookie, SameSite};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db::users::upsert_tg_user;
use crate::helpers::telegram::verify_tg_hash;
use crate::models::app::AppState;

#[derive(Deserialize, Serialize)]
pub struct TelegramAuthParams {
    pub id: i64,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: i64,
    pub hash: String,
    /// Любые другие поля, которые Telegram может добавить в будущем
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}

#[post("/auth/telegram")]
pub async fn telegram_auth(
    params: web::Json<TelegramAuthParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 1. Проверяем подпись
    if !verify_tg_hash(&params, &state.bot_token) {
        return HttpResponse::Unauthorized().finish();
    }

    // 2. Ищем или создаем пользователя
    let user = match upsert_tg_user(&state.pool, &params).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to upsert user: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 3. Выпускаем JWT
    let secret = &state.jwt_secret;

    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        + 60 * 60 * 24 * 7; // 7 days

    let claims = Claims {
        sub: user.id,
        exp: expiration as usize,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create token: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let cookie_secure = std::env::var("COOKIE_SECURE").map(|v| v == "true").unwrap_or(false);

    let cookie = Cookie::build("auth_token", token)
        .http_only(true)
        .secure(cookie_secure)
        .path("/")
        .max_age(time::Duration::days(7))
        .same_site(SameSite::Lax)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({ "status": "success" }))
}

#[get("/auth/me")]
pub async fn auth_me(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let token = match crate::helpers::api::extract_jwt(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims.sub,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    let user = sqlx::query_as::<_, crate::models::app::Author>(
        "SELECT id, username, first_name, last_name, avatar_url FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await;

    match user {
        Ok(Some(u)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": u
        })),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct AuthSessionRow {
    pub telegram_id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub photo_url: Option<String>,
    pub auth_date: Option<i64>,
    pub status: String,
}

#[get("/auth/telegram/request")]
pub async fn telegram_auth_request(
    state: web::Data<AppState>,
) -> impl Responder {
    let token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // Insert new pending token
    let result = sqlx::query(
        "INSERT INTO auth_tokens (token, status, created_at) VALUES ($1, 'pending', $2)"
    )
    .bind(&token)
    .bind(now)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "bot_username": state.bot_username
        })),
        Err(e) => {
            eprintln!("Failed to create auth token: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct TelegramCheckRequest {
    pub token: String,
}

#[post("/auth/telegram/check")]
pub async fn telegram_auth_check(
    params: web::Json<TelegramCheckRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 1. Clean up old tokens (older than 10 mins)
    let expire_limit = chrono::Utc::now().timestamp() - 600;
    let _ = sqlx::query("DELETE FROM auth_tokens WHERE created_at < $1")
        .bind(expire_limit)
        .execute(&state.pool)
        .await;

    // 2. Query token
    let session = sqlx::query_as::<_, AuthSessionRow>(
        "SELECT telegram_id, first_name, last_name, username, photo_url, auth_date, status 
         FROM auth_tokens 
         WHERE token = $1"
    )
    .bind(&params.token)
    .fetch_optional(&state.pool)
    .await;

    let row = match session {
        Ok(Some(r)) => r,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({ "status": "expired" })),
        Err(e) => {
            eprintln!("Database error checking auth token: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if row.status == "pending" {
        return HttpResponse::Ok().json(serde_json::json!({ "status": "pending" }));
    }

    if row.status == "authenticated" {
        // Upsert user
        let tg_id = match row.telegram_id {
            Some(id) => id,
            None => {
                eprintln!("Session authenticated but has no telegram_id");
                return HttpResponse::InternalServerError().finish();
            }
        };

        let auth_params = TelegramAuthParams {
            id: tg_id,
            first_name: row.first_name,
            last_name: row.last_name,
            username: row.username,
            photo_url: row.photo_url,
            auth_date: row.auth_date.unwrap_or(0),
            hash: String::new(),
            extra: HashMap::new(),
        };

        let user = match upsert_tg_user(&state.pool, &auth_params).await {
            Ok(u) => u,
            Err(e) => {
                eprintln!("Failed to upsert user: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        // Deleting the session token so it's a one-time login
        let _ = sqlx::query("DELETE FROM auth_tokens WHERE token = $1")
            .bind(&params.token)
            .execute(&state.pool)
            .await;


        // Issue JWT
        let secret = &state.jwt_secret;
        let expiration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            + 60 * 60 * 24 * 7; // 7 days

        let claims = Claims {
            sub: user.id,
            exp: expiration as usize,
        };

        let token = match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        ) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to create token: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let cookie_secure = std::env::var("COOKIE_SECURE").map(|v| v == "true").unwrap_or(false);

        let cookie = Cookie::build("auth_token", token)
            .http_only(true)
            .secure(cookie_secure)
            .path("/")
            .max_age(time::Duration::days(7))
            .same_site(SameSite::Lax)
            .finish();

        return HttpResponse::Ok()
            .cookie(cookie)
            .json(serde_json::json!({ "status": "success" }));
    }

    HttpResponse::BadRequest().finish()
}

#[post("/auth/logout")]
pub async fn telegram_logout() -> impl Responder {
    let cookie_secure = std::env::var("COOKIE_SECURE").map(|v| v == "true").unwrap_or(false);

    let cookie = Cookie::build("auth_token", "")
        .http_only(true)
        .secure(cookie_secure)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .same_site(SameSite::Lax)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({ "status": "success" }))
}


