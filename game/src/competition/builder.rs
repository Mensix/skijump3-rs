use crate::competition::core::competitor::{
    active_profiles, computer_names_without_replacements, Competitor,
};
use crate::competition::core::schedule::{fixed_schedule, sequential_schedule};
use crate::competition::machine::Competition;
use crate::competition::types::{CupStyle, CustomCupScoring, Participant};
use crate::data::profile::ProfileStore;

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

pub fn build_custom_competition(
    profiles: &ProfileStore,
    computer_names: &[String],
    hill_order: Vec<usize>,
    training_rounds: usize,
    no_same_name: bool,
    ko_system: bool,
    scoring: CustomCupScoring,
) -> Competition {
    let participants = build_participants(profiles, computer_names, no_same_name);
    let mut c = Competition::new(CupStyle::CustomCup, participants, hill_order);
    c.training_rounds = training_rounds;
    c.ko_system = ko_system;
    c.custom_cup_scoring = scoring;
    c
}

fn build_hill_order(style: CupStyle, hill_count: usize) -> Vec<usize> {
    match style {
        CupStyle::FourHills => fixed_schedule([8, 9, 10, 11]),
        _ => sequential_schedule(hill_count, hill_count),
    }
}

fn build_participants(
    profiles: &ProfileStore,
    computer_names: &[String],
    no_same_name: bool,
) -> Vec<Participant> {
    let active_profiles = active_profiles(profiles);
    let profile_count = active_profiles.len();
    let total_slots = computer_names.len().max(profile_count);
    let first_profile_slot = total_slots - profile_count.min(total_slots);
    let computer_names = if no_same_name {
        computer_names_without_replacements(computer_names, &active_profiles)
    } else {
        computer_names.to_vec()
    };

    let mut participants = Vec::with_capacity(total_slots);

    for i in 0..total_slots {
        if i >= first_profile_slot {
            let list_idx = total_slots - 1 - i;
            let (profile_idx, p) = &active_profiles[list_idx];
            let competitor = Competitor::from_profile(i, *profile_idx, p, None);
            let mut participant = Participant::from_competitor(competitor);
            participant.skip_qualification = p.skip_qualification as u8;
            participants.push(participant);
        } else {
            let name = computer_names
                .get(i)
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
    fn builds_all_participants() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2, false, false);
        assert_eq!(comp.field.len(), 2);
    }

    #[test]
    fn user_profiles_are_last_and_not_computer() {
        let profiles = ProfileStore::new();
        let names = vec!["AAA".into(), "BBB".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2, false, false);
        assert!(comp.field.get(0).is_computer);
        assert_eq!(comp.field.get(0).name, "AAA");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert!(!comp.field.get(1).is_computer);
        assert_eq!(comp.field.get(1).name, "SKI JUMPER");
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

        assert_eq!(comp.field.len(), 1);
        assert!(!comp.field.get(0).is_computer);
        assert_eq!(comp.field.get(0).name, "SKI JUMPER");
    }

    #[test]
    fn active_profile_replace_none_keeps_all_computers() {
        let mut profiles = ProfileStore::new();
        profiles.profiles[0].replace = None;
        let names = vec!["ROAR".into(), "ADAM".into(), "JANNE".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, true, false);

        assert_eq!(comp.field.get(0).name, "ROAR");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert_eq!(comp.field.get(1).name, "ADAM");
        assert_eq!(comp.field.get(1).ai_id, 1);
    }

    #[test]
    fn active_profile_replace_some_removes_computer_name() {
        let mut profiles = ProfileStore::new();
        profiles.profiles[0].replace = Some(0);
        let names = vec!["ROAR".into(), "ADAM".into(), "JANNE".into()];

        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, true, false);

        assert_eq!(comp.field.get(0).name, "ADAM");
        assert_eq!(comp.field.get(0).ai_id, 0);
        assert_eq!(comp.field.get(1).name, "JANNE");
    }

    #[test]
    fn first_competition_qualification_order_matches_pascal() {
        let profiles = ProfileStore::new();
        let names = vec!["ROAR".into(), "ADAM".into()];
        let mut comp =
            build_competition(CupStyle::WorldCup, &profiles, &names, 20, 0, false, false);

        comp.advance();
        assert_eq!(comp.current_jumper(), Some(1));

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
