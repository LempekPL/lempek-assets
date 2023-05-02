mod api;
mod frontend;

use actix_multipart::form::tempfile::TempFileConfig;
use actix_web::{App, HttpServer};

const DOMAIN: &'static str = "http://assets.lempek.dev";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(TempFileConfig::default().directory("./static"))
            .service(api::api_endpoints_v1())
            .service(frontend::frontend())
    })
        .bind(("127.0.0.1", 5422))?
        .run()
        .await
}