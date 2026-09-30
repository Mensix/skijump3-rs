use crate::gfx::sprites::Sprite;
use crate::jump::types::{FallType, JumpPhase, LandingStyle};

const SKI_SLOPE: u16 = 70;
const INRUN_BODY: u16 = 100;
const FLIGHT_BODY: u16 = 105;
const POST_LANDING_PHASE0: i32 = 121;
const POST_LANDING_PHASE1: i32 = 122;
const FALL_BASE: i32 = 141;
const CRASH_BASE: i32 = 150;
const CRASH_THRESHOLD: i32 = 154;

#[derive(Debug, Clone, Copy)]
pub(crate) struct JumperAnimationContext {
    pub(crate) phase: JumpPhase,
    pub(crate) frame: i32,
    pub(crate) bar_animation_counter: i32,
    pub(crate) takeoff_counter: u8,
    pub(crate) takeoff_phase: u8,
    pub(crate) body_angle: i32,
    pub(crate) ski_angle: i32,
    pub(crate) height: i32,
    pub(crate) travel: f64,
    pub(crate) slope_angle: i32,
    pub(crate) detached_slope_angle: i32,
    pub(crate) landing_counter: i32,
    pub(crate) start_anim: i32,
    pub(crate) landing_style: LandingStyle,
    pub(crate) fall_type: FallType,
    pub(crate) grade: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct JumperSpriteSelection {
    pub(crate) body: u16,
    pub(crate) skis: u16,
}

pub(crate) fn select_jumper_sprites(context: JumperAnimationContext) -> JumperSpriteSelection {
    let ski_slope_angle = if matches!(context.phase, JumpPhase::Flight)
        && context.height < 6
        && context.travel > 20.0
    {
        context.slope_angle / (context.height + 1)
    } else {
        context.slope_angle
    };
    let slope_ski = slope_ski_anim(ski_slope_angle);
    let detached_ski = slope_ski_anim(context.detached_slope_angle);
    let body = match context.phase {
        JumpPhase::Info | JumpPhase::Disqualified => Sprite::IdleBody as u16,
        JumpPhase::OnBar => bar_body_anim(context.bar_animation_counter),
        JumpPhase::Inrun => {
            if context.takeoff_counter > 0 {
                takeoff_body_anim(context.takeoff_phase)
            } else if context.frame < 28 {
                inrun_transition_body_anim(context.frame)
            } else {
                inrun_body_anim(slope_ski)
            }
        }
        JumpPhase::Flight => {
            if context.takeoff_counter > 0 && context.takeoff_phase < 25 {
                takeoff_body_anim(context.takeoff_phase)
            } else {
                flight_body_anim(context.body_angle)
            }
        }
        JumpPhase::Landing | JumpPhase::Result => {
            if context.fall_type != FallType::None {
                fall_body_anim(
                    context.fall_type,
                    context.landing_counter,
                    context.body_angle,
                    detached_ski,
                    context.landing_style,
                )
            } else {
                post_landing_body_anim(
                    context.landing_counter,
                    context.start_anim,
                    context.landing_style,
                    context.grade,
                    detached_ski,
                )
            }
        }
    };
    let skis = match context.phase {
        JumpPhase::Flight if context.height >= 6 || context.travel <= 20.0 => {
            flight_ski_anim(context.ski_angle)
        }
        _ => slope_ski,
    };
    JumperSpriteSelection { body, skis }
}

pub const fn bar_body_anim(counter: i32) -> u16 {
    let pascal_sprite = match counter {
        1000..=1012 | 1034..=1046 => 169,
        1013..=1033 => 170,
        1047..=1059 | 3038..=3200 => 164,
        1060..=1072 | 1094..=1106 => 171,
        1073..=1093 => 172,
        2000..=2012 | 2072..=2082 => 173,
        2013..=2025 | 2037..=2047 | 2059..=2071 => 174,
        2026..=2036 | 2048..=2058 => 175,
        3000..=3024 => 176,
        3025..=3037 => 177,
        _ => 164,
    };
    pascal_sprite - 1
}

pub const fn crash_risk(slope_angle: i32) -> i64 {
    let extra = match slope_angle {
        31 => 1,
        30 => 2,
        29 => 4,
        28 => 7,
        27 => 12,
        26 => 19,
        25 => 29,
        24 => 41,
        23 => 54,
        22 => 70,
        21 => 90,
        20 => 120,
        19 => 200,
        18 => 300,
        17 => 500,
        16 => 700,
        0..=15 => 950,
        _ => 0,
    };
    1 + extra
}

pub const fn slope_ski_anim(slope_angle: i32) -> u16 {
    let value = match slope_angle {
        4..=6 => 1,
        7..=9 => 2,
        10..=12 => 3,
        13..=16 => 4,
        17..=19 => 5,
        20..=24 => 6,
        25..=28 => 7,
        29..=33 => 8,
        34..=39 => 9,
        40..=48 => 10,
        49..=64 => 11,
        65..=90 => 12,
        _ => 0,
    };
    SKI_SLOPE + value
}

pub const fn inrun_body_anim(ski_anim: u16) -> u16 {
    let ski = ski_anim.saturating_sub(SKI_SLOPE);
    let value = match ski {
        2..=4 => 1,
        5..=6 => 2,
        7..=9 => 3,
        10..=12 => 4,
        _ => 0,
    };
    INRUN_BODY + value
}

pub const fn inrun_transition_body_anim(counter: i32) -> u16 {
    Sprite::InrunTransition as u16 + (counter / 7) as u16
}

pub const fn landing_body_anim(mut ski_anim: u16, landing_style: LandingStyle) -> u16 {
    if ski_anim >= SKI_SLOPE {
        ski_anim -= SKI_SLOPE;
    }
    let mut value = match ski_anim {
        0..=4 => Sprite::LandingBody1 as u16,
        5..=6 => Sprite::LandingBody2 as u16,
        7 => Sprite::LandingBody3 as u16,
        8..=12 => Sprite::LandingBody4 as u16,
        _ => Sprite::LandingBody1 as u16,
    };
    if matches!(landing_style, LandingStyle::TwoFooted) {
        value += 6;
    }
    value
}

fn landing_loop_body_anim(counter: i32, slope_ski_anim: u16, landing_style: LandingStyle) -> u16 {
    if counter < 7 && !matches!(landing_style, LandingStyle::None) {
        return (Sprite::LandingLoopBase as u16) + landing_style.offset() as u16;
    }
    landing_body_anim(slope_ski_anim, landing_style)
}

pub fn post_landing_body_anim(
    counter: i32,
    start_anim: i32,
    landing_style: LandingStyle,
    grade: i32,
    slope_ski_anim: u16,
) -> u16 {
    if counter <= start_anim {
        return landing_loop_body_anim(counter, slope_ski_anim, landing_style);
    }

    let phase = ((counter - start_anim) / 12).min(6);
    let anim_idx = match phase {
        0 => POST_LANDING_PHASE0 + landing_style.offset() * 6,
        1 => POST_LANDING_PHASE1 + landing_style.offset() * 6,
        2 => Sprite::PostLandingPhase2 as i32,
        3..=6 => match grade {
            0..=75 => Sprite::LandingSlide as i32,
            105..=200 => {
                if phase > 3 {
                    if grade > 114 {
                        Sprite::PostLandingRecoveryHigh as i32
                    } else {
                        Sprite::PostLandingRecoveryUp as i32
                    }
                } else {
                    Sprite::PostLandingRecovery as i32
                }
            }
            _ => Sprite::PostLandingPhase2 as i32,
        },
        _ => Sprite::PostLandingPhase2 as i32,
    };
    anim_idx as u16
}

pub(crate) fn fall_body_anim(
    fall_type: FallType,
    counter: i32,
    body_angle: i32,
    detached_slope_ski_anim: u16,
    landing_style: LandingStyle,
) -> u16 {
    let detached_ski = detached_slope_ski_anim.saturating_sub(SKI_SLOPE);
    let anim_idx = match fall_type {
        FallType::Normal | FallType::TwoFooted => {
            let mut extra = 2 - body_angle / 80;
            if extra < 0 {
                extra = 0;
            }
            let mut anim = FALL_BASE + counter / 10 + extra;
            if fall_type == FallType::TwoFooted {
                anim = FALL_BASE + (counter - 6) / 10 + extra;
            }
            if anim > 144 {
                anim = match detached_ski {
                    4 => Sprite::FallSki1 as i32,
                    5 => Sprite::FallSki2 as i32,
                    6 => Sprite::FallSki3 as i32,
                    7..=12 => Sprite::FallSki4 as i32,
                    _ => Sprite::FallDefault as i32,
                };
            }
            if fall_type == FallType::TwoFooted && counter < 6 {
                return landing_loop_body_anim(counter, detached_slope_ski_anim, landing_style);
            }
            anim
        }
        FallType::Crash if counter > 14 => {
            let mut anim = CRASH_BASE + (counter - 14) / 10;
            if anim > CRASH_THRESHOLD {
                anim = match detached_ski {
                    3..=4 => Sprite::CrashFinalNarrow as i32,
                    5..=6 => Sprite::CrashFinalMedium as i32,
                    7..=12 => Sprite::CrashFinalWide as i32,
                    _ => Sprite::CrashFinalDefault as i32,
                };
            } else if matches!(landing_style, LandingStyle::TwoFooted) {
                anim += 5;
            }
            anim
        }
        FallType::Crash => FALL_BASE + counter / 10,
        FallType::None => {
            return landing_loop_body_anim(counter, detached_slope_ski_anim, landing_style)
        }
    };
    anim_idx as u16
}

pub const fn takeoff_body_anim(phase: u8) -> u16 {
    match phase {
        4..=6 => Sprite::Takeoff1 as u16,
        7..=9 => Sprite::Takeoff2 as u16,
        10..=13 => Sprite::Takeoff3 as u16,
        14..=17 => Sprite::Takeoff4 as u16,
        18..=20 => Sprite::Takeoff5 as u16,
        21..=23 => Sprite::Takeoff6 as u16,
        24..=50 => Sprite::TakeoffArmsUp as u16,
        _ => Sprite::TakeoffDefault as u16,
    }
}

pub const fn flight_body_anim(body_angle: i32) -> u16 {
    let value = match body_angle {
        50..=61 => 1,
        62..=76 => 2,
        77..=95 => 3,
        96..=119 => 4,
        120..=149 => 5,
        150..=186 => 6,
        187..=1000 => 7,
        _ => 0,
    };
    FLIGHT_BODY + value
}

pub const fn step_flight_body_angle(body_angle: i32, increase: bool) -> i32 {
    if increase {
        match body_angle {
            ..=49 => 50,
            50..=61 => 62,
            62..=76 => 77,
            77..=95 => 96,
            96..=119 => 120,
            120..=149 => 150,
            150..=186 => 187,
            _ => 600,
        }
    } else {
        match body_angle {
            ..=50 => 0,
            51..=61 => 50,
            62..=76 => 61,
            77..=95 => 76,
            96..=119 => 95,
            120..=149 => 119,
            150..=186 => 149,
            _ => 186,
        }
    }
}

pub const fn flight_ski_anim(ski_angle: i32) -> u16 {
    let value = match ski_angle {
        -900..=-258 => 19,
        -257..=-216 => 18,
        -215..=-176 => 17,
        -175..=-136 => 16,
        -135..=-97 => 15,
        -96..=-59 => 14,
        -58..=-20 => 13,
        20..=58 => 1,
        59..=96 => 2,
        97..=135 => 3,
        136..=175 => 4,
        176..=215 => 5,
        216..=257 => 6,
        _ => 0,
    };
    SKI_SLOPE + value
}

pub(crate) const fn landing_height(slope_angle: i32) -> i32 {
    match slope_angle {
        0..=24 => 50,
        25 => 48,
        26 => 45,
        27 => 40,
        28 => 36,
        29 => 32,
        30 => 28,
        31 => 24,
        32 => 22,
        33..=39 => 20,
        40..=60 => 15,
        _ => 25,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_animation_matches_pascal_sprite_families() {
        assert_eq!(bar_body_anim(1000), 168);
        assert_eq!(bar_body_anim(1013), 169);
        assert_eq!(bar_body_anim(2000), 172);
        assert_eq!(bar_body_anim(2026), 174);
        assert_eq!(bar_body_anim(3000), 175);
        assert_eq!(bar_body_anim(3025), 176);
        assert_eq!(bar_body_anim(0), Sprite::IdleBody as u16);
    }

    #[test]
    fn maps_landing_and_fall_frames() {
        assert_eq!(
            post_landing_body_anim(101, 100, LandingStyle::Telemark, 0, 70),
            127
        );
        assert_eq!(
            post_landing_body_anim(125, 100, LandingStyle::TwoFooted, 120, 70),
            135
        );
        assert_eq!(
            post_landing_body_anim(150, 100, LandingStyle::TwoFooted, 120, 70),
            140
        );
        assert_eq!(
            fall_body_anim(FallType::Normal, 10, 160, 74, LandingStyle::Telemark),
            142
        );
        assert_eq!(
            fall_body_anim(FallType::Crash, 24, 160, 76, LandingStyle::TwoFooted),
            156
        );
    }
}
