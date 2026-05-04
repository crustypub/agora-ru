use crate::models::app::AppState;
use crate::models::wiki::{WikiType, WikiTypeResponse};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

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

    match (result) {
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
