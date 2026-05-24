use crate::helpers::api::{AuthenticatedUser, MaybeAuthenticatedUser};
use crate::models::app::AppState;
use crate::models::post::{
    CreatePostResponse, PostParams, PostRatingMode, PostRatingOperationType, PostRatingRequest,
    PostResponse,
};
use crate::models::post::CreatePost;
use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::post::{CreatePostRequest, Post};
use validator::Validate;

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

    let posts_result = sqlx::query_as::<_, Post>(
        r#"
        SELECT
            p.id, p.title, p.content,
            p.rating_plus, p.rating_minus, p.comments_count,
            p.created_at, p.updated_at,
            json_build_object(
                'id', u.id,
                'username', u.username,
                'first_name', u.first_name,
                'last_name', u.last_name,
                'avatar_url', u.avatar_url
            ) as author
        FROM posts p
        JOIN users u ON p.author = u.id
        ORDER BY p.created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    let count_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(&state.pool)
        .await;

    match (posts_result, count_result) {
        (Ok(posts), Ok(total_count)) => {
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
        (Err(e), _) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch posts" }))
        }
        (_, Err(e)) => {
            eprintln!("Count error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch posts count" }))
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

    // 3. Создаём пост, подставляя author из токена
    let result = sqlx::query_as::<_, CreatePost>(
        r#"
        INSERT INTO posts (author, title, content)
        VALUES ($1, $2, $3)
        RETURNING
            id,
            author,
            title,
            content,
            rating_plus,
            rating_minus,
            comments_count,
            created_at,
            updated_at
        "#,
    )
    .bind(author_id)
    .bind(&params.title)
    .bind(&params.content)
    .fetch_one(&state.pool)
    .await;

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

    match params.operation_type {
        PostRatingOperationType::Add => {
            match params.mode {
                PostRatingMode::Increment => {
                    let result = sqlx::query(
                        r#"
                UPDATE posts
                SET
                    rating_plus = array_append(array_remove(rating_plus, $1), $1),
                    rating_minus = array_remove(rating_minus, $1)
                WHERE id = $2
            "#,
                    )
                    .bind(author_id)
                    .bind(params.post_id)
                    .execute(&state.pool)
                    .await;
                    match result {
                        Ok(rows_affected) => {
                            if rows_affected.rows_affected() == 0 {
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
                            eprintln!("Database error: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to update post rating"
                            }))
                        }
                    }
                }
                PostRatingMode::Decrement => {
                    let result = sqlx::query(
                        r#"
                UPDATE posts
                SET
                    rating_minus = array_append(array_remove(rating_minus, $1), $1),
                    rating_plus = array_remove(rating_plus, $1)
                WHERE id = $2
            "#,
                    )
                    .bind(author_id)
                    .bind(params.post_id)
                    .execute(&state.pool)
                    .await;
                    match result {
                        Ok(rows_affected) => {
                            if rows_affected.rows_affected() == 0 {
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
                            eprintln!("Database error: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to update post rating"
                            }))
                        }
                    }
                }
            }
        }

        PostRatingOperationType::Remove => {
            match params.mode {
                PostRatingMode::Increment => {
                    let result = sqlx::query(
                        r#"
                        UPDATE posts
                        SET
                            rating_plus = array_remove(rating_plus, $1)
                        WHERE id = $2
                    "#,
                    )
                    .bind(author_id)
                    .bind(params.post_id)
                    .execute(&state.pool)
                    .await;
                    match result {
                        Ok(rows_affected) => {
                            if rows_affected.rows_affected() == 0 {
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
                            eprintln!("Database error: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to update post rating"
                            }))
                        }
                    }
                }
                PostRatingMode::Decrement => {
                    let result = sqlx::query(
                        r#"
                        UPDATE posts
                        SET
                            rating_minus = array_remove(rating_minus, $1)
                        WHERE id = $2
                    "#,
                    )
                    .bind(author_id)
                    .bind(params.post_id)
                    .execute(&state.pool)
                    .await;
                    match result {
                        Ok(rows_affected) => {
                            if rows_affected.rows_affected() == 0 {
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
                            eprintln!("Database error: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to update post rating"
                            }))
                        }
                    }
                }
            }
        }
    }
}
