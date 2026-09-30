use std::fmt::Write;

pub fn display_timestamp_now() -> String {
    chrono::Local::now().format("%a %e %b %Y %k:%M").to_string()
}

pub fn format_time(timestamp: &str) -> String {
    if timestamp.trim().is_empty() {
        return "-".to_string();
    }
    let secs: u64 = timestamp.parse().unwrap_or(0);
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%a %e %b %Y %k:%M")
                .to_string()
        })
        .unwrap_or_default()
}

pub fn current_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(|_| String::new(), |d| d.as_secs().to_string())
}

pub fn tenths_to_decimal(value: i32) -> f64 {
    let result = f64::from(value) / 10.0;

    (result * 10.0).round() / 10.0
}

pub fn format_decimal(value: f64) -> String {
    format!("{value:.1}")
}

pub fn format_tenths(value: i32) -> String {
    if value == 0 {
        return "0.0".to_string();
    }
    let mut buf = String::with_capacity(8);
    if value < 0 {
        buf.push('-');
    }
    let abs = value.unsigned_abs();
    let _ = write!(buf, "{}.{}", abs / 10, abs % 10);
    buf
}

pub fn ordinal_dot(n: usize) -> String {
    format!("{n}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_digit() {
        assert_eq!(format_tenths(5), "0.5");
    }

    #[test]
    fn negative_tenths() {
        assert_eq!(format_tenths(-15), "-1.5");
    }

    #[test]
    fn negative_single() {
        assert_eq!(format_tenths(-3), "-0.3");
    }
}
