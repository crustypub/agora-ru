use actix_web::HttpRequest;

/// Извлекает JWT из запроса.
/// Порядок приоритета:
///   1. `Authorization: Bearer <token>` — для API-клиентов (Postman, curl и т.д.)
///   2. Cookie `auth_token`             — для браузерных запросов (HttpOnly, JS не может читать)
pub fn extract_jwt(req: &HttpRequest) -> Option<String> {
    // 1. Authorization: Bearer <token> (приоритет — не ломает API-клиентов)
    if let Some(token) = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_string())
    {
        return Some(token);
    }

    // 2. Cookie auth_token (для браузерных SSR/CSR запросов)
    if let Some(cookie_header) = req.headers().get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for part in cookie_str.split(';') {
                let mut kv = part.trim().splitn(2, '=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    if key.trim() == "auth_token" {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Экранирует спецсимволы ILIKE-паттерна (`%`, `_`, `\`),
/// чтобы пользовательский ввод не ломал поисковую логику.
pub fn escape_like_pattern(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('%', "\\%")
     .replace('_', "\\_")
}

use actix_web::dev::Payload;
use actix_web::{FromRequest, error::InternalError, HttpResponse};
use std::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;
use crate::models::auth::Claims;
use crate::models::app::AppState;

pub struct AuthenticatedUser {
    pub id: Uuid,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = match extract_jwt(req) {
            Some(t) => t,
            None => {
                return ready(Err(InternalError::from_response(
                    "Missing token",
                    HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Missing authentication token" }))
                ).into()));
            }
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => {
                return ready(Err(InternalError::from_response(
                    "AppState not found",
                    HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Internal server error" }))
                ).into()));
            }
        };

        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => ready(Ok(AuthenticatedUser { id: data.claims.sub })),
            Err(_) => ready(Err(InternalError::from_response(
                "Invalid token",
                HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid or expired token" }))
            ).into())),
        }
    }
}

pub struct MaybeAuthenticatedUser {
    pub id: Option<Uuid>,
}

impl FromRequest for MaybeAuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = match extract_jwt(req) {
            Some(t) => t,
            None => return ready(Ok(MaybeAuthenticatedUser { id: None })),
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return ready(Ok(MaybeAuthenticatedUser { id: None })),
        };

        match decode::<Claims>(
            &token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => ready(Ok(MaybeAuthenticatedUser { id: Some(data.claims.sub) })),
            Err(_) => ready(Ok(MaybeAuthenticatedUser { id: None })),
        }
    }
}