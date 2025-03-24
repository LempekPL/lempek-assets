#[macro_use]
extern crate rocket;
mod auth;
mod db;
mod files;
mod models;
mod cors;

use db::connect_db;
use dotenvy::dotenv;
use rocket::fairing::{Fairing};
use rocket::{Config};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};
use uuid::Uuid;
use crate::cors::Cors;

static ADMIN_UUID: OnceLock<Uuid> = OnceLock::new();
static FILES_DIR: OnceLock<String> = OnceLock::new();
static PUBLIC_DIR_UUID: OnceLock<Uuid> = OnceLock::new();

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let config = Config::figment();
    let temp_dir: String = config
        .extract_inner("temp_dir")
        .unwrap_or_else(|_| "/tmp".to_string());
    if !PathBuf::from(&temp_dir).exists() {
        fs::create_dir_all(&temp_dir).expect("create temp dir");
    }
    ADMIN_UUID
        .set(Uuid::parse_str(&*env::var("ADMIN_USER_UUID").unwrap()).unwrap())
        .unwrap();
    FILES_DIR.set(env::var("FILES_DIR").unwrap()).unwrap();

    if !PathBuf::from(&FILES_DIR.get().unwrap()).exists() {
        fs::create_dir_all(&FILES_DIR.get().unwrap()).unwrap();
    }

    rocket::build()
        .manage(connect_db().await)
        .attach(Cors)
        .mount(
            "/",
            routes![
                cors::options_handler,
                auth::login,
                auth::register,
                auth::logout,
                auth::get_user,
                files::get_items,
                files::upload_file,
                files::get_files,
                files::create_folder,
                files::get_folders,
                files::delete_folder,
            ],
        )
}
