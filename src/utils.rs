use std::fs;
use rand::Rng;

pub fn check_file_exists(filename: &str) -> bool {
    if let Ok(metadata) = fs::metadata(filename) {
        metadata.is_file()
    } else {
        false
    }
}

pub fn get_filepath(full_name: &str, overwrite: bool, timestamped: bool, random_name: bool, custom_filename: Option<String>) -> String {
    let timestamp = chrono::Utc::now();
    // extract extension, the extension must exist otherwise don't use this
    let extension = full_name.split('.').rev().take(1).collect::<String>();
    let file_name = custom_filename.unwrap_or(full_name.split('.').take_while(|v| v != &extension).collect::<String>());

    let filepath = match (timestamped, random_name) {
        (true, true) => format!("./files/public/{}_{}.{}", timestamp.format("%Y%m%d-%H%M%S"), random_string(20), extension),
        (true, false) => format!("./files/public/{}_{}.{}", timestamp.format("%Y%m%d-%H%M%S"), file_name, extension),
        (false, true) => format!("./files/public/{}.{}", random_string(20), extension),
        (false, false) => format!("./files/public/{}.{}", file_name, extension),
    };

    if !overwrite && check_file_exists(&filepath) {
        for i in 1..20 {
            let i = if i > 10 { random_string(10) } else { i.to_string() };
            let filepath = match (timestamped, random_name) {
                (true, true) => format!("./files/public/{}-{}_{}.{}", timestamp.format("%Y%m%d-%H%M%S"), i, random_string(20), extension),
                (true, false) => format!("./files/public/{}-{}_{}.{}", timestamp.format("%Y%m%d-%H%M%S"), i, file_name, extension),
                (false, true) => format!("./files/public/{}-{}.{}", random_string(20), i, extension),
                (false, false) => format!("./files/public/{}-{}.{}", file_name, i, extension),
            };
            if !check_file_exists(&filepath) {
                return filepath;
            }
        }
    }
    return filepath;
}

const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn random_string(length: u32) -> String {
    let mut rng = rand::thread_rng();
    let random_string: String = (0..length)
        .map(|_| {
            let random_index = rng.gen_range(0..CHARSET.len());
            CHARSET.chars().nth(random_index).unwrap()
        })
        .collect();
    random_string
}