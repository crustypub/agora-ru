use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web::{self}};
use std::sync::Arc;

mod db;
mod handlers;
mod helpers;
mod models;

use db::setup::setup_db;
use handlers::{
    auth::{telegram_auth, telegram_auth_request, telegram_auth_check, telegram_logout, auth_me},
    post::{create_post, get_posts, post_rating_update},
    wiki::{create_wiki_article, get_wiki_article, get_wiki_types, update_wiki_article, delete_wiki_article},
};
use models::app::AppState;

use crate::{handlers::{chats::get_rooms, user::{check_user_exist, delete_avatar, get_users, update_user_info, upload_avatar}, wiki::{add_star_to_wiki, get_wiki_articles, remove_star_from_wiki, upload_wiki_article_images}}, models::chat::ChatServerState};
use crate::handlers::comment::{create_comment, delete_comment, edit_comment, get_comments};
use crate::handlers::chats_ws::chat_ws_route;
use crate::handlers::chats::{create_room, add_member, remove_member, delete_message, get_room_messages, upload_file, parse_link};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::from_filename("shared.env").ok();
    dotenvy::from_filename("../shared.env").ok();
    dotenvy::dotenv().ok();
    let pool = setup_db().await;
    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let client = helpers::proxy::get_telegram_client().expect("Failed to initialize HTTP client");
    let s3_client = helpers::s3::setup_s3_client().await;
    let s3_public_client = helpers::s3::setup_s3_public_client().await;

    // Get bot username from Telegram API
    let bot_username = helpers::telegram_bot::get_bot_username(&client, &bot_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to fetch Telegram bot username: {}. Polling might fail.", e);
            "unknown_bot".to_string()
        });

    // Start background polling task
    let polling_client = client.clone();
    let polling_token = bot_token.clone();
    let polling_pool = pool.clone();
    tokio::spawn(async move {
        helpers::telegram_bot::run_bot_polling(polling_client, polling_token, polling_pool).await;
    });
    let chat_server = Arc::new(ChatServerState::new());

    let app_state: web::Data<AppState> = web::Data::new(AppState {
        pool,
        bot_token,
        jwt_secret,
        bot_username,
        client,
        s3_client,
        s3_public_client,
        chat_server,
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
                .service(telegram_auth_request)
                .service(telegram_auth_check)
                .service(telegram_logout)
                .service(auth_me)
                .service(get_posts)
                .service(create_post)
                .service(post_rating_update)
                .service(get_wiki_types)
                .service(create_wiki_article)
                .service(get_wiki_article)
                .service(get_wiki_articles)
                .service(add_star_to_wiki)
                .service(remove_star_from_wiki)
                .service(update_wiki_article)
                .service(upload_wiki_article_images)
                .service(delete_wiki_article)
                .service(get_comments)
                .service(create_comment)
                .service(edit_comment)
                .service(delete_comment)
                .service(update_user_info)
                .service(get_users)
                .service(upload_avatar)
                .service(delete_avatar)
                .service(chat_ws_route)
                .service(create_room)
                .service(get_rooms)
                .service(parse_link)
                .service(get_room_messages)
                .service(add_member)
                .service(remove_member)
                .service(delete_message)
                .service(upload_file)
                .service(check_user_exist)
        )
    })
    .bind(("0.0.0.0", 6080))?
    .run()
    .await
}
