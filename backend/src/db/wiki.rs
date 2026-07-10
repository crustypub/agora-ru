use sqlx::PgPool;
use uuid::Uuid;
use crate::models::wiki::{
    CreateWikiArticle, CreateWikiStar, WikIArticlesParams, Wiki, WikiListItem, WikiType,
    WikiTypeResponse,
};
use crate::models::app::SortField;

pub async fn check_wiki_article_exists(
    pool: &PgPool,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM wiki_articles WHERE id = $1)")
            .bind(id)
            .fetch_one(pool)
            .await?;
    Ok(exists)
}

pub async fn get_wiki_articles_paginated(
    pool: &PgPool,
    current_user_id: Option<Uuid>,
    params: &WikIArticlesParams,
) -> Result<(Vec<WikiListItem>, i64), sqlx::Error> {
    let limit = params.limit;
    let offset = params.offset();

    let sort_col = params.sort_by.as_sql_column();
    let sort_dir = params.sort_order.as_sql();

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

    let search = params
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(crate::helpers::api::escape_like_pattern);

    let articles = sqlx::query_as::<_, WikiListItem>(&articles_sql)
        .bind(params.wiki_type)
        .bind(params.is_confirmed)
        .bind(&search)
        .bind(limit)
        .bind(offset)
        .bind(current_user_id)
        .fetch_all(pool)
        .await?;

    let count = sqlx::query_scalar::<_, i64>(count_sql)
        .bind(params.wiki_type)
        .bind(params.is_confirmed)
        .bind(&search)
        .fetch_one(pool)
        .await?;

    Ok((articles, count))
}

pub async fn get_wiki_types(pool: &PgPool) -> Result<Vec<WikiType>, sqlx::Error> {
    sqlx::query_as::<_, WikiType>(
        r#"
        SELECT id, title, created_at, updated_at
        FROM wiki_types
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn create_wiki_article(
    pool: &PgPool,
    author_id: Uuid,
    title: &str,
    content: &str,
    wiki_type_id: i32,
) -> Result<(CreateWikiArticle, WikiTypeResponse), sqlx::Error> {
    let article = sqlx::query_as::<_, CreateWikiArticle>(
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
    .bind(title)
    .bind(content)
    .bind(wiki_type_id)
    .bind(author_id)
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    let wiki_type = sqlx::query_as::<_, WikiTypeResponse>(
        r#"
        SELECT id, title, created_at, updated_at
        FROM wiki_types
        WHERE id = $1
        "#,
    )
    .bind(wiki_type_id)
    .fetch_one(pool)
    .await?;

    Ok((article, wiki_type))
}

pub async fn get_wiki_article(
    pool: &PgPool,
    article_id: Uuid,
    current_user_id: Option<Uuid>,
) -> Result<Option<Wiki>, sqlx::Error> {
    sqlx::query_as::<_, Wiki>(
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
    .fetch_optional(pool)
    .await
}

pub async fn add_wiki_star(
    pool: &PgPool,
    article_id: Uuid,
    user_id: Uuid,
) -> Result<CreateWikiStar, sqlx::Error> {
    sqlx::query_as::<_, CreateWikiStar>(
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
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn remove_wiki_star(
    pool: &PgPool,
    article_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM wiki_stars
        WHERE wiki_id = $1 AND user_id = $2
        "#,
    )
    .bind(article_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_wiki_article_content_and_author(
    pool: &PgPool,
    article_id: Uuid,
) -> Result<Option<(String, Uuid)>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct ContentAndAuthor {
        content: String,
        created_by: Uuid,
    }
    
    let res = sqlx::query_as::<_, ContentAndAuthor>(
        "SELECT content, created_by FROM wiki_articles WHERE id = $1"
    )
    .bind(article_id)
    .fetch_optional(pool)
    .await?;

    Ok(res.map(|r| (r.content, r.created_by)))
}

pub async fn update_wiki_article(
    pool: &PgPool,
    article_id: Uuid,
    author_id: Uuid,
    title: Option<&str>,
    content: Option<&str>,
    wiki_type_id: Option<i32>,
) -> Result<Option<Wiki>, sqlx::Error> {
    sqlx::query_as::<_, Wiki>(
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
    .bind(title)
    .bind(content)
    .bind(wiki_type_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_wiki_article(
    pool: &PgPool,
    article_id: Uuid,
    author_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "DELETE FROM wiki_articles WHERE id = $1 AND created_by = $2 RETURNING content",
    )
    .bind(article_id)
    .bind(author_id)
    .fetch_optional(pool)
    .await
}
