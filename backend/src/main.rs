#[macro_use]
extern crate rocket;
mod assets;
mod auth;
mod cors;
mod db;
mod models;
mod perms;

use crate::cors::Cors;
use db::connect_db;
use dotenvy::dotenv;
use rocket::Config;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};

static FILES_DIR: OnceLock<String> = OnceLock::new();

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
                auth::login,
                auth::register,
                auth::logout,
                auth::get_user,
                auth::change_password,
                assets::create_folder,
                assets::delete_folder,
                assets::edit_folder,
                assets::get_all_folders,
                assets::get_folders_path,
                assets::get_folders,
                assets::upload_file,
                assets::get_all_files,
                assets::get_files,
                assets::delete_file,
                assets::edit_file,
            ],
        )
}
