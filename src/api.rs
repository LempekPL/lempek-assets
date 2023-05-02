use actix_multipart::form;
use actix_multipart::form::{MultipartForm, text};
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_web::{web, Responder, Scope, get, post, HttpResponse};
use rand::distributions::{Alphanumeric, DistString};
use crate::DOMAIN;

const CODE: &'static str = "zaq1@WSXz";

#[derive(serde::Serialize, proc_lempek_assets::SerJsonBody)]
struct SuccessfulUpload {
    errors: Option<Vec<usize>>,
    path: Vec<String>,
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
    #[multipart(rename = "code")]
    form_code: Text<String>,
    #[multipart(rename = "replace")]
    replace: Text<bool>,
}

pub fn api_endpoints_v1() -> Scope {
    web::scope("/api/v1")
        .service(upload_v1)
}

#[post("/upload")]
async fn upload_v1(
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    let Text(form_code) = form.form_code;
    let Text(replace) = form.replace;
    if CODE.to_string() != form_code {
        return SuccessfulUpload { errors: None, path: vec![] };
    }
    let mut errors = Vec::new();
    let paths: Vec<String> = form.files.into_iter().enumerate().filter_map(|(p, f)| {
        if f.size > 100_000_000 {
            errors.push(p);
            return Some("File too big. Max 100Mb".to_string());
        }
        if f.size == 0 {
            errors.push(p);
            return Some("File not found".to_string());
        }

        let file_name = f.file_name.clone()
            .unwrap_or(Alphanumeric.sample_string(&mut rand::thread_rng(), 8));
        let mut path = format!("./static/files/{}", file_name);
        if replace {
            if let Err(e) = f.file.persist(path.clone()) {
                if let Err(e) = e.file.persist(path.clone()) {
                    errors.push(p);
                    return Some(e.error.to_string());
                }
            }
        } else {
            if let Err(e) = f.file.persist_noclobber(path.clone()) {
                let random = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
                path = format!("./static/files/{}_{}", random, file_name);

                if let Err(e) = e.file.persist_noclobber(path.clone()) {
                    errors.push(p);
                    return Some(e.error.to_string());
                }
            }
        }
        Some(path.replace("./static/files/", &format!("{}/files/", DOMAIN)))
    }).collect();
    dbg!(paths.clone());

    if errors.len() > 0 {
        SuccessfulUpload { errors: Some(errors), path: paths }
    } else {
        SuccessfulUpload { errors: None, path: paths }
    }
}