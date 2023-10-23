mod api;
mod dashboard;
mod utils;

use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{web, App, HttpServer};

const DOMAIN: &'static str = "https://assets.lempek.dev";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory("temp"))
            .service(api::api())
            .service(dashboard::dashboard())
            .service(web::scope("/").service(web::redirect("/", "/dashboard")))
    })
        .bind(("127.0.0.1", 5422))?
        .run()
        .await
}