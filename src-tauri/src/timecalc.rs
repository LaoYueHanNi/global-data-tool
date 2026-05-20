use chrono::Utc;
use chrono_tz::Tz;

pub fn get_local_time(timezone_id: &str) -> (String, String, String) {
    let tz: Tz = timezone_id.parse().expect("Invalid timezone");
    let now_utc = Utc::now();
    let local = now_utc.with_timezone(&tz);
    let time_str = local.format("%H:%M:%S").to_string();
    let date_str = local.format("%Y-%m-%d").to_string();
    let offset_str = local.format("%:z").to_string();
    (time_str, date_str, offset_str)
}
