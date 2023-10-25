use actix_web::{Scope, web};

mod v1;
mod lempek;

pub fn api() -> Scope {
    web::scope("/api")
        .service(lempek::api_endpoints_lempek())
        .service(v1::api_endpoints_v1())
}

fn get_file_names_in_public() -> Vec<String> {
    let paths = std::fs::read_dir("./files/public").unwrap();
    let mut files = Vec::new();
    for path in paths {
        if let Ok(path) = path {
            files.push(format!("/public/{}", path.file_name().into_string().unwrap()));
        } else {
            continue;
        }
    }

    files
}

#[derive(serde::Serialize, proc_lempek_assets::SerJsonBody)]
struct ReturnVecString(Vec<String>);