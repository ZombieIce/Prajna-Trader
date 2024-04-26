use chrono::prelude::*;

pub fn get_datetime_from_timestamp(timestamp: i64) -> DateTime<Local> {
    let datetime = DateTime::from_timestamp_millis(timestamp).unwrap();
    datetime.with_timezone(&Local)
}

pub fn get_datetime_from_str(datetime_str: &str) -> DateTime<Local> {
    let dt = Local::now();
    let offset = *dt.offset();
    let naive_utc = NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S").unwrap();
    DateTime::<Local>::from_naive_utc_and_offset(naive_utc, offset)
}

pub fn get_now_string() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_datetime_from_timestamp() {
        let timestamp = 1713427200000; // Replace with your desired timestamp
        let expected_datetime = Local.with_ymd_and_hms(2024, 4, 18, 16, 0, 0).unwrap();

        let result = get_datetime_from_timestamp(timestamp);

        assert_eq!(result, expected_datetime);
    }

    #[test]
    fn test_get_datetime_from_str() {
        let datetime_str = "2024-04-18 08:00:00"; // Replace with your desired datetime string
        let expected_datetime = Local.with_ymd_and_hms(2024, 4, 18, 16, 0, 0).unwrap();

        let result = get_datetime_from_str(datetime_str);

        assert_eq!(result, expected_datetime);
    }
}
