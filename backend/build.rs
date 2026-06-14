use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Получаем путь к корневому .env относительно директории манифеста (backend/)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let env_path = Path::new(&manifest_dir).join("../.env");

    // Указываем Cargo отслеживать изменения в корневом .env
    if let Some(path_str) = env_path.to_str() {
        println!("cargo:rerun-if-changed={}", path_str);
    }

    if let Ok(content) = fs::read_to_string(&env_path) {
        let mut user = String::new();
        let mut password = String::new();
        let mut db_name = String::new();

        for line in content.lines() {
            let line = line.trim();
            // Пропускаем комментарии и строки без знака '='
            if line.starts_with('#') || !line.contains('=') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                match parts[0].trim() {
                    "DB_USER" => user = parts[1].trim().trim_matches('"').trim_matches('\'').to_string(),
                    "DB_PASSWORD" => password = parts[1].trim().trim_matches('"').trim_matches('\'').to_string(),
                    "DB_NAME" => db_name = parts[1].trim().trim_matches('"').trim_matches('\'').to_string(),
                    _ => {}
                }
            }
        }

        if !user.is_empty() && !password.is_empty() && !db_name.is_empty() {
            let db_url = format!("postgres://{}:{}@127.0.0.1:5432/{}", user, password, db_name);
            println!("cargo:rustc-env=DATABASE_URL={}", db_url);
        }
    }
}
