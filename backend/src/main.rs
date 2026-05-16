use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};

mod db;
mod handlers;
mod helpers;
mod models;

use db::setup::setup_db;
use handlers::{
    auth::telegram_auth,
    post::{create_post, get_posts, post_rating_update},
    wiki::{create_wiki_article, get_wiki_article, get_wiki_types},
};
use models::app::AppState;

use crate::handlers::wiki::get_wiki_articles;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let pool = setup_db().await;
    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let app_state: web::Data<AppState> = web::Data::new(AppState {
        pool,
        bot_token,
        jwt_secret,
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:80")
            .allowed_origin("http://127.0.0.1")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
                http::header::COOKIE, // нужен, иначе actix-cors отрезает заголовок auth_token
            ])
            .supports_credentials()
            .max_age(3600);

        App::new().wrap(cors).app_data(app_state.clone()).service(
            web::scope("/api")
                .service(telegram_auth)
                .service(get_posts)
                .service(create_post)
                .service(post_rating_update)
                .service(get_wiki_types)
                .service(create_wiki_article)
                .service(get_wiki_article)
                .service(get_wiki_articles),
        )
    })
    .bind(("0.0.0.0", 6080))?
    .run()
    .await
}
