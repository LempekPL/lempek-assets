use dotenvy::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Method, Status};
use rocket::{Request, Response};
use std::env;

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if let Some(origin) = request.headers().get_one("Origin") {
            let allowed = env::var("ALLOWED_ORIGINS").unwrap_or_default();
            let allowed_origins: Vec<&str> = allowed.split(",").collect();

            if allowed_origins.contains(&origin) {
                response.set_header(Header::new("Access-Control-Allow-Origin", origin));
            }
        }
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "content-type"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS, DELETE, PATCH",
        ));

        if request.method() == Method::Options {
            response.set_status(Status::Ok);
        }
    }
}

#[options("/<_..>")]
pub fn options_handler() -> Status {
    Status::Ok
}
