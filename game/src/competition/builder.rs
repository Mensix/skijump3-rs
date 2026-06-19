use crate::competition::core::competitor::{
    active_profiles, computer_names_without_replacements, Competitor,
};
use crate::competition::core::schedule::{fixed_schedule, sequential_schedule};
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
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
) -> Competition {
    let participants = build_participants(profiles, computer_names, no_same_name);
    let hill_order = build_hill_order(style, hill_count);
    let mut c = Competition::new(style, participants, hill_order);
    c.training_rounds = training_rounds;
    c.ko_system = ko_system;
    c
}

/// Build a Competition with a custom hill order (for Custom Cup).
#[must_use]
pub fn build_custom_competition(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_order: Vec<usize>,
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
) -> Competition {
    let participants = build_participants(profiles, computer_names, no_same_name);
    let mut c = Competition::new(CupStyle::CustomCup, participants, hill_order);
    c.training_rounds = training_rounds;
    c.ko_system = ko_system;
    c
}

fn build_hill_order(style: CupStyle, hill_count: usize) -> Vec<usize> {
    match style {
        CupStyle::FourHills => fixed_schedule([8, 9, 10, 11]),
        _ => sequential_schedule(hill_count, TOTAL_SLOTS),
    }
}

fn build_participants(
    profiles: &ProfileStore,
    computer_names: &[String],
    no_same_name: bool,
) -> Vec<Participant> {
    let mut participants = Vec::with_capacity(TOTAL_SLOTS);
    let active_profiles = active_profiles(profiles);
    let profile_count = active_profiles.len().min(TOTAL_SLOTS);
    let first_profile_slot = TOTAL_SLOTS - profile_count;
    let computer_names = if no_same_name {
        computer_names_without_replacements(computer_names, &active_profiles)
    } else {
        computer_names.to_vec()
    };

    for i in 0..TOTAL_SLOTS {
        if i >= first_profile_slot {
            let list_idx = TOTAL_SLOTS - 1 - i;
            let (profile_idx, p) = &active_profiles[list_idx];
            let competitor = Competitor::from_profile(i, *profile_idx, p, None);
            participants.push(Participant {
                id: i,
                ai_id: competitor.ai_id,
                profile_idx: competitor.profile_idx,
                name: competitor.name,
                real_name: competitor.real_name,
                suit_color: competitor.suit_color,
                ski_color: competitor.ski_color,
                team: competitor.team,
                is_computer: competitor.is_computer,
                skip_quali: p.skip_quali as u8,
                wc_points: 0,
                four_hills_points: 0.0,
                injury: 0,
                points: None,
                rank: 0,
                round1_rank: 0,
                qual: QualificationStatus::NotQualified,
                round1_len: 0.0,
                round1_score: 0.0,
                round2_len: 0.0,
                qual_len: 0.0,
            });
        } else {
            let name = computer_names
                .get(i % computer_names.len().max(1))
                .cloned()
                .unwrap_or_else(|| format!("Computer {}", i + 1));
            let competitor = Competitor::computer(i, i, name, None);
            participants.push(Participant::from_competitor(competitor));
        }
    }

    participants
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::types::CompetitionJumpOutcome;
    use crate::data::profile::{Profile, ProfileStore};
    use crate::jump::types::FallType;

    #[test]
    fn builds_75_participants() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2, false, false);
        assert_eq!(comp.field.len(), 75);
    }

    #[test]
    fn user_profiles_are_last_and_not_computer() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2, false, false);
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
        profiles.profiles.push(Profile {
            name: "INACTIVE".into(),
            ..Profile::default()
        });
        let names = vec!["CPU".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, false, false);

        assert!(!comp.field.get(74).is_computer);
        assert_eq!(comp.field.get(74).name, "SKI JUMPER");
        assert!(comp.field.get(73).is_computer);
    }

    #[test]
    fn active_profile_replace_removes_computer_name() {
        let mut profiles = ProfileStore::new();
        profiles.profiles[0].replace = 1;
        let names = vec!["ROAR".into(), "ADAM".into(), "JANNE".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, false, false);

        assert_eq!(comp.field.get(0).name, "ADAM");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert_eq!(comp.field.get(1).name, "JANNE");
    }

    #[test]
    fn first_competition_qualification_order_matches_pascal() {
        let profiles = ProfileStore::new();
        let names = vec!["ROAR".into(), "ADAM".into()];
        let mut comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, false, false);

        comp.advance();
        assert_eq!(comp.current_jumper(), Some(74));

        let mut last = None;
        while let Some(idx) = comp.current_jumper() {
            last = Some(idx);
            comp.record_jump(CompetitionJumpOutcome {
                score: 0.0,
                distance: 0.0,
                fall_type: FallType::None,
            });
        }
        assert_eq!(last, Some(0));
        assert_eq!(comp.field.get(0).name, "ROAR");
    }

    #[test]
    fn four_hills_order_is_fixed() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::FourHills, &profiles, &names, 20, 0, false, false);
        assert_eq!(comp.hill_order, vec![8, 9, 10, 11]);
    }

    #[test]
    fn world_cup_uses_all_hills() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 10, 0, false, false);
        assert_eq!(comp.hill_order.len(), 10);
        assert_eq!(comp.hill_order[0], 0);
        assert_eq!(comp.hill_order[9], 9);
    }
}
