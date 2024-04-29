use chrono::{Utc, DateTime, FixedOffset};
use chrono::format::strftime::StrftimeItems;

#[allow(deprecated)]
pub fn get_format_datetime() -> String {
    let utc_time: DateTime<Utc> = Utc::now();
    let fixed_offset = FixedOffset::east(8 * 3600); // UTC+8 时区偏移量，单位为秒
    let local_time: DateTime<FixedOffset> = utc_time.with_timezone(&fixed_offset);

    let format = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
    local_time.format_with_items(format).to_string()
}


#[allow(deprecated)]
pub fn get_format_date() -> String {
    let utc_time: DateTime<Utc> = Utc::now();
    let fixed_offset = FixedOffset::east(8 * 3600); // UTC+8 时区偏移量，单位为秒
    let local_time: DateTime<FixedOffset> = utc_time.with_timezone(&fixed_offset);

    let format = StrftimeItems::new("%Y-%m-%d");
    local_time.format_with_items(format).to_string()
}
