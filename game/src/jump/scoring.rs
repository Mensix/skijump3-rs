use crate::data::hill_profile::HillTerrain;
use crate::jump::animation::crash_risk;
use crate::jump::math::pascal_round;
use crate::rng::Random;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LandingRisk {
    pub(crate) quality: i32,
    pub(crate) risk: i32,
    pub(crate) fall_type: u8,
    pub(crate) style_penalty: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ScoreResult {
    pub(crate) style_base: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) score: i32,
}

pub(crate) fn landing_risk(
    terrain: &HillTerrain,
    x: i32,
    distance: i32,
    hill_kr: i32,
    body_angle: i32,
    landing_style: u8,
) -> LandingRisk {
    let slope_angle = terrain.hill_angle(x);
    let quality = pascal_round(f64::from(slope_angle) * 1.34 + f64::from(body_angle) / 10.0);
    let mut risk = crash_risk(slope_angle) as i32;
    if f64::from(distance) < (20.0 / 3.0) * f64::from(hill_kr) {
        risk = 1;
    }
    if quality < 63 {
        risk = pascal_round(f64::from(risk) * (1.0 + f64::from(63 - quality) * 0.075));
    }

    let mut fall_type = 0;
    let mut style_penalty = 0;
    if landing_style == 0 || quality < 56 {
        fall_type = if landing_style == 0 { 1 } else { 2 };
    }
    if landing_style == 1 {
        risk *= 3;
        if quality < 60 {
            style_penalty += 5;
        }
        if quality < 64 {
            style_penalty += 5;
        }
    }

    LandingRisk {
        quality,
        risk,
        fall_type,
        style_penalty,
    }
}

pub(crate) fn calculate_score(
    style_base: i32,
    hill_kr: i32,
    distance: i32,
    fall_type: u8,
    landing_style: u8,
    rng: &mut Random,
) -> ScoreResult {
    let mut base = style_base;
    let short_jump_penalty_count = pascal_round(
        (f64::from(hill_kr) + f64::from(hill_kr) / 20.0 - (f64::from(distance) / 10.0)) / 6.0,
    );
    if short_jump_penalty_count > 0 {
        base -= short_jump_penalty_count * 5;
    }

    if fall_type > 0 {
        base -= 100;
    } else if landing_style == 2 {
        base -= 15 + rng.random_i32(2) * 5;
    }

    let mut style_points = [0; 5];
    style_points[0] = base;
    for point in style_points.iter_mut().skip(1) {
        let temp = rng.random_i32(4);
        *point = base - (temp - 1) * 5;
    }

    let mut min_style = 200;
    let mut max_style = 0;
    for point in &mut style_points {
        *point = (*point).clamp(0, 200);
        min_style = min_style.min(*point);
        max_style = max_style.max(*point);
    }

    let mut score = style_points.iter().sum::<i32>() - min_style - max_style;
    if hill_kr != 0 {
        score += pascal_round(
            ((f64::from(distance) / 10.0) - (f64::from(hill_kr) * 2.0 / 3.0))
                * (180.0 / f64::from(hill_kr))
                * 10.0,
        );
    }

    ScoreResult {
        style_base: base,
        style_points,
        score,
    }
}
