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
        CupStyle::FourHills => vec![9, 10, 11, 12],
        _ => (1..=hill_count.min(TOTAL_SLOTS)).collect(),
    }
}

fn build_participants(profiles: &ProfileStore, computer_names: &[String]) -> Vec<Participant> {
    let mut participants = Vec::with_capacity(TOTAL_SLOTS);

    for (i, p) in profiles.profiles.iter().enumerate() {
        participants.push(Participant {
            id: i,
            name: p.name.clone(),
            real_name: p.real_name.clone(),
            suit_color: p.suit_color as u8,
            ski_color: p.ski_color as u8,
            team: None,
            is_computer: false,
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
    }

    for i in profiles.profiles.len()..TOTAL_SLOTS {
        let name = computer_names
            .get(i % computer_names.len().max(1))
            .cloned()
            .unwrap_or_else(|| format!("Computer {}", i + 1));
        participants.push(Participant::computer(i, name));
    }

    participants
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
    fn user_profiles_are_first_and_not_computer() {
        let profiles = ProfileStore::new();
        let names = vec!["CPU".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 20, 2);
        assert!(!comp.field.get(0).is_computer);
        assert_eq!(comp.field.get(0).name, "SKI JUMPER");
        assert!(comp.field.get(1).is_computer);
        assert_eq!(comp.field.get(1).name, "CPU");
    }

    #[test]
    fn four_hills_order_is_fixed() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::FourHills, &profiles, &names, 20, 0);
        assert_eq!(comp.hill_order, vec![9, 10, 11, 12]);
    }

    #[test]
    fn world_cup_uses_all_hills() {
        let profiles = ProfileStore::new();
        let names = vec!["X".into()];
        let comp = build_competition(CupStyle::WorldCup, &profiles, &names, 10, 0);
        assert_eq!(comp.hill_order.len(), 10);
        assert_eq!(comp.hill_order[0], 1);
        assert_eq!(comp.hill_order[9], 10);
    }
}
