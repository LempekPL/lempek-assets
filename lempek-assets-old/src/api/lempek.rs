use actix_multipart::form::{MultipartForm};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::{web, Responder, Scope, post, HttpResponse, Either, get};
use crate::api::{get_file_names_in_public, ReturnVecString};
use crate::utils::get_filepath;

#[derive(serde::Serialize, proc_lempek_assets::SerJsonBody)]
struct SuccessfulUpload {
    path: String,
}

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
    // default: false
    random_name: Option<Text<String>>,
}

pub fn api_endpoints_lempek() -> Scope {
    web::scope("/lempek")
        .service(upload)
        .service(files)
}

#[post("/upload")]
async fn upload(
    MultipartForm(form): MultipartForm<UploadForm>
) -> Either<HttpResponse, SuccessfulUpload> {
    let Some(ref file_name) = form.files.file_name else {
        return Either::Left(HttpResponse::BadRequest().body("error while parsing files"));
    };
    if form.files.content_type.as_ref().is_some_and(|t| t.to_string() == "application/octet-stream") {
        return Either::Left(HttpResponse::BadRequest().body("unsupported content type"));
    }
    let timestamped = form.timestamped.and_then(|v| Some(v.0 == "true")).unwrap_or(true);
    let overwrite = form.overwrite.and_then(|v| Some(v.0 == "true")).unwrap_or(false);
    let random_name = form.random_name.and_then(|v| Some(v.0 == "true")).unwrap_or(false);
    let filepath = get_filepath(file_name, overwrite, timestamped, random_name, form.name.and_then(|v| Some(v.0)));
    if let Err(e) = form.files.file.persist(&filepath) {
        eprintln!("{}", e);
        return Either::Left(HttpResponse::BadGateway().body("Something went wrong..."));
    }

    Either::Right(SuccessfulUpload {
        path: filepath.replace("./files", "")
    })
}

#[get("/files")]
pub async fn files() -> ReturnVecString {
    ReturnVecString(get_file_names_in_public())
}