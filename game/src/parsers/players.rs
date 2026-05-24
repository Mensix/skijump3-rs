use crate::data::profile::{Profile, ProfileStore};
use crate::parsers::ParseError;
use crate::text::encoding;

/// Parse PLAYERS.SKI (Pascal format) into a ProfileStore.
/// Line endings are normalized; each line is CP850-decoded.
pub struct PlayersParser;

fn decode_line(bytes: &[u8]) -> String {
    let trimmed = trim_line(bytes);
    encoding::decode(trimmed)
}

fn trim_line(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}

fn parse_usize(s: &str) -> usize {
    s.trim().parse().unwrap_or(0)
}

impl PlayersParser {
    pub fn parse(data: &[u8]) -> Result<ProfileStore, ParseError> {
        let mut lines: Vec<String> = data.split(|&b| b == b'\n').map(decode_line).collect();

        // Remove trailing empty lines
        while let Some(last) = lines.last() {
            if last.is_empty() {
                lines.pop();
            } else {
                break;
            }
        }

        if lines.is_empty() {
            return Err(ParseError {
                message: "PLAYERS.SKI is empty".to_string(),
                byte_offset: None,
            });
        }

        let count: usize = lines[0].trim().parse().map_err(|_| ParseError {
            message: format!("Invalid profile count: {}", lines[0]),
            byte_offset: None,
        })?;

        if count == 0 || count > 20 {
            return Err(ParseError {
                message: format!("Profile count out of range: {count}"),
                byte_offset: None,
            });
        }

        let mut profiles = Vec::with_capacity(count);
        let mut idx = 1;

        for _ in 0..count {
            // Expect "*N" marker
            if idx >= lines.len() {
                return Err(ParseError {
                    message: "Unexpected end of PLAYERS.SKI".to_string(),
                    byte_offset: None,
                });
            }

            let marker = lines[idx].trim();
            if !marker.starts_with('*') {
                return Err(ParseError {
                    message: format!("Expected profile marker, got: {marker}"),
                    byte_offset: None,
                });
            }
            idx += 1;

            let read_str = |i: &mut usize| -> String {
                if *i < lines.len() {
                    let v = lines[*i].clone();
                    *i += 1;
                    v
                } else {
                    String::new()
                }
            };

            let name = read_str(&mut idx);
            let suit_color = parse_usize(&read_str(&mut idx));
            let ski_color = parse_usize(&read_str(&mut idx));
            let coach_style = parse_usize(&read_str(&mut idx));
            let koth_level = parse_usize(&read_str(&mut idx));
            let replace = parse_usize(&read_str(&mut idx));
            let world_cups = parse_usize(&read_str(&mut idx));
            let legs_won = parse_usize(&read_str(&mut idx));
            let world_cups_won = parse_usize(&read_str(&mut idx));
            let best_wc_jump = parse_usize(&read_str(&mut idx));
            let bestwchill = parse_usize(&read_str(&mut idx));
            let best_jump = parse_usize(&read_str(&mut idx));
            let besthill_idx = parse_usize(&read_str(&mut idx));
            let besthillfile = read_str(&mut idx);
            let best_result = read_str(&mut idx);
            let bestpoints = parse_usize(&read_str(&mut idx));
            let best_4h_result = read_str(&mut idx);
            let best4points = parse_usize(&read_str(&mut idx));
            let total_jumps = parse_usize(&read_str(&mut idx));
            let skip_quali = parse_usize(&read_str(&mut idx));
            let real_name = read_str(&mut idx);
            let _reserved1 = read_str(&mut idx); // "0"
            let _reserved2 = read_str(&mut idx); // "0"
            let _code = read_str(&mut idx); // checksum, ignored for now

            profiles.push(Profile {
                name,
                real_name,
                suit_color,
                ski_color,
                replace,
                coach_style,
                skip_quali,
                total_jumps,
                world_cups,
                legs_won,
                world_cups_won,
                best_result,
                best_4h_result,
                best_wc_jump,
                bestwchill,
                best_jump,
                besthill_idx,
                besthillfile,
                bestpoints,
                best4points,
                koth_level,
                best_wc_hill_display: String::new(),
                best_hill_display: String::new(),
            });
        }

        Ok(ProfileStore::from_profiles(profiles))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::players::profiles_to_bytes;

    #[test]
    fn parse_default_players_file() {
        let data = include_bytes!("../../assets/PLAYERS.SKI");
        let store = PlayersParser::parse(data).expect("parse bundled PLAYERS.SKI");
        // Default file has 1 profile
        assert_eq!(store.profiles.len(), 1);
        let p = &store.profiles[0];
        assert_eq!(p.name, "SKI JUMPER");
        assert!(p.real_name.is_empty());
        assert_eq!(p.suit_color, 0);
        assert_eq!(p.ski_color, 0);
        assert_eq!(p.coach_style, 1);
        assert_eq!(p.koth_level, 0);
        assert_eq!(p.replace, 0);
        assert_eq!(p.world_cups, 0);
        assert_eq!(p.legs_won, 0);
        assert_eq!(p.world_cups_won, 0);
        assert_eq!(p.best_wc_jump, 930);
        assert_eq!(p.bestwchill, 1);
        assert_eq!(p.best_jump, 930);
        assert_eq!(p.besthill_idx, 1);
        assert_eq!(p.besthillfile, "HILLBASE");
        assert_eq!(p.best_result, "0 (-)");
        assert_eq!(p.bestpoints, 0);
        assert_eq!(p.best_4h_result, "-");
        assert_eq!(p.best4points, 0);
        assert_eq!(p.total_jumps, 1);
        assert_eq!(p.skip_quali, 2);
    }

    #[test]
    fn roundtrip_preserves_all_profiles() {
        let original = include_bytes!("../../assets/PLAYERS.SKI");
        let store = PlayersParser::parse(original).expect("parse bundled PLAYERS.SKI");

        // Roundtrip through profiles_to_bytes and re-parse
        let rewritten = profiles_to_bytes(&store);
        let reparsed = PlayersParser::parse(&rewritten).expect("re-parse rewritten PLAYERS.SKI");

        assert_eq!(
            store.profiles.len(),
            reparsed.profiles.len(),
            "profile count mismatch"
        );

        for (i, (a, b)) in store
            .profiles
            .iter()
            .zip(reparsed.profiles.iter())
            .enumerate()
        {
            assert_eq!(a.name, b.name, "profile {i}: name mismatch");
            assert_eq!(a.real_name, b.real_name, "profile {i}: real_name mismatch");
            assert_eq!(
                a.suit_color, b.suit_color,
                "profile {i}: suit_color mismatch"
            );
            assert_eq!(a.ski_color, b.ski_color, "profile {i}: ski_color mismatch");
            assert_eq!(
                a.coach_style, b.coach_style,
                "profile {i}: coach_style mismatch"
            );
            assert_eq!(
                a.koth_level, b.koth_level,
                "profile {i}: koth_level mismatch"
            );
            assert_eq!(a.replace, b.replace, "profile {i}: replace mismatch");
            assert_eq!(
                a.world_cups, b.world_cups,
                "profile {i}: world_cups mismatch"
            );
            assert_eq!(a.legs_won, b.legs_won, "profile {i}: legs_won mismatch");
            assert_eq!(
                a.world_cups_won, b.world_cups_won,
                "profile {i}: world_cups_won mismatch"
            );
            assert_eq!(
                a.best_wc_jump, b.best_wc_jump,
                "profile {i}: best_wc_jump mismatch"
            );
            assert_eq!(
                a.bestwchill, b.bestwchill,
                "profile {i}: bestwchill mismatch"
            );
            assert_eq!(a.best_jump, b.best_jump, "profile {i}: best_jump mismatch");
            assert_eq!(
                a.besthill_idx, b.besthill_idx,
                "profile {i}: besthill_idx mismatch"
            );
            assert_eq!(
                a.besthillfile, b.besthillfile,
                "profile {i}: besthillfile mismatch"
            );
            assert_eq!(
                a.best_result, b.best_result,
                "profile {i}: best_result mismatch"
            );
            assert_eq!(
                a.bestpoints, b.bestpoints,
                "profile {i}: bestpoints mismatch"
            );
            assert_eq!(
                a.best_4h_result, b.best_4h_result,
                "profile {i}: best_4h_result mismatch"
            );
            assert_eq!(
                a.best4points, b.best4points,
                "profile {i}: best4points mismatch"
            );
            assert_eq!(
                a.total_jumps, b.total_jumps,
                "profile {i}: total_jumps mismatch"
            );
            assert_eq!(
                a.skip_quali, b.skip_quali,
                "profile {i}: skip_quali mismatch"
            );
        }
    }
}
