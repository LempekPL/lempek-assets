#[macro_use]
extern crate rocket;
mod auth;
mod db;
mod files;
mod models;

use std::fs;
use std::path::PathBuf;
use db::connect_db;
use dotenvy::dotenv;
use rocket::Config;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let config = Config::figment();
    let temp_dir: String = config.extract_inner("temp_dir").unwrap_or_else(|_| "/tmp".to_string());
    if !PathBuf::from(&temp_dir).exists() {
        fs::create_dir_all(&temp_dir).expect("create temp dir");
    }
    rocket::build()
        .manage(connect_db().await)
        .mount("/", routes![auth::login, auth::register, files::upload_file])
}
