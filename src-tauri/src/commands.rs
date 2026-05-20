use crate::db::Database;
use crate::models::{CityResult, SearchResponse};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[tauri::command]
pub fn search_cities(state: State<AppState>, query: String) -> Result<SearchResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let cities = db.search_cities(&query, 10).map_err(|e| e.to_string())?;

    let mut countries = Vec::new();
    let isos = db.match_countries(&query).map_err(|e| e.to_string())?;
    for iso in isos {
        if let Some(c) = db.get_country_cities(&iso, 8).map_err(|e| e.to_string())? {
            countries.push(c);
        }
    }

    Ok(SearchResponse { cities, countries })
}

#[tauri::command]
pub fn get_recent_cities(state: State<AppState>) -> Result<Vec<CityResult>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_cities(10).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_recent_city(state: State<AppState>, city: CityResult) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_recent_city(&city).map_err(|e| e.to_string())
}
