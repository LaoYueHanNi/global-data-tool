"""Build SQLite database from GeoNames data files."""
import gzip
import os
import shutil
import sqlite3

from opencc import OpenCC

RAW_DIR = os.path.join(os.path.dirname(__file__), "raw_data")
DB_PATH = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "data", "cities.db")

cc = OpenCC("t2s")  # 繁体转简体

# ── 数据过滤规则 ──────────────────────────────────────────────
# GeoNames 数据中港澳台地区被标记为独立的 ISO 代码，
# 此处将其纠正为中国（CN），确保搜索"中国"时包含这些城市。
REGION_REMAP = {
    "HK": "CN",  # 香港 → 中国
    "MO": "CN",  # 澳门 → 中国
    "TW": "CN",  # 台湾 → 中国
}

def correct_country_code(code: str) -> str:
    """纠正港澳台地区的 ISO 归属为 CN，其他地区不变。"""
    return REGION_REMAP.get(code, code)


def create_tables(conn: sqlite3.Connection) -> None:
    """Create database tables and FTS5 index."""
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS countries (
            iso          TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            name_cn      TEXT,
            continent    TEXT
        );

        CREATE TABLE IF NOT EXISTS cities (
            geonameid    INTEGER PRIMARY KEY,
            name         TEXT NOT NULL,
            name_cn      TEXT,
            country_iso  TEXT NOT NULL,
            latitude     REAL NOT NULL,
            longitude    REAL NOT NULL,
            timezone     TEXT NOT NULL,
            population   INTEGER NOT NULL,
            is_capital   INTEGER NOT NULL DEFAULT 0,
            feature_code TEXT
        );

        CREATE TABLE IF NOT EXISTS city_names (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            geonameid    INTEGER NOT NULL,
            lang         TEXT,
            name         TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_cities_country ON cities(country_iso);
        CREATE INDEX IF NOT EXISTS idx_cities_pop ON cities(population DESC);
        CREATE INDEX IF NOT EXISTS idx_city_names_geonameid ON city_names(geonameid);
        CREATE INDEX IF NOT EXISTS idx_city_names_lang ON city_names(lang);
    """)


def load_countries(conn: sqlite3.Connection) -> dict:
    """Load country data and return country_geonameid_to_iso mapping."""
    country_file = os.path.join(RAW_DIR, "countryInfo.txt")
    geonameid_to_iso = {}

    print("Loading countries...")
    count = 0
    with open(country_file, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            parts = line.strip().split("\t")
            if len(parts) < 17:
                continue

            iso = parts[0]
            name = parts[4]
            continent = parts[8]
            geonameid = int(parts[16]) if parts[16] else None

            conn.execute(
                "INSERT OR REPLACE INTO countries (iso, name, continent) VALUES (?, ?, ?)",
                (iso, name, continent)
            )

            if geonameid:
                geonameid_to_iso[geonameid] = iso

            count += 1

    print(f"  Loaded {count} countries")
    return geonameid_to_iso


def load_cities(conn: sqlite3.Connection) -> set:
    """Load city data and return set of geonameids."""
    cities_file = os.path.join(RAW_DIR, "cities5000.txt")
    geonameids = set()

    print("Loading cities...")
    count = 0
    with open(cities_file, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split("\t")
            if len(parts) < 19:
                continue

            geonameid = int(parts[0])
            name = parts[1]
            latitude = float(parts[4])
            longitude = float(parts[5])
            feature_class = parts[6]
            feature_code = parts[7]
            original_country = parts[8]
            country_code = correct_country_code(original_country)
            population = int(parts[14]) if parts[14] else 0
            timezone = parts[17]

            if feature_class != "P":
                continue

            is_capital = 1 if feature_code == "PPLC" else 0

            # 港澳台城市纠正归属后，移除其首都标记（如台北不再标记为中国首都）
            if original_country in REGION_REMAP:
                is_capital = 0
                if feature_code == "PPLC":
                    feature_code = None

            conn.execute(
                "INSERT OR REPLACE INTO cities (geonameid, name, country_iso, latitude, longitude, timezone, population, is_capital, feature_code) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (geonameid, name, country_code, latitude, longitude, timezone, population, is_capital, feature_code)
            )

            geonameids.add(geonameid)
            count += 1

    print(f"  Loaded {count} cities")
    return geonameids


def load_alternate_names(conn: sqlite3.Connection, valid_geonameids: set, country_geonameid_to_iso: dict) -> tuple:
    """Load alternate names and return (city_chinese_names, country_chinese_names)."""
    alt_file = os.path.join(RAW_DIR, "alternateNamesV2.txt")
    chinese_names = {}
    country_chinese = {}
    all_valid_ids = valid_geonameids | set(country_geonameid_to_iso.keys())

    print("Loading alternate names...")
    count = 0
    with open(alt_file, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split("\t")
            if len(parts) < 4:
                continue

            geonameid = int(parts[1])
            lang = parts[2]
            alt_name = parts[3]
            is_preferred = len(parts) > 4 and parts[4] == "1"
            is_historic = len(parts) > 7 and parts[7] == "1"

            if is_historic or not lang:
                continue

            if lang in ("zh", "zh-CN", "zh-Hant"):
                if geonameid in valid_geonameids:
                    if geonameid not in chinese_names or (lang == "zh-CN" and is_preferred):
                        chinese_names[geonameid] = alt_name
                    conn.execute(
                        "INSERT INTO city_names (geonameid, lang, name) VALUES (?, ?, ?)",
                        (geonameid, lang, alt_name)
                    )
                elif geonameid in country_geonameid_to_iso:
                    iso = country_geonameid_to_iso[geonameid]
                    if iso not in country_chinese or (lang == "zh-CN" and is_preferred):
                        country_chinese[iso] = alt_name
            elif lang == "en":
                if geonameid in valid_geonameids:
                    conn.execute(
                        "INSERT INTO city_names (geonameid, lang, name) VALUES (?, ?, ?)",
                        (geonameid, lang, alt_name)
                    )

            count += 1

    print(f"  Processed {count} alternate names")
    print(f"  Found Chinese names for {len(chinese_names)} cities")
    print(f"  Found Chinese names for {len(country_chinese)} countries")
    return chinese_names, country_chinese


def update_chinese_names(conn: sqlite3.Connection, chinese_names: dict, country_chinese: dict) -> None:
    """Update cities and countries with Chinese names (converted to simplified)."""
    print("Converting traditional to simplified and updating...")

    for geonameid, name_cn in chinese_names.items():
        simplified = cc.convert(name_cn)
        conn.execute(
            "UPDATE cities SET name_cn = ? WHERE geonameid = ?",
            (simplified, geonameid)
        )

    for iso, name_cn in country_chinese.items():
        simplified = cc.convert(name_cn)
        conn.execute(
            "UPDATE countries SET name_cn = ? WHERE iso = ?",
            (simplified, iso)
        )


def build_fts_index(conn: sqlite3.Connection) -> None:
    """Build FTS5 search index."""
    print("Building FTS5 index...")
    conn.executescript("""
        CREATE VIRTUAL TABLE IF NOT EXISTS city_search USING fts5(
            name,
            name_cn,
            geonameid UNINDEXED,
            country_iso UNINDEXED,
            content='cities',
            content_rowid='geonameid'
        );

        INSERT INTO city_search(rowid, name, name_cn, geonameid, country_iso)
        SELECT geonameid, name, name_cn, geonameid, country_iso FROM cities;
    """)


def optimize_database(conn: sqlite3.Connection) -> None:
    """Optimize database with ANALYZE and VACUUM."""
    print("Optimizing database...")
    conn.executescript("""
        INSERT INTO city_search(city_search) VALUES('rebuild');
        ANALYZE;
    """)
    conn.commit()
    conn.execute("VACUUM")


def main():
    os.makedirs(os.path.dirname(DB_PATH), exist_ok=True)

    if os.path.exists(DB_PATH):
        os.remove(DB_PATH)

    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA journal_mode=WAL")

    create_tables(conn)
    geonameid_to_iso = load_countries(conn)
    valid_ids = load_cities(conn)
    chinese_names, country_chinese = load_alternate_names(conn, valid_ids, geonameid_to_iso)
    update_chinese_names(conn, chinese_names, country_chinese)
    build_fts_index(conn)
    optimize_database(conn)

    conn.close()

    file_size = os.path.getsize(DB_PATH) / (1024 * 1024)
    print(f"\nDatabase built successfully: {DB_PATH}")
    print(f"File size: {file_size:.1f} MB")

    gz_path = DB_PATH + ".gz"
    with open(DB_PATH, "rb") as f_in, gzip.open(gz_path, "wb") as f_out:
        shutil.copyfileobj(f_in, f_out)
    gz_size = os.path.getsize(gz_path) / (1024 * 1024)
    print(f"Compressed: {gz_path} ({gz_size:.1f} MB)")

    print("\nNext step: pnpm tauri dev")


if __name__ == "__main__":
    main()
