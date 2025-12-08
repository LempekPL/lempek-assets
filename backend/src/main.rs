#[macro_use]
extern crate rocket;
mod assets;
mod auth;
mod cors;
mod db;
mod models;
mod perms;

use crate::cors::Cors;
use crate::models::ApiResponse;
use chrono::Duration;
use db::connect_db;
use dotenvy::dotenv;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Config;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};

const ACCESS_TOKEN_TIME: Duration = Duration::minutes(5);
const REFRESH_TOKEN_TIME: Duration = Duration::days(30);

static FILES_DIR: OnceLock<String> = OnceLock::new();

pub type ApiResult<T = (Status, Json<ApiResponse>)> = Result<T, (Status, Json<ApiResponse>)>;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    FILES_DIR.set(env::var("FILES_DIR").unwrap()).unwrap();

    if !PathBuf::from(&FILES_DIR.get().unwrap()).exists() {
        fs::create_dir_all(&FILES_DIR.get().unwrap()).unwrap();
    }

    let config = Config::figment();
    let temp_dir: String = config
        .extract_inner("temp_dir")
        .unwrap_or_else(|_| "../tmp".to_string());
    if !PathBuf::from(&temp_dir).exists() {
        fs::create_dir_all(&temp_dir).expect("create temp dir");
    }

    rocket::build()
        .manage(connect_db().await)
        .attach(Cors)
        .mount(
            "/api",
            routes![
                cors::options_handler,
                auth::endpoints::login,
                auth::endpoints::logout,
                auth::endpoints::get_user,
                auth::endpoints::get_user_all,
                auth::endpoints::get_user_all_admin,
                auth::endpoints::get_user_tokens,
                auth::endpoints::remove_user_token,
                auth::endpoints::change_password,
                auth::endpoints::change_username,
                auth::endpoints::create_user,
                assets::create_folder,
                assets::delete_folder,
                assets::edit_folder,
                assets::move_folder,
                assets::get_folder,
                assets::get_all_folders,
                assets::get_folders_path,
                assets::get_folders,

                assets::upload_file,
                assets::get_all_files,
                assets::get_files,
                assets::delete_file,
                assets::edit_file,
                assets::move_file,
            ],
        )
}
