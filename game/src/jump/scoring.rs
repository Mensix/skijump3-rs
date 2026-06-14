use crate::data::hill_profile::HillTerrain;
use crate::jump::animation::crash_risk;
use crate::jump::math;
use crate::jump::types::{FallType, LandingStyle};
use crate::rng::Random;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandingRisk {
    pub(crate) quality: i32,
    pub(crate) risk: i32,
    pub(crate) fall_type: FallType,
    pub(crate) style_penalty: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreResult {
    pub(crate) style_base: i32,
    pub(crate) style_points: [i32; 5],
    pub(crate) score: i32,
}

pub fn landing_risk(
    terrain: &HillTerrain,
    x: i32,
    distance: f64,
    hill_kr: i32,
    body_angle: i32,
    landing_style: LandingStyle,
) -> LandingRisk {
    let slope_angle = terrain.hill_angle(x);
    let quality = math::round(f64::from(slope_angle).mul_add(1.34, f64::from(body_angle) / 10.0));
    let mut risk = crash_risk(slope_angle) as i32;
    if distance < (20.0 / 3.0) * f64::from(hill_kr) {
        risk = 1;
    }
    if quality < 63 {
        risk = math::round(f64::from(risk) * (1.0 + f64::from(63 - quality) * 0.075));
    }

    let mut fall_type = FallType::None;
    let mut style_penalty = 0;
    if landing_style == LandingStyle::None || quality < 56 {
        fall_type = if landing_style == LandingStyle::None {
            FallType::Normal
        } else {
            FallType::TwoFooted
        };
    }
    if landing_style == LandingStyle::Telemark {
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

pub fn calculate_score(
    style_base: i32,
    hill_kr: i32,
    distance: f64,
    fall_type: FallType,
    landing_style: LandingStyle,
    rng: &mut Random,
) -> ScoreResult {
    let mut base = style_base;
    let short_jump_penalty_count =
        math::round((f64::from(hill_kr) + f64::from(hill_kr) / 20.0 - distance) / 6.0);
    if short_jump_penalty_count > 0 {
        base -= short_jump_penalty_count * 5;
    }

    if fall_type != FallType::None {
        base -= 100;
    } else if landing_style == LandingStyle::TwoFooted {
        base -= 15 + rng.random_i32(2) * 5;
    }

    let mut style_points = [0; 5];
    style_points[0] = base;
    for point in style_points.iter_mut().skip(1) {
        let offset = rng.random_i32(4);
        *point = base - (offset - 1) * 5;
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
        score += math::round(
            (distance - (f64::from(hill_kr) * 2.0 / 3.0)) * (180.0 / f64::from(hill_kr)) * 10.0,
        );
    }

    ScoreResult {
        style_base: base,
        style_points,
        score,
    }
}
