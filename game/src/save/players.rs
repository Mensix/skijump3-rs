use crate::data::profile::{Profile, ProfileStore};
use crate::save::crypt::profile_code;

use super::write_lines;

#[must_use] 
pub fn profiles_to_bytes(store: &ProfileStore) -> Vec<u8> {
    let mut out = Vec::new();

    out.extend(format!("{}\n", store.profiles.len()).as_bytes());

    for (idx, profile) in store.profiles.iter().enumerate() {
        out.extend(format!("*{}\n", idx + 1).as_bytes());
        out.extend(profile_to_lines(profile));
    }

    out
}

fn profile_to_lines(profile: &Profile) -> Vec<u8> {
    let code = profile_code(
        &profile.name,
        &profile.best_result,
        &profile.best_4h_result,
        profile.suit_color as u8,
        profile.ski_color as u8,
        profile.koth_level as u8,
        profile.replace as u8,
        profile.bestwchill as u8,
        profile.best_wc_jump as u16,
        profile.besthill_idx as u8,
        profile.best_jump as u16,
        profile.bestpoints as u16,
        profile.best4points as u16,
        profile.coach_style as u8,
        profile.world_cups as i32,
        profile.legs_won as i32,
        profile.world_cups_won as i32,
        profile.total_jumps as i32,
    );

    // File format (matching Pascal WriteProfiles):
    //   name, suitcolor, skicolor, cstyle, kothlevel, replace,
    //   wcs, legswon, wcswon, bestwcjump, bestwchill, bestjump, besthill,
    //   besthillfile, bestresult, bestpoints, best4result, best4points,
    //   totaljumps, skipquali, realname, 0, 0, profilecode, \n, \n
    let lines = vec![
        profile.name.clone(),
        profile.suit_color.to_string(),
        profile.ski_color.to_string(),
        profile.coach_style.to_string(),
        profile.koth_level.to_string(),
        profile.replace.to_string(),
        profile.world_cups.to_string(),
        profile.legs_won.to_string(),
        profile.world_cups_won.to_string(),
        profile.best_wc_jump.to_string(),
        profile.bestwchill.to_string(),
        profile.best_jump.to_string(),
        profile.besthill_idx.to_string(),
        profile.besthillfile.clone(),
        profile.best_result.clone(),
        profile.bestpoints.to_string(),
        profile.best_4h_result.clone(),
        profile.best4points.to_string(),
        profile.total_jumps.to_string(),
        profile.skip_quali.to_string(),
        profile.real_name.clone(),
        "0".to_string(),
        "0".to_string(),
        code.to_string(),
        String::new(),
        String::new(),
    ];

    let mut out = Vec::new();
    write_lines(&mut out, &lines);
    out
}

impl super::SaveFormat for ProfileStore {
    fn to_bytes(&self) -> Vec<u8> {
        profiles_to_bytes(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_example_file() {
        let mut store = ProfileStore::new();
        let p = &mut store.profiles[0];
        p.suit_color = 0;
        p.ski_color = 0;
        p.coach_style = 1;
        p.koth_level = 0;
        p.replace = 0;
        p.world_cups = 0;
        p.legs_won = 0;
        p.world_cups_won = 0;
        p.best_wc_jump = 930;
        p.bestwchill = 1;
        p.best_jump = 930;
        p.besthill_idx = 1;
        p.besthillfile = "HILLBASE".to_string();
        p.best_result = "0 (-)".to_string();
        p.bestpoints = 0;
        p.best_4h_result = "-".to_string();
        p.best4points = 0;
        p.total_jumps = 1;
        p.skip_quali = 2;

        let got = profiles_to_bytes(&store);
        let expected = include_bytes!("../../assets/PLAYERS.SKI");

        assert_eq!(
            got.as_slice(),
            expected as &[u8],
            "output does not match reference PLAYERS.SKI"
        );
    }
}
