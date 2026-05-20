#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Read;

mod commands;
mod db;
mod models;
mod timecalc;

use commands::AppState;
use db::Database;
use flate2::read::GzDecoder;
use std::sync::Mutex;
use tauri::Manager;

// 嵌入压缩后的数据库文件
const DB_DATA: &[u8] = include_bytes!("../data/data.db.gz");
const DB_VERSION: &str = "4";

fn decompress_db(compressed: &[u8], dest: &std::path::Path) -> Result<(), String> {
    let mut decoder = GzDecoder::new(compressed);
    let mut buf = Vec::with_capacity(compressed.len() * 2);
    decoder.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    std::fs::write(dest, &buf).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 获取用户数据目录
            let data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

            let db_path = data_dir.join("data.db");
            let version_path = data_dir.join("data.db.version");

            // 判断是否需要重新解压：文件不存在，或版本不匹配
            let need_extract = !db_path.exists()
                || std::fs::read_to_string(&version_path).unwrap_or_default().trim() != DB_VERSION;

            if need_extract {
                let _ = std::fs::remove_file(data_dir.join("data.db-shm"));
                let _ = std::fs::remove_file(data_dir.join("data.db-wal"));
                let _ = std::fs::remove_file(&db_path);
                decompress_db(DB_DATA, &db_path)?;
                std::fs::write(&version_path, DB_VERSION).map_err(|e| e.to_string())?;
            }

            let db = Database::open(&db_path).expect("Failed to open database");
            db.init_recent_table().expect("Failed to init recent table");

            app.manage(AppState {
                db: Mutex::new(db),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_cities,
            commands::get_recent_cities,
            commands::add_recent_city,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
