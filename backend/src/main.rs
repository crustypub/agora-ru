use actix_web::{App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use serde::Deserialize;

mod handlers;
mod db;
mod models;
mod helpers;

use handlers::auth::telegram_auth;
use db::setup::setup_db;
use models::{app::AppState};

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
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(echo)
            .service(telegram_auth)
            .route("/hey", web::get().to(manual_hello))
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
