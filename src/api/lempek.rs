use actix_multipart::form::{MultipartForm};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::{web, Responder, Scope, post, HttpResponse, Either};
use crate::utils::get_filepath;

const CODE: &'static str = "zaq1@WSXz";

#[derive(serde::Serialize, proc_lempek_assets::SerJsonBody)]
struct SuccessfulUpload {
    path: String,
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
    // default: false
    random_name: Option<Text<String>>,
}

pub fn api_endpoints_lempek() -> Scope {
    web::scope("/lempek")
        .service(upload)
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

// #[post("/upload_old")]
// async fn upload_old(
//     MultipartForm(form): MultipartForm<UploadForm>,
// ) -> impl Responder {
// let Text(form_code) = form.form_code;
// let Text(replace) = form.replace;
// if CODE.to_string() != form_code {
//     return SuccessfulUpload { errors: None, path: vec![] };
// }
// let mut errors = Vec::new();
// let paths: Vec<String> = form.files.into_iter().enumerate().filter_map(|(p, f)| {
//     if f.size > 100_000_000 {
//         errors.push(p);
//         return Some("File too big. Max 100Mb".to_string());
//     }
//     if f.size == 0 {
//         errors.push(p);
//         return Some("File not found".to_string());
//     }
//
//     let file_name = f.file_name.clone()
//         .unwrap_or(Alphanumeric.sample_string(&mut rand::thread_rng(), 8));
//     let mut path = format!("./static/files/{}", file_name);
//     if replace {
//         if let Err(e) = f.file.persist(path.clone()) {
//             if let Err(e) = e.file.persist(path.clone()) {
//                 errors.push(p);
//                 return Some(e.error.to_string());
//             }
//         }
//     } else {
//         if let Err(e) = f.file.persist_noclobber(path.clone()) {
//             let random = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
//             path = format!("./static/files/{}_{}", random, file_name);
//
//             if let Err(e) = e.file.persist_noclobber(path.clone()) {
//                 errors.push(p);
//                 return Some(e.error.to_string());
//             }
//         }
//     }
//     Some(path.replace("./static/files/", &format!("{}/files/", DOMAIN)))
// }).collect();
// dbg!(paths.clone());
//
// if errors.len() > 0 {
//     SuccessfulUpload { errors: Some(errors), path: paths }
// } else {
//     SuccessfulUpload { errors: None, path: paths }
// }
// }