use std::fmt::Write;

/// Format an integer representing tenths as a decimal string.
/// `1195` → `"119.5"`, `0` → `"0.0"`, `-5` → `"-0.5"`.
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

/// Format an i64 as tenths, delegating to `format_tenths`.
/// Values outside i32 range are safe (no narrowing cast).
pub fn format_tenths_i64(value: i64) -> String {
    if value == 0 {
        return "0.0".to_string();
    }
    let mut buf = String::with_capacity(16);
    if value < 0 {
        buf.push('-');
    }
    let abs = value.unsigned_abs();
    write!(buf, "{}.{}", abs / 10, abs % 10).unwrap();
    buf
}

/// `"5."`, `"12."`, etc.
pub fn ordinal_dot(n: usize) -> String {
    format!("{n}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero() {
        assert_eq!(format_tenths(0), "0.0");
    }

    #[test]
    fn positive_integer() {
        assert_eq!(format_tenths(1234), "123.4");
    }

    #[test]
    fn positive_with_carry() {
        assert_eq!(format_tenths(100), "10.0");
    }

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

    #[test]
    fn i64_roundtrip() {
        assert_eq!(format_tenths_i64(1234), "123.4");
        assert_eq!(format_tenths_i64(-15), "-1.5");
        assert_eq!(format_tenths_i64(0), "0.0");
    }

    #[test]
    fn ordinal() {
        assert_eq!(ordinal_dot(0), "0.");
        assert_eq!(ordinal_dot(5), "5.");
        assert_eq!(ordinal_dot(12), "12.");
    }
}
