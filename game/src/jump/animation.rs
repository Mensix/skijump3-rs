use crate::gfx::sprites::Sprite;
use crate::jump::types::FallType;

const SKI_SLOPE: u16 = 70;
const INRUN_BODY: u16 = 100;
const FLIGHT_BODY: u16 = 105;
pub(crate) const INRUN_TRANSITION: u16 = 164;
const POST_LANDING_PHASE0: i32 = 121;
const POST_LANDING_PHASE1: i32 = 122;
const FALL_BASE: i32 = 141;
const CRASH_BASE: i32 = 150;
const CRASH_THRESHOLD: i32 = 154;

#[must_use]
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

#[must_use]
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

#[must_use]
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

#[must_use]
pub const fn inrun_transition_body_anim(counter: i32) -> u16 {
    INRUN_TRANSITION + (counter / 7) as u16
}

#[must_use]
pub const fn landing_body_anim(mut ski_anim: u16, landing_style: u8) -> u16 {
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
    if landing_style == 2 {
        value += 6;
    }
    value
}

fn landing_loop_body_anim(counter: i32, slope_ski_anim: u16, landing_style: u8) -> u16 {
    // Pascal: if (counter<7) and (landing>0) then JumperAnim:=113+landing;
    if counter < 7 && landing_style > 0 {
        return (Sprite::LandingLoopBase as u16) + u16::from(landing_style);
    }
    landing_body_anim(slope_ski_anim, landing_style)
}

#[must_use]
pub fn post_landing_body_anim(
    counter: i32,
    start_anim: i32,
    landing_style: u8,
    grade: i32,
    slope_ski_anim: u16,
) -> u16 {
    if counter <= start_anim {
        return landing_loop_body_anim(counter, slope_ski_anim, landing_style);
    }

    let phase = ((counter - start_anim) / 12).min(6);
    let anim_idx = match phase {
         0 => POST_LANDING_PHASE0 + i32::from(landing_style) * 6,
         1 => POST_LANDING_PHASE1 + i32::from(landing_style) * 6,
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

#[must_use]
pub(crate) fn fall_body_anim(
    fall_type: FallType,
    counter: i32,
    body_angle: i32,
    detached_slope_ski_anim: u16,
    landing_style: u8,
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
            } else if landing_style == 2 {
                anim += 5;
            }
            anim
        }
        FallType::Crash => FALL_BASE + counter / 10,
        FallType::None => return landing_loop_body_anim(counter, detached_slope_ski_anim, landing_style),
    };
    anim_idx as u16
}

pub const fn takeoff_body_anim(phase: &mut u8) -> u16 {
    let value = match *phase {
        4..=6 => Sprite::Takeoff1 as u16,
        7..=9 => Sprite::Takeoff2 as u16,
        10..=13 => Sprite::Takeoff3 as u16,
        14..=17 => Sprite::Takeoff4 as u16,
        18..=20 => Sprite::Takeoff5 as u16,
        21..=23 => Sprite::Takeoff6 as u16,
        24..=50 => Sprite::TakeoffArmsUp as u16,
        _ => Sprite::TakeoffDefault as u16,
    };
    *phase = phase.saturating_add(1);
    value
}

#[must_use]
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

#[must_use]
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

#[must_use]
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
    fn maps_inrun_ski_and_body_frames() {
        assert_eq!(slope_ski_anim(0), 70);
        assert_eq!(slope_ski_anim(4), 71);
        assert_eq!(slope_ski_anim(90), 82);
        assert_eq!(inrun_body_anim(70), 100);
        assert_eq!(inrun_body_anim(72), 101);
        assert_eq!(inrun_body_anim(82), 104);
    }

    #[test]
    fn maps_takeoff_and_flight_frames() {
        let mut phase = 0;
        assert_eq!(takeoff_body_anim(&mut phase), 116);
        phase = 4;
        assert_eq!(takeoff_body_anim(&mut phase), 117);
        phase = 24;
        assert_eq!(takeoff_body_anim(&mut phase), 111);

        assert_eq!(flight_body_anim(49), 105);
        assert_eq!(flight_body_anim(50), 106);
        assert_eq!(flight_body_anim(187), 112);
        assert_eq!(flight_ski_anim(-258), 89);
        assert_eq!(flight_ski_anim(20), 71);
    }

    #[test]
    fn maps_landing_height() {
        assert_eq!(landing_height(24), 50);
        assert_eq!(landing_height(25), 48);
        assert_eq!(landing_height(60), 15);
    }

    #[test]
    fn maps_landing_thresholds_and_risk() {
        assert_eq!(crash_risk(31), 2);
        assert_eq!(crash_risk(16), 701);
        assert_eq!(crash_risk(15), 951);
    }

    #[test]
    fn maps_landing_and_fall_frames() {
        assert_eq!(post_landing_body_anim(101, 100, 1, 0, 70), 127);
        assert_eq!(post_landing_body_anim(125, 100, 2, 120, 70), 135);
        assert_eq!(post_landing_body_anim(150, 100, 2, 120, 70), 140);
        assert_eq!(fall_body_anim(FallType::Normal, 10, 160, 74, 1), 142);
        assert_eq!(fall_body_anim(FallType::Crash, 24, 160, 76, 2), 156);
    }
}
