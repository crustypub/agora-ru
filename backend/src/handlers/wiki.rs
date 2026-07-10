use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::wiki::{
        add_wiki_star, check_wiki_article_exists, create_wiki_article as db_create_wiki_article,
        delete_wiki_article as db_delete_wiki_article, get_wiki_article as db_get_wiki_article,
        get_wiki_article_content_and_author, get_wiki_articles_paginated, get_wiki_types as db_get_wiki_types,
        remove_wiki_star, update_wiki_article as db_update_wiki_article,
    },
    helpers::{api::{AuthenticatedUser, MaybeAuthenticatedUser}, images},
    models::{
        app::AppState,
        wiki::{
            CreateWikiArticleResponse, CreateWikiArticleRequest,
            UpdateWikiArticleRequest, WikIArticlesParams, Wiki, WikiListItem, WikiTypeResponse,
        },
    },
};

#[get("/wiki_articles")]
pub async fn get_wiki_articles(
    user: MaybeAuthenticatedUser,
    params: web::Query<WikIArticlesParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let current_user_id = user.id;

    let result = get_wiki_articles_paginated(&state.pool, current_user_id, &params).await;

    match result {
        Ok((articles, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

            let data: Vec<WikiListItem> = articles
                .into_iter()
                .map(|row| WikiListItem {
                    id: row.id,
                    title: row.title,
                    wiki_type: row.wiki_type,
                    created_by: row.created_by,
                    last_edited_by: row.last_edited_by,
                    is_confirmed: row.is_confirmed,
                    comment_count: row.comment_count,
                    stars_count: row.stars_count,
                    is_starred: row.is_starred,
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
        Err(e) => {
            eprintln!("Database error fetching wiki articles: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch wiki articles" }))
        }
    }
}

#[get("/wiki_types")]
pub async fn get_wiki_types(state: web::Data<AppState>) -> impl Responder {
    let result = db_get_wiki_types(&state.pool).await;

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
    user: AuthenticatedUser,
    params: web::Json<CreateWikiArticleRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }),
        );
    }

    let result = db_create_wiki_article(
        &state.pool,
        author_id,
        &params.title,
        &params.content,
        params.wiki_type_id,
    )
    .await;

    match result {
        Ok((article, wiki_type)) => {
            let response = CreateWikiArticleResponse {
                id: article.id,
                title: article.title,
                wiki_type,
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
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create wiki article." }))
        }
    }
}

#[get("/wiki/{id}")]
pub async fn get_wiki_article(
    user: MaybeAuthenticatedUser,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();
    let current_user_id = user.id;

    let result = db_get_wiki_article(&state.pool, article_id, current_user_id).await;

    match result {
        Ok(Some(article)) => {
            let response = Wiki {
                id: article.id,
                title: article.title,
                content: article.content,
                wiki_type: article.wiki_type,
                created_by: article.created_by,
                last_edited_by: article.last_edited_by,
                is_confirmed: article.is_confirmed,
                comment_count: article.comment_count,
                stars_count: article.stars_count,
                is_starred: article.is_starred,
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

#[patch("/wiki/{id}/star")]
pub async fn add_star_to_wiki(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();
    let author_id = user.id;

    match check_wiki_article_exists(&state.pool, article_id).await {
        Ok(true) => {
            let result = add_wiki_star(&state.pool, article_id, author_id).await;

            match result {
                Ok(_article) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                })),
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to add star to wiki article." }))
                }
            }
        }
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Wiki article not found" }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/wiki/{id}/star")]
pub async fn remove_star_from_wiki(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();
    let author_id = user.id;

    match check_wiki_article_exists(&state.pool, article_id).await {
        Ok(true) => {
            let result = remove_wiki_star(&state.pool, article_id, author_id).await;

            match result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                })),
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({ "error": "Failed to remove star from wiki article." }),
                    )
                }
            }
        }
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "Wiki article not found" }))
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[patch("/wiki/{id}")]
pub async fn update_wiki_article(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<UpdateWikiArticleRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();
    let author_id = user.id;

    if let Err(errors) = body.validate() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }),
        );
    }

    // 1. Fetch old content before update (only if new content is provided)
    let mut old_content = None;
    if body.content.is_some() {
        let old_content_res = get_wiki_article_content_and_author(&state.pool, article_id).await;

        match old_content_res {
            Ok(Some((content, created_by))) => {
                if created_by != author_id {
                    return HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "Article not found or you are not the author"
                    }));
                }
                old_content = Some(content);
            }
            Ok(None) => {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Article not found or you are not the author"
                }));
            }
            Err(e) => {
                eprintln!("DB error fetching old wiki article: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to update wiki article"
                }));
            }
        }
    }

    let result = db_update_wiki_article(
        &state.pool,
        article_id,
        author_id,
        body.title.as_deref(),
        body.content.as_deref(),
        body.wiki_type_id,
    )
    .await;

    match result {
        Ok(Some(article)) => {
            // Clean up S3 images if content changed
            if let (Some(old), Some(ref new)) = (old_content, &body.content) {
                images::cleanup_unused_images(&old, new, &state.s3_client).await;
            }
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": article,
            }))
        }
        Ok(None) => HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "Article not found or you are not the author" })),
        Err(e) => {
            eprintln!("DB error updating wiki article: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to update wiki article" }))
        }
    }
}

#[delete("/wiki/{id}")]
pub async fn delete_wiki_article(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let article_id = path.into_inner();
    let author_id = user.id;

    let result = db_delete_wiki_article(&state.pool, article_id, author_id).await;

    match result {
        Ok(Some(content)) => {
            // Delete associated images from S3
            images::delete_all_images(&content, &state.s3_client).await;
            HttpResponse::Ok().json(serde_json::json!({ "status": "success" }))
        }
        Ok(None) => HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "Article not found or you are not the author" })),
        Err(e) => {
            eprintln!("DB error deleting wiki article: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to delete wiki article" }))
        }
    }
}

#[post("/wiki/image")]
pub async fn upload_wiki_article_images(
    user: AuthenticatedUser,
    payload: Multipart,
    state: web::Data<AppState>,
) -> impl Responder {
    // 300 MB limit
    let max_size = 300 * 1024 * 1024;

    let bytes = match images::read_first_file(payload, max_size).await {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    };

    // Resize to 600x600 thumbnail
    let webp_bytes = match images::resize_and_encode_webp(&bytes, 600, 600) {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    };

    let bucket_name = std::env::var("MINIO_BUCKET_WIKI_MEDIA")
        .expect("MINIO_BUCKET_WIKI_MEDIA must be set");
    // Generate unique key based on user_id and new UUID
    let key = format!("{}_{}.webp", user.id, uuid::Uuid::new_v4());

    let public_endpoint = match std::env::var("S3_PUBLIC_URL") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("S3_PUBLIC_URL must be set");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "S3 configuration error"
            }));
        }
    };

    let image_url = format!("{}/{}/{}", public_endpoint, &bucket_name, &key);

    match state
        .s3_client
        .put_object()
        .bucket(&bucket_name)
        .key(&key)
        .body(webp_bytes.into())
        .content_type("image/webp")
        .send()
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "url": image_url
        })),
        Err(e) => {
            eprintln!("Failed to upload wiki image to S3: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to upload image to storage"
            }))
        }
    }
}
