use crate::helpers::api::{AuthenticatedUser, MaybeAuthenticatedUser};
use crate::models::app::{AppState, SortField};
use crate::models::wiki::{
    CreateWikiArticle, CreateWikiArticleRequest, CreateWikiArticleResponse, CreateWikiStar,
    UpdateWikiArticleRequest, WikIArticlesParams, Wiki, WikiListItem, WikiType, WikiTypeResponse,
};
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse, Responder};
use sqlx::{Error, PgPool};
use uuid::Uuid;
use validator::Validate;

async fn wiki_article_exists(pool: &PgPool, id: Uuid) -> Result<bool, Error> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_articles WHERE id = $1)")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(exists)
}

#[get("/wiki_articles")]
pub async fn get_wiki_articles(
    user: MaybeAuthenticatedUser,
    params: web::Query<WikIArticlesParams>,
    state: web::Data<AppState>,
) -> impl Responder {
    let limit = params.limit;
    let offset = params.offset();

    let current_user_id = user.id;

    // Безопасно: колонка берётся из белого списка трейта SortField, а не из user input
    let sort_col = params.sort_by.as_sql_column();
    let sort_dir = params.sort_order.as_sql();

    // -- Основной запрос --------------------------------------------------
    // Динамическая сортировка через format! безопасна: sort_col и sort_dir
    // берутся только из &'static str enum-а, никакого user input внутри.
    let articles_sql = format!(
        r#"
        SELECT
            wa.id,
            wa.title,
            wa.content,
            wa.wiki_type_id,
            wa.is_confirmed,
            wa.comment_count,
            wa.stars_count,
            wa.created_at,
            wa.updated_at,
            EXISTS(SELECT 1 FROM wiki_stars ws WHERE ws.wiki_id = wa.id AND ws.user_id = $6) AS is_starred,
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
        WHERE
            ($1::int  IS NULL OR wa.wiki_type_id = $1)
            AND ($2::bool IS NULL OR wa.is_confirmed  = $2)
            AND ($3::text IS NULL OR (
                wa.title   ILIKE '%' || $3 || '%' ESCAPE '\'
                OR wa.content ILIKE '%' || $3 || '%' ESCAPE '\'
            ))
        ORDER BY {sort_col} {sort_dir}
        LIMIT $4 OFFSET $5
        "#
    );

    // -- Запрос COUNT (те же фильтры) ------------------------------------
    let count_sql = r#"
        SELECT COUNT(*)
        FROM wiki_articles wa
        WHERE
            ($1::int  IS NULL OR wa.wiki_type_id = $1)
            AND ($2::bool IS NULL OR wa.is_confirmed  = $2)
            AND ($3::text IS NULL OR (
                wa.title   ILIKE '%' || $3 || '%' ESCAPE '\'
                OR wa.content ILIKE '%' || $3 || '%' ESCAPE '\'
            ))
    "#;

    let search = params.search.as_deref().filter(|s| !s.is_empty()).map(crate::helpers::api::escape_like_pattern);

    let articles_result = sqlx::query_as::<_, WikiListItem>(&articles_sql)
        .bind(params.wiki_type)    // $1 — wiki_type
        .bind(params.is_confirmed) // $2 — is_confirmed
        .bind(&search)             // $3 — search
        .bind(limit)               // $4 — limit
        .bind(offset)              // $5 — offset
        .bind(current_user_id)     // $6 — current_user_id (для is_starred)
        .fetch_all(&state.pool)
        .await;

    let count_result = sqlx::query_scalar::<_, i64>(count_sql)
        .bind(params.wiki_type)
        .bind(params.is_confirmed)
        .bind(&search)
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
    user: AuthenticatedUser,
    params: web::Json<CreateWikiArticleRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let author_id = user.id;

    if let Err(errors) = params.validate() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

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
pub async fn get_wiki_article(user: MaybeAuthenticatedUser, path: web::Path<Uuid>, state: web::Data<AppState>) -> impl Responder {
    let article_id = path.into_inner();

    let current_user_id = user.id;

    let wiki_article = sqlx::query_as::<_, Wiki>(
        r#"
        SELECT
            wa.id,
            wa.title,
            wa.content,
            wa.wiki_type_id,
            wa.is_confirmed,
            wa.comment_count,
            wa.stars_count,
            wa.created_at,
            wa.updated_at,
            EXISTS(SELECT 1 FROM wiki_stars ws WHERE ws.wiki_id = wa.id AND ws.user_id = $2) AS is_starred,
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
    .bind(current_user_id)
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

    match wiki_article_exists(&state.pool, article_id).await {
        Ok(true) => {
            let wiki_article_create_result = sqlx::query_as::<_, CreateWikiStar>(
                r#"
                INSERT INTO wiki_stars (wiki_id, user_id)
                VALUES ($1, $2)
                RETURNING
                    id,
                    wiki_id,
                    user_id,
                    created_at,
                    updated_at
                "#,
            )
            .bind(article_id)
            .bind(author_id)
            .fetch_one(&state.pool)
            .await;

            match wiki_article_create_result {
                Ok(_article) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                    }))
                }
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to add star to wiki article." }))
                }
            }
        }
        Ok(false) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Wiki article not found" })),
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

    match wiki_article_exists(&state.pool, article_id).await {
        Ok(true) => {
            let delete_result = sqlx::query(
                r#"
                DELETE FROM wiki_stars
                WHERE wiki_id = $1 AND user_id = $2
                "#,
            )
            .bind(article_id)
            .bind(author_id)
            .execute(&state.pool)
            .await;

            match delete_result {
                Ok(_) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "success",
                    }))
                }
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to remove star from wiki article." }))
                }
            }
        }
        Ok(false) => HttpResponse::NotFound()
            .json(serde_json::json!({ "error": "Wiki article not found" })),
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
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "Validation failed", "details": errors.to_string() }));
    }

    // Атомарная проверка авторства + обновление одним запросом:
    // если created_by не совпадает — UPDATE затронет 0 строк → 403.
    let result = sqlx::query_as::<_, Wiki>(
        r#"
        UPDATE wiki_articles
        SET
            title          = COALESCE($3, title),
            content        = COALESCE($4, content),
            wiki_type_id   = COALESCE($5, wiki_type_id),
            last_edited_by = $2,
            updated_at     = EXTRACT(EPOCH FROM now())::bigint
        WHERE id = $1 AND created_by = $2
        RETURNING
            id,
            title,
            content,
            wiki_type_id,
            is_confirmed,
            comment_count,
            stars_count,
            created_at,
            updated_at,
            FALSE AS is_starred,
            (SELECT json_build_object(
                'id', u.id, 'username', u.username,
                'first_name', u.first_name, 'last_name', u.last_name,
                'avatar_url', u.avatar_url
            ) FROM users u WHERE u.id = created_by) AS created_by,
            (SELECT json_build_object(
                'id', u.id, 'username', u.username,
                'first_name', u.first_name, 'last_name', u.last_name,
                'avatar_url', u.avatar_url
            ) FROM users u WHERE u.id = last_edited_by) AS last_edited_by,
            (SELECT json_build_object(
                'id', wt.id, 'title', wt.title,
                'created_at', wt.created_at, 'updated_at', wt.updated_at
            ) FROM wiki_types wt WHERE wt.id = wiki_type_id) AS wiki_type
        "#,
    )
    .bind(article_id)
    .bind(author_id)
    .bind(&body.title)
    .bind(&body.content)
    .bind(body.wiki_type_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(article)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": article,
        })),
        Ok(None) => HttpResponse::Forbidden().json(
            serde_json::json!({ "error": "Article not found or you are not the author" })
        ),
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

    // Атомарно: удаляем только если created_by совпадает.
    // rows_affected == 0 → не найдено или не автор → 403.
    let result = sqlx::query(
        "DELETE FROM wiki_articles WHERE id = $1 AND created_by = $2",
    )
    .bind(article_id)
    .bind(author_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            HttpResponse::Ok().json(serde_json::json!({ "status": "success" }))
        }
        Ok(_) => HttpResponse::Forbidden().json(
            serde_json::json!({ "error": "Article not found or you are not the author" })
        ),
        Err(e) => {
            eprintln!("DB error deleting wiki article: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "Failed to delete wiki article" }))
        }
    }
}
