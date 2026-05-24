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