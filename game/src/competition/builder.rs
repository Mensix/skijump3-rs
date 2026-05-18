use crate::competition::machine::Competition;
use crate::competition::types::{CupStyle, Participant, QualificationStatus};
use crate::data::profile::ProfileStore;

const TOTAL_SLOTS: usize = 75;

/// Build a Competition from game state.
#[must_use]
pub fn build_competition(
    style: CupStyle,
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_count: usize,
    trainrounds: usize,
) -> Competition {
    let participants = build_participants(profiles, computer_names);
    let hill_order = build_hill_order(style, hill_count);
    let mut c = Competition::new(style, participants, hill_order);
    c.trainrounds = trainrounds;
    c
}

fn build_hill_order(style: CupStyle, hill_count: usize) -> Vec<usize> {
    match style {
        CupStyle::FourHills => vec![8, 9, 10, 11],
        _ => (0..hill_count.min(TOTAL_SLOTS)).collect(),
    }
}

fn build_participants(profiles: &ProfileStore, computer_names: &[String]) -> Vec<Participant> {
    let mut participants = Vec::with_capacity(TOTAL_SLOTS);
    let active_profiles: Vec<_> = profiles
        .active_order
        .iter()
        .copied()
        .filter_map(|idx| profiles.profiles.get(idx))
        .collect();
    let profile_count = active_profiles.len().min(TOTAL_SLOTS);
    let first_profile_slot = TOTAL_SLOTS - profile_count;
    let computer_names = computer_names_without_replacements(computer_names, &active_profiles);

    for i in 0..TOTAL_SLOTS {
        if i >= first_profile_slot {
            let profile_idx = TOTAL_SLOTS - 1 - i;
            let p = active_profiles[profile_idx];
            participants.push(Participant {
                id: i,
                ai_id: 0,
                name: p.name.clone(),
                real_name: p.real_name.clone(),
                suit_color: p.suit_color as u8,
                ski_color: p.ski_color as u8,
                team: None,
                is_computer: false,
                skip_quali: p.skip_quali > 0,
                wc_points: 0,
                four_hills_points: 0,
                injury: 0,
                points: 0,
                rank: 0,
                qual: QualificationStatus::NotQualified,
                round1_len: 0,
                round2_len: 0,
                qual_len: 0,
            });
        } else {
            let name = computer_names
                .get(i % computer_names.len().max(1))
                .cloned()
                .unwrap_or_else(|| format!("Computer {}", i + 1));
            participants.push(Participant::computer(i, i, name));
        }
    }

    participants
}

fn computer_names_without_replacements(
    computer_names: &[String],
    active_profiles: &[&crate::data::profile::Profile],
) -> Vec<String> {
    computer_names
        .iter()
        .enumerate()
        .filter(|(idx, _)| {
            let replace = idx + 1;
            !active_profiles
                .iter()
                .any(|profile| profile.replace == replace)
        })
        .map(|(_, name)| name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::profile::ProfileStore;

    #[test]
    fn builds_75_participants() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2);
        assert_eq!(comp.field.len(), 75);
    }

    #[test]
    fn user_profiles_are_last_and_not_computer() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2);
        assert!(comp.field.get(0).is_computer);
        assert_eq!(comp.field.get(0).name, "AAA");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert_eq!(comp.field.get(1).name, "BBB");
        assert_eq!(comp.field.get(1).ai_id, 1);
        assert!(!comp.field.get(74).is_computer);
        assert_eq!(comp.field.get(74).name, "SKI JUMPER");
    }

    #[test]
    fn only_active_profiles_join_competition() {
        let mut profiles = ProfileStore::new();
        profiles.profiles.push(crate::data::profile::Profile {
            name: "INACTIVE".into(),
            ..crate::data::profile::Profile::default()
        });
        let names = vec!["CPU".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0);

        assert!(!comp.field.get(74).is_computer);
        assert_eq!(comp.field.get(74).name, "SKI JUMPER");
        assert!(comp.field.get(73).is_computer);
    }

    #[test]
    fn active_profile_replace_removes_computer_name() {
        let mut profiles = ProfileStore::new();
        profiles.profiles[0].replace = 1;
        let names = vec!["ROAR".into(), "ADAM".into(), "JANNE".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0);

        assert_eq!(comp.field.get(0).name, "ADAM");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert_eq!(comp.field.get(1).name, "JANNE");
    }

    #[test]
    fn first_competition_qualification_order_matches_pascal() {
        let profiles = ProfileStore::new();
        let names = vec!["ROAR".into(), "ADAM".into()];
        let mut comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0);

        comp.advance();
        assert_eq!(comp.current_jumper(), Some(74));

        let mut last = None;
        while let Some(idx) = comp.current_jumper() {
            last = Some(idx);
            comp.record_jump(0, 0);
        }
        assert_eq!(last, Some(0));
        assert_eq!(comp.field.get(0).name, "ROAR");
    }

    #[test]
    fn four_hills_order_is_fixed() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::FourHills, &profiles, &names, 20, 0);
        assert_eq!(comp.hill_order, vec![8, 9, 10, 11]);
    }

    #[test]
    fn world_cup_uses_all_hills() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 10, 0);
        assert_eq!(comp.hill_order.len(), 10);
        assert_eq!(comp.hill_order[0], 0);
        assert_eq!(comp.hill_order[9], 9);
    }
}
