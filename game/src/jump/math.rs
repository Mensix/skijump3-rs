pub(crate) fn pascal_round(value: f64) -> i32 {
    if value >= 0.0 {
        (value + 0.5).floor() as i32
    } else {
        (value - 0.5).ceil() as i32
    }
}

pub(crate) fn nsqrt(value: f64) -> f64 {
    let root = value.abs().sqrt();
    if value < 0.0 {
        -root
    } else {
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_half_away_from_zero() {
        assert_eq!(pascal_round(0.5), 1);
        assert_eq!(pascal_round(1.5), 2);
        assert_eq!(pascal_round(2.5), 3);
        assert_eq!(pascal_round(37.5), 38);
        assert_eq!(pascal_round(38.5), 39);
        assert_eq!(pascal_round(-0.5), -1);
        assert_eq!(pascal_round(-1.5), -2);
        assert_eq!(pascal_round(0.0), 0);
        assert_eq!(pascal_round(0.1), 0);
        assert_eq!(pascal_round(0.9), 1);
    }
}
