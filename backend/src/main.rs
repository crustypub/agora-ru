use actix_cors::Cors;
use actix_web::{
    get, http, post, web, App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;

mod db;
mod handlers;
mod helpers;
mod models;

use db::setup::setup_db;
use handlers::auth::telegram_auth;
use models::app::AppState;

#[derive(Deserialize)]
struct MyData {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let pool = setup_db().await;
    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");

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
            ])
            .supports_credentials()
            .max_age(3600);

        App::new().wrap(cors).app_data(app_state.clone()).service(
            web::scope("/api")
                .service(hello)
                .service(echo)
                .service(telegram_auth)
                .route("/hey", web::get().to(manual_hello)),
        )
    })
    .bind(("0.0.0.0", 6080))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello(item: web::Json<MyData>) -> impl Responder {
    format!("wot, it's working! {:?}", item.name)
}
