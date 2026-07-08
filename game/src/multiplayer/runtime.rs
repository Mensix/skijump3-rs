use net::protocol::{JumpRound, MPStandingEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultiplayerPhase {
    RoundJumping,
    RoundResults,
    FinalResults,
}

#[derive(Debug, Clone)]
pub struct MultiplayerRuntime {
    pub(crate) phase: MultiplayerPhase,
    pub(crate) round: usize,
    pub(crate) hill_idx: usize,
    pub(crate) total_legs: usize,
    pub(crate) start_gate: i32,
    pub(crate) entries: Vec<MPStandingEntry>,
    pub(crate) my_player_id: usize,
}

impl MultiplayerRuntime {
    pub(crate) fn new(
        hill_idx: usize,
        total_legs: usize,
        start_gate: i32,
        entries: Vec<MPStandingEntry>,
        my_player_id: usize,
    ) -> Self {
        Self {
            phase: MultiplayerPhase::RoundJumping,
            round: 0,
            hill_idx,
            total_legs: total_legs.max(1),
            start_gate,
            entries,
            my_player_id,
        }
    }

    pub(crate) fn apply_round(&mut self, round: &JumpRound) {
        self.phase = MultiplayerPhase::RoundJumping;
        self.round = round.round;
        self.hill_idx = round.hill_idx;
        self.start_gate = round.start_gate;
    }

    pub(crate) fn apply_results(&mut self, round: usize, entries: Vec<MPStandingEntry>) {
        self.round = round;
        self.entries = entries;
        self.phase = if self.round == 0 {
            MultiplayerPhase::RoundResults
        } else {
            MultiplayerPhase::FinalResults
        };
    }

    pub(crate) fn sync_standings(&mut self, round: usize, entries: Vec<MPStandingEntry>) {
        self.round = round;
        self.entries = entries;
        if self.round_complete() {
            self.phase = if self.round == 0 {
                MultiplayerPhase::RoundResults
            } else {
                MultiplayerPhase::FinalResults
            };
        } else {
            self.phase = MultiplayerPhase::RoundJumping;
        }
    }

    pub(crate) fn apply_jump_result(
        &mut self,
        player_id: usize,
        distance: f64,
        score: f64,
    ) -> bool {
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.player_id == player_id)
        else {
            return false;
        };

        if self.round == 0 {
            if entry.round1_len > 0.0 || entry.round1_score > 0.0 {
                return false;
            }
            entry.round1_len = distance;
            entry.round1_score = score;
        } else {
            if entry.round2_len > 0.0 || entry.round2_score > 0.0 {
                return false;
            }
            entry.round2_len = distance;
            entry.round2_score = score;
        }
        entry.total_points = entry.round1_score + entry.round2_score;
        if self.round_complete() {
            self.phase = if self.round == 0 {
                MultiplayerPhase::RoundResults
            } else {
                MultiplayerPhase::FinalResults
            };
        }
        true
    }

    pub(crate) fn can_local_jump(&self) -> bool {
        self.phase == MultiplayerPhase::RoundJumping && !self.player_has_jump(self.my_player_id)
    }

    pub(crate) fn round_complete(&self) -> bool {
        !self.entries.is_empty()
            && self.entries.iter().all(|entry| {
                if self.round == 0 {
                    entry.round1_len > 0.0 || entry.round1_score > 0.0
                } else {
                    entry.round2_len > 0.0 || entry.round2_score > 0.0
                }
            })
    }

    pub(crate) fn next_round(&mut self, wind_seed: u32, wind_position: u8) -> Option<JumpRound> {
        if self.phase != MultiplayerPhase::RoundResults || !self.round_complete() {
            return None;
        }
        self.round = 1;
        self.phase = MultiplayerPhase::RoundJumping;
        Some(JumpRound {
            hill_idx: self.hill_idx,
            round: self.round,
            wind_seed,
            wind_position,
            start_gate: self.start_gate,
        })
    }

    pub(crate) fn next_leg(&mut self, wind_seed: u32, wind_position: u8) -> Option<JumpRound> {
        if self.phase != MultiplayerPhase::FinalResults || self.hill_idx + 1 >= self.total_legs {
            return None;
        }
        self.hill_idx += 1;
        self.round = 0;
        self.phase = MultiplayerPhase::RoundJumping;
        for entry in &mut self.entries {
            entry.round1_len = 0.0;
            entry.round1_score = 0.0;
            entry.round2_len = 0.0;
            entry.round2_score = 0.0;
            entry.total_points = 0.0;
        }
        Some(JumpRound {
            hill_idx: self.hill_idx,
            round: self.round,
            wind_seed,
            wind_position,
            start_gate: self.start_gate,
        })
    }

    fn player_has_jump(&self, player_id: usize) -> bool {
        self.entries
            .iter()
            .find(|entry| entry.player_id == player_id)
            .is_some_and(|entry| {
                if self.round == 0 {
                    entry.round1_len > 0.0 || entry.round1_score > 0.0
                } else {
                    entry.round2_len > 0.0 || entry.round2_score > 0.0
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(player_id: usize) -> MPStandingEntry {
        MPStandingEntry {
            player_id,
            name: format!("P{player_id}"),
            round1_len: 0.0,
            round1_score: 0.0,
            round2_len: 0.0,
            round2_score: 0.0,
            total_points: 0.0,
        }
    }

    #[test]
    fn round_results_after_all_players_jump() {
        let mut runtime = MultiplayerRuntime::new(0, 20, 15, vec![entry(0), entry(1)], 0);

        assert!(runtime.apply_jump_result(0, 120.0, 100.0));
        assert_eq!(runtime.phase, MultiplayerPhase::RoundJumping);

        assert!(runtime.apply_jump_result(1, 121.0, 101.0));
        assert_eq!(runtime.phase, MultiplayerPhase::RoundResults);
    }

    #[test]
    fn host_advance_moves_to_round_two_only_from_results() {
        let mut runtime = MultiplayerRuntime::new(0, 20, 15, vec![entry(0)], 0);
        assert!(runtime.next_round(1, 2).is_none());

        runtime.apply_jump_result(0, 120.0, 100.0);
        let round = runtime.next_round(1, 2).unwrap();

        assert_eq!(round.round, 1);
        assert_eq!(runtime.phase, MultiplayerPhase::RoundJumping);
        assert!(runtime.can_local_jump());
    }

    #[test]
    fn round_two_completion_is_final_results() {
        let mut runtime = MultiplayerRuntime::new(0, 20, 15, vec![entry(0)], 0);
        runtime.apply_jump_result(0, 120.0, 100.0);
        runtime.next_round(1, 2);
        runtime.apply_jump_result(0, 121.0, 101.0);

        assert_eq!(runtime.phase, MultiplayerPhase::FinalResults);
    }

    #[test]
    fn final_results_can_advance_to_next_leg() {
        let mut runtime = MultiplayerRuntime::new(0, 2, 15, vec![entry(0)], 0);
        runtime.apply_jump_result(0, 120.0, 100.0);
        runtime.next_round(1, 2);
        runtime.apply_jump_result(0, 121.0, 101.0);

        let round = runtime.next_leg(3, 4).unwrap();

        assert_eq!(round.hill_idx, 1);
        assert_eq!(round.round, 0);
        assert_eq!(runtime.phase, MultiplayerPhase::RoundJumping);
        assert_eq!(runtime.entries[0].total_points, 0.0);
    }
}
