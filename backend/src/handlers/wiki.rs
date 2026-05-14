use crate::handlers::auth::Claims;
use crate::helpers::api::extract_jwt;
use crate::models::app::AppState;
use crate::models::wiki::{CreateWikiArticle, CreateWikiArticleRequest, CreateWikiArticleResponse, WikiType, WikiTypeResponse, Wiki, WikiResponse};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

#[get("/wiki_types")]
pub async fn get_wiki_types(_req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as::<_, WikiType>(
        r#"
        SELECT 
            id, 
            title, 
            created_at, 
            updated_at
        FROM wiki_types
        "#,
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(wiki_types) => {
            let response: Vec<WikiTypeResponse> = wiki_types
                .into_iter()
                .map(|wiki_type| WikiTypeResponse {
                    id: wiki_type.id,
                    title: wiki_type.title,
                    created_at: wiki_type.created_at,
                    updated_at: wiki_type.updated_at,
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": response,
            }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki_types" }))
        }
    }
}

#[post("/wiki")]
pub async fn create_wiki_article(
    req: HttpRequest,
    params: web::Json<CreateWikiArticleRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let token = match extract_jwt(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(
                serde_json::json!({ "error": "Missing auth_token cookie or Authorization header" }),
            );
        }
    };
    let author_id = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims.sub,
        Err(e) => {
            eprintln!("JWT decode error: {}", e);
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "error": "Invalid or expired token" }));
        }
    };

    let wiki_article_create_result = sqlx::query_as::<_, CreateWikiArticle>(
        r#"
        INSERT INTO wiki_articles (title, content, wiki_type_id, created_by, last_edited_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id, 
            title, 
            wiki_type_id, 
            created_by, 
            last_edited_by,
            is_confirmed,
            created_at, 
            updated_at
        "#,
    )
    .bind(&params.title)
    .bind(&params.content)
    .bind(&params.wiki_type_id)
    .bind(author_id)
    .bind(author_id)
    .fetch_one(&state.pool)
    .await;

    let wiki_type = sqlx::query_as::<_, WikiTypeResponse>(
        r#"
        SELECT id, title, created_at, updated_at 
        FROM wiki_types 
        WHERE id = $1
        "#,
    )
    .bind(&params.wiki_type_id)
    .fetch_one(&state.pool)
    .await;

    match (wiki_article_create_result, wiki_type) {
        (Ok(article), Ok(wiki_type)) => {

            let response = CreateWikiArticleResponse {
                id: article.id,
                title: article.title,
                wiki_type: wiki_type,
                created_by: article.created_by,
                last_edited_by: article.last_edited_by,
                is_confirmed: article.is_confirmed,
                created_at: article.created_at,
                updated_at: article.updated_at,
            };
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": response,
            }))
        }
        (Err(e), _) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create wiki article." }))
        }
        (_, Err(e)) => {
            eprintln!("Count error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki_type" }))
        }
    }
}

#[get("/wiki/{id}")]
pub async fn get_wiki_article(
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();

    let wiki_article = sqlx::query_as::<_, Wiki>(
        r#"
        SELECT 
            id, 
            title, 
            content,
            wiki_type_id, 
            created_by, 
            last_edited_by,
            is_confirmed,
            created_at, 
            updated_at
        FROM wiki_articles
        WHERE id = $1
        "#,
    )
    .bind(article_id)
    .fetch_optional(&state.pool)
    .await;

    match wiki_article {
        Ok(Some(article)) => {
            let wiki_type = sqlx::query_as::<_, WikiTypeResponse>(
                r#"
                SELECT id, title, created_at, updated_at 
                FROM wiki_types 
                WHERE id = $1
                "#,
            )
            .bind(article.wiki_type_id)
            .fetch_one(&state.pool)
            .await;

            match wiki_type {
                Ok(w_type) => {
                    let response = WikiResponse {
                        id: article.id,
                        title: article.title,
                        content: article.content,
                        wiki_type: w_type,
                        created_by: article.created_by,
                        last_edited_by: article.last_edited_by,
                        is_confirmed: article.is_confirmed,
                        created_at: article.created_at,
                        updated_at: article.updated_at,
                    };
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                        "data": response,
                    }))
                }
                Err(e) => {
                    eprintln!("Database error fetching wiki type: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to fetch wiki type." }))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({ "error": "Article not found" })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki article." }))
        }
    }
}
