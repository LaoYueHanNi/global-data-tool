use crate::models::{CityResult, CitySummary, CountryWithCities};
use crate::timecalc;
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    pub fn init_recent_table(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recent_searches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                geonameid INTEGER NOT NULL UNIQUE,
                name TEXT NOT NULL,
                name_cn TEXT,
                country_iso TEXT NOT NULL,
                country_name TEXT NOT NULL,
                country_name_cn TEXT,
                timezone TEXT NOT NULL,
                is_capital INTEGER NOT NULL DEFAULT 0,
                feature_code TEXT,
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                population INTEGER NOT NULL,
                searched_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_recent_geonameid ON recent_searches(geonameid);
            CREATE INDEX IF NOT EXISTS idx_recent_searched_at ON recent_searches(searched_at DESC);"
        )?;
        Ok(())
    }

    pub fn get_recent_cities(&self, limit: i64) -> Result<Vec<CityResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT geonameid, name, name_cn, country_iso, country_name, country_name_cn,
                    timezone, is_capital, feature_code, latitude, longitude, population
             FROM recent_searches
             ORDER BY searched_at DESC
             LIMIT ?1"
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(CityRow {
                geonameid: row.get(0)?,
                name: row.get(1)?,
                name_cn: row.get(2)?,
                country_iso: row.get(3)?,
                timezone: row.get(6)?,
                population: row.get(11)?,
                is_capital: row.get::<_, i32>(7)? != 0,
                latitude: row.get(9)?,
                longitude: row.get(10)?,
                feature_code: row.get(8)?,
                country_name: row.get(4)?,
                country_name_cn: row.get(5)?,
            })
        })?;

        // 实时计算时间
        let mut results = Vec::new();
        for row in rows {
            let row = row?;
            let (local_time, local_date, utc_offset) = timecalc::get_local_time(&row.timezone);
            results.push(CityResult {
                geonameid: row.geonameid,
                name: row.name,
                name_cn: row.name_cn,
                country_iso: row.country_iso,
                country_name: row.country_name,
                country_name_cn: row.country_name_cn,
                timezone: row.timezone,
                local_time,
                local_date,
                utc_offset,
                is_capital: row.is_capital,
                population: row.population,
                latitude: row.latitude,
                longitude: row.longitude,
                feature_code: row.feature_code,
            });
        }
        Ok(results)
    }

    pub fn add_recent_city(&self, city: &CityResult) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO recent_searches
             (geonameid, name, name_cn, country_iso, country_name, country_name_cn,
              timezone, is_capital, feature_code, latitude, longitude, population)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                city.geonameid,
                city.name,
                city.name_cn,
                city.country_iso,
                city.country_name,
                city.country_name_cn,
                city.timezone,
                city.is_capital as i32,
                city.feature_code,
                city.latitude,
                city.longitude,
                city.population,
            ],
        )?;

        // 只保留最近10条
        self.conn.execute(
            "DELETE FROM recent_searches WHERE id NOT IN (
                SELECT id FROM recent_searches ORDER BY searched_at DESC LIMIT 10
            )",
            [],
        )?;

        Ok(())
    }

    pub fn search_cities(&self, query: &str, limit: i64) -> Result<Vec<CityResult>> {
        let has_cjk = query.chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FFF}');

        let mut results = if has_cjk {
            self.search_cities_fts_cjk(query, limit)?
        } else {
            self.search_cities_fts(query, limit)?
        };

        if has_cjk && results.len() < limit as usize {
            let fallback = self.search_cities_like(query, limit)?;
            let existing_ids: Vec<i64> = results.iter().map(|c| c.geonameid).collect();
            for city in fallback {
                if !existing_ids.contains(&city.geonameid) {
                    results.push(city);
                }
            }
        }

        results.sort_by(|a, b| {
            b.is_capital.cmp(&a.is_capital)
                .then(b.population.cmp(&a.population))
        });
        results.truncate(limit as usize);
        Ok(results)
    }

    fn search_cities_fts(&self, query: &str, limit: i64) -> Result<Vec<CityResult>> {
        let fts_query = format!("{}*", query.replace("\"", "").replace("*", ""));
        let mut stmt = self.conn.prepare(
            "SELECT c.geonameid, c.name, c.name_cn, c.country_iso, c.timezone,
                    c.population, c.is_capital, c.latitude, c.longitude, c.feature_code,
                    co.name as country_name, co.name_cn as country_name_cn
             FROM city_search cs
             JOIN cities c ON c.geonameid = cs.geonameid
             JOIN countries co ON co.iso = c.country_iso
             WHERE city_search MATCH ?1
             ORDER BY c.is_capital DESC, c.population DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![fts_query, limit], |row| {
            Ok(CityRow {
                geonameid: row.get(0)?,
                name: row.get(1)?,
                name_cn: row.get(2)?,
                country_iso: row.get(3)?,
                timezone: row.get(4)?,
                population: row.get(5)?,
                is_capital: row.get::<_, i32>(6)? != 0,
                latitude: row.get(7)?,
                longitude: row.get(8)?,
                feature_code: row.get(9)?,
                country_name: row.get(10)?,
                country_name_cn: row.get(11)?,
            })
        })?;

        self.build_results(rows)
    }

    fn search_cities_fts_cjk(&self, query: &str, limit: i64) -> Result<Vec<CityResult>> {
        let fts_query = format!("name_cn : {}*", query.replace("\"", "").replace("*", ""));
        let mut stmt = self.conn.prepare(
            "SELECT c.geonameid, c.name, c.name_cn, c.country_iso, c.timezone,
                    c.population, c.is_capital, c.latitude, c.longitude, c.feature_code,
                    co.name as country_name, co.name_cn as country_name_cn
             FROM city_search cs
             JOIN cities c ON c.geonameid = cs.geonameid
             JOIN countries co ON co.iso = c.country_iso
             WHERE city_search MATCH ?1
             ORDER BY c.is_capital DESC, c.population DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![fts_query, limit], |row| {
            Ok(CityRow {
                geonameid: row.get(0)?,
                name: row.get(1)?,
                name_cn: row.get(2)?,
                country_iso: row.get(3)?,
                timezone: row.get(4)?,
                population: row.get(5)?,
                is_capital: row.get::<_, i32>(6)? != 0,
                latitude: row.get(7)?,
                longitude: row.get(8)?,
                feature_code: row.get(9)?,
                country_name: row.get(10)?,
                country_name_cn: row.get(11)?,
            })
        })?;

        self.build_results(rows)
    }

    fn search_cities_like(&self, query: &str, limit: i64) -> Result<Vec<CityResult>> {
        let pattern = format!("{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT c.geonameid, c.name, c.name_cn, c.country_iso, c.timezone,
                    c.population, c.is_capital, c.latitude, c.longitude, c.feature_code,
                    co.name as country_name, co.name_cn as country_name_cn
             FROM city_names cn
             JOIN cities c ON c.geonameid = cn.geonameid
             JOIN countries co ON co.iso = c.country_iso
             WHERE cn.name LIKE ?1
             ORDER BY c.is_capital DESC, c.population DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(CityRow {
                geonameid: row.get(0)?,
                name: row.get(1)?,
                name_cn: row.get(2)?,
                country_iso: row.get(3)?,
                timezone: row.get(4)?,
                population: row.get(5)?,
                is_capital: row.get::<_, i32>(6)? != 0,
                latitude: row.get(7)?,
                longitude: row.get(8)?,
                feature_code: row.get(9)?,
                country_name: row.get(10)?,
                country_name_cn: row.get(11)?,
            })
        })?;

        self.build_results(rows)
    }

    fn build_results(&self, rows: rusqlite::MappedRows<impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<CityRow>>) -> Result<Vec<CityResult>> {
        let mut results = Vec::new();
        for row in rows {
            let row = row?;
            let (local_time, local_date, utc_offset) = timecalc::get_local_time(&row.timezone);
            results.push(CityResult {
                geonameid: row.geonameid,
                name: row.name,
                name_cn: row.name_cn,
                country_iso: row.country_iso,
                country_name: row.country_name,
                country_name_cn: row.country_name_cn,
                timezone: row.timezone,
                local_time,
                local_date,
                utc_offset,
                is_capital: row.is_capital,
                population: row.population,
                latitude: row.latitude,
                longitude: row.longitude,
                feature_code: row.feature_code,
            });
        }
        Ok(results)
    }

    pub fn match_countries(&self, query: &str) -> Result<Vec<String>> {
        let has_cjk = query.chars().any(|c| c >= '\u{4E00}' && c <= '\u{9FFF}');
        let mut stmt = if has_cjk {
            self.conn.prepare("SELECT iso FROM countries WHERE name_cn LIKE ?1")?
        } else {
            self.conn.prepare("SELECT iso FROM countries WHERE name LIKE ?1 OR iso LIKE ?1")?
        };

        let pattern = format!("{}%", query);
        let rows = stmt.query_map(params![pattern], |row| {
            row.get::<_, String>(0)
        })?;

        let mut isos = Vec::new();
        for row in rows {
            if let Ok(iso) = row {
                isos.push(iso);
            }
        }
        Ok(isos)
    }

    pub fn get_country_cities(&self, country_iso: &str, limit: i64) -> Result<Option<CountryWithCities>> {
        let mut stmt = self.conn.prepare(
            "SELECT iso, name, name_cn, continent FROM countries WHERE iso = ?1"
        )?;

        let mut rows = stmt.query_map(params![country_iso], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, Option<String>>(3)?))
        })?;

        let (iso, name, name_cn, continent) = match rows.next() {
            Some(Ok(row)) => row,
            _ => return Ok(None),
        };

        let mut stmt = self.conn.prepare(
            "SELECT geonameid, name, name_cn, timezone, is_capital, feature_code
             FROM cities
             WHERE country_iso = ?1
             ORDER BY is_capital DESC, population DESC
             LIMIT ?2"
        )?;

        let city_rows = stmt.query_map(params![country_iso, limit], |row| {
            Ok(CitySummaryRow {
                geonameid: row.get(0)?,
                name: row.get(1)?,
                name_cn: row.get(2)?,
                timezone: row.get(3)?,
                is_capital: row.get::<_, i32>(4)? != 0,
                feature_code: row.get(5)?,
            })
        })?;

        let mut cities = Vec::new();
        for row in city_rows {
            let row = row?;
            let (local_time, _, _) = timecalc::get_local_time(&row.timezone);
            cities.push(CitySummary {
                geonameid: row.geonameid,
                name: row.name,
                name_cn: row.name_cn,
                timezone: row.timezone,
                local_time,
                is_capital: row.is_capital,
                feature_code: row.feature_code,
            });
        }

        Ok(Some(CountryWithCities {
            country_iso: iso,
            country_name: name,
            country_name_cn: name_cn,
            continent,
            cities,
        }))
    }
}

struct CityRow {
    geonameid: i64,
    name: String,
    name_cn: Option<String>,
    country_iso: String,
    timezone: String,
    population: i64,
    is_capital: bool,
    latitude: f64,
    longitude: f64,
    feature_code: Option<String>,
    country_name: String,
    country_name_cn: Option<String>,
}

struct CitySummaryRow {
    geonameid: i64,
    name: String,
    name_cn: Option<String>,
    timezone: String,
    is_capital: bool,
    feature_code: Option<String>,
}
