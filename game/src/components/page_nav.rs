#[must_use]
pub fn cycle_index(current: usize, len: usize, dir: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i64;
    let cur_i = current as i64;
    let result = (cur_i + i64::from(dir)).rem_euclid(len_i);
    result as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_forward() {
        assert_eq!(cycle_index(0, 5, 1), 1);
        assert_eq!(cycle_index(4, 5, 1), 0);
    }

    #[test]
    fn cycle_backward() {
        assert_eq!(cycle_index(0, 5, -1), 4);
        assert_eq!(cycle_index(4, 5, -1), 3);
    }

    #[test]
    fn empty_list() {
        assert_eq!(cycle_index(0, 0, 1), 0);
    }

    #[test]
    fn large_delta() {
        assert_eq!(cycle_index(0, 5, 7), 2);
        assert_eq!(cycle_index(0, 5, -7), 3);
    }
}
