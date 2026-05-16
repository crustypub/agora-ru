use crate::handlers::auth::Claims;
use crate::helpers::api::extract_jwt;
use crate::models::app::AppState;
use crate::models::wiki::{
    CreateWikiArticle, CreateWikiArticleRequest, CreateWikiArticleResponse, WikIArticlesParams,
    Wiki, WikiListItem, WikiType, WikiTypeResponse,
};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

#[get("/wiki_articles")]
pub async fn get_wiki_articles(
    _req: HttpRequest,
    params: web::Query<WikIArticlesParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();
    let wiki_type_filter = params.wiki_type;

    let articles_result = sqlx::query_as::<_, WikiListItem>(
        r#"
        SELECT
            wa.id,
            wa.title,
            wa.content,
            wa.wiki_type_id,
            wa.is_confirmed,
            wa.stars_count,
            wa.created_at,
            wa.updated_at,
            json_build_object(
                'id',         u1.id,
                'username',   u1.username,
                'first_name', u1.first_name,
                'last_name',  u1.last_name,
                'avatar_url', u1.avatar_url
            ) AS created_by,
            json_build_object(
                'id',         u2.id,
                'username',   u2.username,
                'first_name', u2.first_name,
                'last_name',  u2.last_name,
                'avatar_url', u2.avatar_url
            ) AS last_edited_by,
            json_build_object(
                'id', wt.id,
                'title', wt.title,
                'created_at', wt.created_at,
                'updated_at', wt.updated_at
            ) as wiki_type
        FROM wiki_articles wa
        JOIN wiki_types  wt ON wa.wiki_type_id  = wt.id
        JOIN users       u1 ON wa.created_by     = u1.id
        JOIN users       u2 ON wa.last_edited_by = u2.id
        WHERE ($1::int IS NULL OR wa.wiki_type_id = $1)
        ORDER BY wa.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(wiki_type_filter)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM wiki_articles WHERE ($1::int IS NULL OR wiki_type_id = $1)",
    )
    .bind(wiki_type_filter)
    .fetch_one(&state.pool)
    .await;

    match (articles_result, count_result) {
        (Ok(rows), Ok(total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

            let data: Vec<WikiListItem> = rows
                .into_iter()
                .map(|row| WikiListItem {
                    id: row.id,
                    title: row.title,
                    wiki_type: row.wiki_type,
                    created_by: row.created_by,
                    last_edited_by: row.last_edited_by,
                    is_confirmed: row.is_confirmed,
                    stars_count: row.stars_count,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": data,
                "meta": {
                    "current_page": params.page,
                    "per_page": limit,
                    "total_count": total_count,
                    "total_pages": total_pages,
                    "has_next": params.page < total_pages,
                    "has_previous": params.page > 1
                }
            }))
        }
        (Err(e), _) => {
            eprintln!("Database error fetching wiki articles: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki articles" }))
        }
        (_, Err(e)) => {
            eprintln!("Count error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki articles count" }))
        }
    }
}

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
pub async fn get_wiki_article(path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let article_id = path.into_inner();

    let wiki_article = sqlx::query_as::<_, Wiki>(
        r#"
        SELECT
            wa.id,
            wa.title,
            wa.content,
            wa.wiki_type_id,
            wa.is_confirmed,
            wa.stars_count,
            wa.created_at,
            wa.updated_at,
            json_build_object(
                'id', cu.id,
                'username', cu.username,
                'first_name', cu.first_name,
                'last_name', cu.last_name,
                'avatar_url', cu.avatar_url
            ) as created_by,
            json_build_object(
                'id', uu.id,
                'username', uu.username,
                'first_name', uu.first_name,
                'last_name', uu.last_name,
                'avatar_url', uu.avatar_url
            ) as last_edited_by,
            json_build_object(
                'id', wt.id,
                'title', wt.title,
                'created_at', wt.created_at,
                'updated_at', wt.updated_at
            ) as wiki_type
        FROM wiki_articles wa
        JOIN wiki_types wt ON wa.wiki_type_id = wt.id
        JOIN users cu ON wa.created_by = cu.id
        JOIN users uu ON wa.last_edited_by = uu.id
        WHERE wa.id = $1
        "#,
    )
    .bind(article_id)
    .fetch_optional(&state.pool)
    .await;

    match wiki_article {
        Ok(Some(article)) => {
            let response = Wiki {
                id: article.id,
                title: article.title,
                content: article.content,
                wiki_type: article.wiki_type,
                created_by: article.created_by,
                last_edited_by: article.last_edited_by,
                is_confirmed: article.is_confirmed,
                stars_count: article.stars_count,
                created_at: article.created_at,
                updated_at: article.updated_at,
            };
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": response,
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Article not found" }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki article." }))
        }
    }
}
