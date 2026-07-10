use std::collections::HashMap;

use actix_web::cookie::{time, Cookie, SameSite};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

use crate::db::auth::{
    clean_expired_tokens, create_pending_token, delete_auth_token, get_author_by_id,
    get_auth_session,
};
use crate::db::users::upsert_tg_user;
use crate::helpers::api::extract_jwt;
use crate::helpers::telegram::verify_tg_hash;
use crate::models::app::AppState;
use crate::models::auth::{Claims, TelegramAuthParams, TelegramCheckRequest};

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
    let token = match extract_jwt(&req) {
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

    let user = get_author_by_id(&state.pool, user_id).await;

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

#[get("/auth/telegram/request")]
pub async fn telegram_auth_request(
    state: web::Data<AppState>,
) -> impl Responder {
    let token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // Insert new pending token
    let result = create_pending_token(&state.pool, &token, now).await;

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

#[post("/auth/telegram/check")]
pub async fn telegram_auth_check(
    params: web::Json<TelegramCheckRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // 1. Clean up old tokens (older than 10 mins)
    let expire_limit = chrono::Utc::now().timestamp() - 600;
    let _ = clean_expired_tokens(&state.pool, expire_limit).await;

    // 2. Query token
    let session = get_auth_session(&state.pool, &params.token).await;

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
        let _ = delete_auth_token(&state.pool, &params.token).await;

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
