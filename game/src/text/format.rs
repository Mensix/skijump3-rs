use std::fmt::Write;

#[must_use]
pub fn tenths_to_decimal(value: i32) -> f64 {
    let result = f64::from(value) / 10.0;

    (result * 10.0).round() / 10.0
}

#[must_use]
pub fn format_decimal(value: f64) -> String {
    format!("{value:.1}")
}

#[must_use]
pub fn format_tenths(value: i32) -> String {
    if value == 0 {
        return "0.0".to_string();
    }
    let mut buf = String::with_capacity(8);
    if value < 0 {
        buf.push('-');
    }
    let abs = value.unsigned_abs();
    write!(buf, "{}.{}", abs / 10, abs % 10).unwrap();
    buf
}

#[must_use]
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
