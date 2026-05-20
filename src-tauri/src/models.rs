use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CityResult {
    pub geonameid: i64,
    pub name: String,
    pub name_cn: Option<String>,
    pub country_iso: String,
    pub country_name: String,
    pub country_name_cn: Option<String>,
    pub timezone: String,
    pub local_time: String,
    pub local_date: String,
    pub utc_offset: String,
    pub is_capital: bool,
    pub population: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub feature_code: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct CountryWithCities {
    pub country_iso: String,
    pub country_name: String,
    pub country_name_cn: Option<String>,
    pub continent: Option<String>,
    pub cities: Vec<CitySummary>,
}

#[derive(Serialize, Clone)]
pub struct CitySummary {
    pub geonameid: i64,
    pub name: String,
    pub name_cn: Option<String>,
    pub timezone: String,
    pub local_time: String,
    pub is_capital: bool,
    pub feature_code: Option<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub cities: Vec<CityResult>,
    pub countries: Vec<CountryWithCities>,
}
