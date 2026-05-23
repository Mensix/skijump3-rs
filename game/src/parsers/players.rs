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
        let mut lines: Vec<String> = data
            .split(|&b| b == b'\n')
            .map(decode_line)
            .collect();

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
