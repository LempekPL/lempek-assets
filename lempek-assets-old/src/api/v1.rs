use actix_multipart::form::{MultipartForm};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::{web, Responder, Scope, post, get, HttpResponse, FromRequest};
use actix_web::http::StatusCode;
use chrono::Duration;
use jsonwebtoken::{DecodingKey, encode, decode, EncodingKey, TokenData, Validation, Header};
use serde::{Deserialize, Serialize};
use crate::api::{get_file_names_in_public, ReturnVecString};

#[derive(serde::Serialize, proc_lempek_assets::SerJsonBody)]
struct SuccessfulUpload {
    errors: Option<Vec<usize>>,
    path: Vec<String>,
}

// #[derive(Debug, MultipartForm)]
// struct BulkUploadForm {
//     #[multipart(rename = "files")]
//     files: Vec<TempFile>,
//
//     // #[multipart(rename = "replace")]
//     // replace: Text<bool>,
// }


#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "512 MB")]
    files: TempFile,
    // if no name provided then use tempfiles name
    #[multipart(limit = "8 MB")]
    name: Option<Text<String>>,
    // default: true
    timestamped: Option<Text<String>>,
    // default: false
    overwrite: Option<Text<String>>,
}

pub fn api_endpoints_v1() -> Scope {
    web::scope("/v1")
        .service(upload)
        .service(login)
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn generate_token(user_id: &str) -> String {
    let header = Header::default();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (chrono::Utc::now() + Duration::hours(1)).timestamp() as usize,
    };
    let encoding_key = EncodingKey::from_secret("sixtynine108923!#".as_bytes());
    encode(&header, &claims, &encoding_key).unwrap()
}

fn verify_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret("sixtynine108923!#".as_bytes());
    decode(token, &decoding_key, &Validation::default())
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    token: String,
}

#[post("/login")]
async fn login(data: web::Json<AuthRequest>) -> actix_web::Result<HttpResponse> {
    if verify_token(&data.token).is_ok() {
        Ok(HttpResponse::Ok().body("Login successful"))
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}


#[post("/upload")]
async fn upload(
    MultipartForm(_form): MultipartForm<UploadForm>
) -> impl Responder {
    // let Some(ref file_name) = form.files.file_name else {
    //     return ("error while parsing files", StatusCode::from_u16(400).unwrap());
    // };
    // if form.files.content_type.as_ref().is_some_and(|t| t.to_string() == "application/octet-stream") {
    //     return ("unsupported content type", StatusCode::from_u16(400).unwrap());
    // }
    // let timestamped = form.timestamped.and_then(|v| Some(v.0 == "true")).unwrap_or(true);
    // let overwrite = form.overwrite.and_then(|v| Some(v.0 == "true")).unwrap_or(false);
    // let filepath = get_filepath(file_name, overwrite, timestamped, form.name.and_then(|v| Some(v.0)));
    // dbg!(&filepath);
    // if let Err(e) = form.files.file.persist(&filepath) {
    //     eprintln!("{}", e);
    //     return ("Something went wrong...", StatusCode::from_u16(500).unwrap());
    // }
    //
    ("", StatusCode::from_u16(200).unwrap())
}

#[get("/files")]
pub async fn files() -> ReturnVecString {
    ReturnVecString(get_file_names_in_public())
}