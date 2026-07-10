use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::comments::{
        create_comment as db_create_comment, delete_comment as db_delete_comment,
        get_comment_author, get_comments_paginated, update_comment as db_update_comment,
    },
    helpers::api::AuthenticatedUser,
    models::{
        app::AppState,
        comment::{CommentParams, CreateCommentRequest, UpdateCommentRequest},
    },
};

#[get("/comments")]
pub async fn get_comments(
    params: web::Query<CommentParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();
    let entity_type = &params.entity_type;
    let entity_id = params.entity_id;

    let result = get_comments_paginated(
        &state.pool,
        entity_type,
        entity_id,
        limit,
        offset,
    )
    .await;

    match result {
        Ok((rows, total_count)) => {
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
        Err(e) => {
            eprintln!("Database error fetching comments: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to fetch comments" }))
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

    let result = db_create_comment(
        &state.pool,
        author_id,
        &params.entity_type,
        params.entity_id,
        &params.content,
    )
    .await;

    match result {
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
    let ownership_check = get_comment_author(&state.pool, comment_id).await;

    match ownership_check {
        Ok(Some(owner)) if owner == author_id => {
            let update_result = db_update_comment(&state.pool, comment_id, &params.content).await;

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
    let ownership_check = get_comment_author(&state.pool, comment_id).await;

    match ownership_check {
        Ok(Some(owner)) if owner == author_id => {
            let delete_result = db_delete_comment(&state.pool, comment_id).await;

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
