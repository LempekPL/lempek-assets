use actix_web::{Scope, web};

mod v1;
mod lempek;

pub fn api() -> Scope {
    web::scope("/api")
        .service(lempek::api_endpoints_lempek())
        .service(v1::api_endpoints_v1())
}