use crate::helpers::api::AuthenticatedUser;
use crate::models::app::AppState;
use crate::models::comment::{CommentParams, CommentResponse, CreateCommentRequest, UpdateCommentRequest, Comment};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

#[get("/comments")]
pub async fn get_comments(
    params: web::Query<CommentParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();
    let entity_type = &params.entity_type;
    let entity_id = params.entity_id;

    let comments_result = sqlx::query_as::<_, CommentResponse>(
        r#"
        SELECT
            c.id,
            c.entity_type,
            c.entity_id,
            c.content,
            c.created_at,
            c.updated_at,
            json_build_object(
                'id',         u.id,
                'username',   u.username,
                'first_name', u.first_name,
                'last_name',  u.last_name,
                'avatar_url', u.avatar_url
            ) AS author
        FROM comments c
        JOIN users u ON c.author = u.id
        WHERE c.entity_type = $1 AND c.entity_id = $2
        ORDER BY c.created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await;

    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM comments WHERE entity_type = $1 AND entity_id = $2",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_one(&state.pool)
    .await;

    match (comments_result, count_result) {
        (Ok(rows), Ok(total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as i64;

            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": rows,
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
            eprintln!("Database error fetching comments: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch comments" }))
        }
        (_, Err(e)) => {
            eprintln!("Count error: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch comments count" }))
        }
    }
}

#[post("/comments")]
pub async fn create_comment(
    user: AuthenticatedUser,
    params: web::Json<CreateCommentRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    let comment_create_result = sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (entity_type, entity_id, author, content)
        VALUES ($1, $2, $3, $4)
        RETURNING id, entity_type, entity_id, author, content, created_at, updated_at
        "#,
    )
    .bind(&params.entity_type)
    .bind(params.entity_id)
    .bind(author_id)
    .bind(&params.content)
    .fetch_one(&state.pool)
    .await;

    match comment_create_result {
        Ok(comment) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": comment,
            }))
        }
        Err(e) => {
            eprintln!("Database error creating comment: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to create comment." }))
        }
    }
}

#[patch("/comments/{id}")]
pub async fn edit_comment(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    params: web::Json<UpdateCommentRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let comment_id = path.into_inner();
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    // Verify ownership
    let ownership_check = sqlx::query_scalar::<_, Uuid>(
        "SELECT author FROM comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_optional(&state.pool)
    .await;

    match ownership_check {
        Ok(Some(owner)) if owner == author_id => {
            let update_result = sqlx::query_as::<_, Comment>(
                r#"
                UPDATE comments
                SET content = $1
                WHERE id = $2
                RETURNING id, entity_type, entity_id, author, content, created_at, updated_at
                "#,
            )
            .bind(&params.content)
            .bind(comment_id)
            .fetch_one(&state.pool)
            .await;

            match update_result {
                Ok(comment) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "data": comment,
                })),
                Err(e) => {
                    eprintln!("Database error updating comment: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to update comment." }))
                }
            }
        }
        Ok(Some(_)) => HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "You do not have permission to edit this comment." })),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Comment not found." })),
        Err(e) => {
            eprintln!("Database error checking comment ownership: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Database error." }))
        }
    }
}

#[delete("/comments/{id}")]
pub async fn delete_comment(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    let comment_id = path.into_inner();
    let author_id = user.id;

    // Verify ownership
    let ownership_check = sqlx::query_scalar::<_, Uuid>(
        "SELECT author FROM comments WHERE id = $1",
    )
    .bind(comment_id)
    .fetch_optional(&state.pool)
    .await;

    match ownership_check {
        Ok(Some(owner)) if owner == author_id => {
            let delete_result = sqlx::query(
                "DELETE FROM comments WHERE id = $1",
            )
            .bind(comment_id)
            .execute(&state.pool)
            .await;

            match delete_result {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                })),
                Err(e) => {
                    eprintln!("Database error deleting comment: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to delete comment." }))
                }
            }
        }
        Ok(Some(_)) => HttpResponse::Forbidden()
            .json(serde_json::json!({ "error": "You do not have permission to delete this comment." })),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Comment not found." })),
        Err(e) => {
            eprintln!("Database error checking comment ownership: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Database error." }))
        }
    }
}
