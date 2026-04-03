use actix_web::{App, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web};
use serde::Deserialize;

#[derive(Deserialize)]
struct MyData {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
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
