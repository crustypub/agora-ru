use actix_web::{get, post, web, HttpResponse, Responder};
use validator::Validate;

use crate::{
    db::posts::{
        add_post_rating_dislike, add_post_rating_like, create_post as db_create_post,
        get_posts_paginated, remove_post_rating_dislike, remove_post_rating_like,
    },
    helpers::api::{AuthenticatedUser, MaybeAuthenticatedUser},
    models::{
        app::AppState,
        post::{
            CreatePostRequest, CreatePostResponse, Post, PostParams, PostRatingMode,
            PostRatingOperationType, PostRatingRequest, PostResponse,
        },
    },
};

fn get_posts_anonymous(posts: Vec<Post>, total_count: i64, params: &PostParams) -> HttpResponse {
    let limit = params.limit;
    let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

    let response: Vec<PostResponse> = posts
        .into_iter()
        .map(|post| PostResponse {
            id: post.id,
            author: post.author,
            title: post.title,
            content: post.content,
            rating_plus: post.rating_plus.len(),
            rating_minus: post.rating_minus.len(),
            comments_count: post.comments_count,
            created_at: post.created_at,
            updated_at: post.updated_at,
            is_liked: false,
            is_disliked: false,
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": response,
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

#[get("/post")]
pub async fn get_posts(
    user: MaybeAuthenticatedUser,
    params: web::Query<PostParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();

    let result = get_posts_paginated(&state.pool, limit, offset).await;

    match result {
        Ok((posts, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

            if let Some(author_id) = user.id {
                let response: Vec<PostResponse> = posts
                    .into_iter()
                    .map(|post| {
                        let is_liked = post.rating_plus.contains(&author_id);
                        let is_disliked = post.rating_minus.contains(&author_id);
                        PostResponse {
                            id: post.id,
                            author: post.author,
                            title: post.title,
                            content: post.content,
                            rating_plus: post.rating_plus.len(),
                            rating_minus: post.rating_minus.len(),
                            comments_count: post.comments_count,
                            created_at: post.created_at,
                            updated_at: post.updated_at,
                            is_liked,
                            is_disliked,
                        }
                    })
                    .collect();

                HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "data": response,
                    "meta": {
                        "current_page": params.page,
                        "per_page": limit,
                        "total_count": total_count,
                        "total_pages": total_pages,
                        "has_next": params.page < total_pages,
                        "has_previous": params.page > 1
                    }
                }))
            } else {
                get_posts_anonymous(posts, total_count, &params)
            }
        }
        Err(e) => {
            eprintln!("Database error fetching posts: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch posts" }))
        }
    }
}

#[post("/post")]
pub async fn create_post(
    user: AuthenticatedUser,
    params: web::Json<CreatePostRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    let result = db_create_post(&state.pool, author_id, &params.title, &params.content).await;

    match result {
        Ok(post) => {
            let response = CreatePostResponse {
                id: post.id,
                author: post.author,
                title: post.title,
                content: post.content,
                created_at: post.created_at,
                updated_at: post.updated_at,
            };
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": response,
            }))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create post"
            }))
        }
    }
}

#[post("/post_rating")]
pub async fn post_rating_update(
    user: AuthenticatedUser,
    params: web::Json<PostRatingRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    let db_res = match params.operation_type {
        PostRatingOperationType::Add => match params.mode {
            PostRatingMode::Increment => {
                add_post_rating_like(&state.pool, params.post_id, author_id).await
            }
            PostRatingMode::Decrement => {
                add_post_rating_dislike(&state.pool, params.post_id, author_id).await
            }
        },
        PostRatingOperationType::Remove => match params.mode {
            PostRatingMode::Increment => {
                remove_post_rating_like(&state.pool, params.post_id, author_id).await
            }
            PostRatingMode::Decrement => {
                remove_post_rating_dislike(&state.pool, params.post_id, author_id).await
            }
        },
    };

    match db_res {
        Ok(rows_affected) => {
            if rows_affected == 0 {
                HttpResponse::NotFound().json(serde_json::json!({
                    "status": "error",
                    "error": "Post not found",
                    "post_id": params.post_id.to_string()
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                }))
            }
        }
        Err(e) => {
            eprintln!("Database error updating post rating: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update post rating"
            }))
        }
    }
}
