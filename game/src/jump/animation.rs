const fn rust_sprite(pascal_anim: u16) -> u16 {
    pascal_anim.saturating_sub(1)
}

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
    rust_sprite(value + 71)
}

#[must_use]
pub const fn inrun_body_anim(ski_anim: u16) -> u16 {
    let ski = ski_anim.saturating_sub(rust_sprite(71));
    let value = match ski {
        2..=4 => 1,
        5..=6 => 2,
        7..=9 => 3,
        10..=12 => 4,
        _ => 0,
    };
    rust_sprite(value + 101)
}

#[must_use]
pub const fn inrun_transition_body_anim(counter: i32) -> u16 {
    rust_sprite((165 + counter / 7) as u16)
}

#[must_use]
pub const fn landing_body_anim(mut ski_anim: u16, landing_style: u8) -> u16 {
    if ski_anim >= rust_sprite(71) {
        ski_anim -= rust_sprite(71);
    }
    let mut value = match ski_anim {
        0..=4 => 127,
        5..=6 => 126,
        7 => 125,
        8..=12 => 124,
        _ => 127,
    };
    if landing_style == 2 {
        value += 6;
    }
    rust_sprite(value)
}

fn landing_loop_body_anim(counter: i32, slope_ski_anim: u16, landing_style: u8) -> u16 {
    // Pascal: if (counter<7) and (landing>0) then JumperAnim:=113+landing;
    if counter < 7 && landing_style > 0 {
        return rust_sprite(113 + u16::from(landing_style));
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
    let pascal_anim = match phase {
        0 => 122 + i32::from(landing_style) * 6,
        1 => 123 + i32::from(landing_style) * 6,
        2 => 136,
        3..=6 => match grade {
            0..=75 => 137,
            105..=200 => {
                if phase > 3 {
                    if grade > 114 {
                        141
                    } else {
                        140
                    }
                } else {
                    139
                }
            }
            _ => 136,
        },
        _ => 136,
    };
    rust_sprite(pascal_anim as u16)
}

#[must_use]
pub fn fall_body_anim(
    fall_type: u8,
    counter: i32,
    body_angle: i32,
    detached_slope_ski_anim: u16,
    landing_style: u8,
) -> u16 {
    let detached_ski = detached_slope_ski_anim.saturating_sub(rust_sprite(71));
    let pascal_anim = match fall_type {
        1 | 2 => {
            let mut extra = 2 - body_angle / 80;
            if extra < 0 {
                extra = 0;
            }
            let mut anim = 142 + counter / 10 + extra;
            if fall_type == 2 {
                anim = 142 + (counter - 6) / 10 + extra;
            }
            if anim > 145 {
                anim = match detached_ski {
                    4 => 145,
                    5 => 146,
                    6 => 147,
                    7..=12 => 148,
                    _ => 144,
                };
            }
            if fall_type == 2 && counter < 6 {
                return landing_loop_body_anim(counter, detached_slope_ski_anim, landing_style);
            }
            anim
        }
        3 if counter > 14 => {
            let mut anim = 151 + (counter - 14) / 10;
            if anim > 155 {
                anim = match detached_ski {
                    3..=4 => 162,
                    5..=6 => 161,
                    7..=12 => 160,
                    _ => 163,
                };
            } else if landing_style == 2 {
                anim += 5;
            }
            anim
        }
        _ => return landing_loop_body_anim(counter, detached_slope_ski_anim, landing_style),
    };
    rust_sprite(pascal_anim as u16)
}

pub const fn takeoff_body_anim(phase: &mut u8) -> u16 {
    let value = match *phase {
        4..=6 => 118,
        7..=9 => 119,
        10..=13 => 120,
        14..=17 => 121,
        18..=20 => 122,
        21..=23 => 123,
        24..=50 => 112,
        _ => 117,
    };
    *phase = phase.saturating_add(1);
    rust_sprite(value)
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
    rust_sprite(value + 106)
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
    rust_sprite(value + 71)
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
        assert_eq!(fall_body_anim(1, 10, 160, 74, 1), 142);
        assert_eq!(fall_body_anim(3, 24, 160, 76, 2), 156);
    }
}
