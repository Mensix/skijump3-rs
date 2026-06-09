use crate::rng::Random;

#[must_use]
pub fn sequential_schedule(hill_count: usize, max_count: usize) -> Vec<usize> {
    (0..hill_count.min(max_count)).collect()
}

#[must_use]
pub fn fixed_schedule<const N: usize>(hills: [usize; N]) -> Vec<usize> {
    hills.to_vec()
}

pub fn random_unique_schedule(hill_count: usize, count: usize, rng: &mut Random) -> Vec<usize> {
    if hill_count <= count {
        return sequential_schedule(hill_count, count);
    }

    let mut hills: Vec<usize> = (0..hill_count).collect();
    let mut schedule = Vec::with_capacity(count);
    for _ in 0..count {
        let idx = rng.random_i32(hills.len() as i32) as usize;
        schedule.push(hills.remove(idx));
    }
    schedule
}
