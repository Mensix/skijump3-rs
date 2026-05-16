use crate::competition::field::{CompetitionField, SortBy};
use crate::competition::scoring;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};

const QUALIFICATION_SPOTS: usize = 50;
const ROUND2_SPOTS: usize = 30;

/// Drives a single competition event (or a full season).
///
/// Call `advance()` to enter the first phase, then loop:
/// 1. `current_jumper()` → who's up (None = phase transition needed)
/// 2. `is_human_current()` → wait for input or auto-pilot
/// 3. `record_jump(points, length)` → store result
/// 4. `advance()` → move to next jumper or next phase
/// 5. `is_over()` → season finished?
#[derive(Debug, Clone)]
pub struct Competition {
    pub field: CompetitionField,
    pub style: CupStyle,
    pub hill_order: Vec<usize>,
    pub current_event: usize,
    pub phase: CompetitionPhase,
    pub trainrounds: usize,

    start_list: Vec<usize>,
    start_pos: usize,
}

impl Competition {
    pub fn new(style: CupStyle, participants: Vec<Participant>, hill_order: Vec<usize>) -> Self {
        Self {
            field: CompetitionField::new(participants),
            style,
            hill_order,
            current_event: 0,
            phase: CompetitionPhase::Setup,
            trainrounds: 2,
            start_list: Vec::new(),
            start_pos: 0,
        }
    }

    // ── queries ────────────────────────────────────────────────

    pub fn current_jumper(&self) -> Option<usize> {
        if self.start_pos < self.start_list.len() {
            Some(self.start_list[self.start_pos])
        } else {
            None
        }
    }

    pub fn is_human_current(&self) -> bool {
        self.current_jumper()
            .map(|idx| !self.field.get(idx).is_computer)
            .unwrap_or(false)
    }

    pub fn is_over(&self) -> bool {
        self.phase == CompetitionPhase::SeasonComplete
    }

    pub fn phase_progress(&self) -> (usize, usize) {
        (self.start_pos, self.start_list.len())
    }

    /// Number of events in the season.
    pub fn total_events(&self) -> usize {
        self.hill_order.len()
    }

    /// Participants in event-points order (for results lists).
    pub fn event_standings(&self) -> Vec<&Participant> {
        self.field
            .event_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    /// Participants in season-points order (for WC standings).
    pub fn overall_standings(&self) -> Vec<&Participant> {
        self.field
            .master_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    // ── drive ──────────────────────────────────────────────────

    /// Advance to the next state. Call after recording a jump or
    /// when entering the machine for the first time.
    pub fn advance(&mut self) {
        match self.phase {
            CompetitionPhase::SeasonComplete => {}

            CompetitionPhase::Setup => self.enter_setup(),
            CompetitionPhase::Training(n) => {
                if self.start_pos >= self.start_list.len() {
                    let next = n + 1;
                    if next as usize <= self.trainrounds {
                        self.enter_phase(CompetitionPhase::Training(next));
                    } else {
                        self.enter_phase(CompetitionPhase::Qualification);
                    }
                }
            }
            CompetitionPhase::Qualification => {
                if self.start_pos >= self.start_list.len() {
                    self.resolve_qualification();
                    self.enter_phase(CompetitionPhase::Round1);
                }
            }
            CompetitionPhase::Round1 => {
                if self.start_pos >= self.start_list.len() {
                    self.cut_to_round2();
                    self.enter_phase(CompetitionPhase::Round2);
                }
            }
            CompetitionPhase::Round2 => {
                if self.start_pos >= self.start_list.len() {
                    self.field.sort_field(SortBy::EventPoints);
                    self.enter_phase(CompetitionPhase::Results);
                }
            }
            CompetitionPhase::Results => {
                self.award_points();
                self.enter_phase(CompetitionPhase::EventComplete);
            }
            CompetitionPhase::EventComplete => {
                self.current_event += 1;
                if self.current_event >= self.hill_order.len() {
                    self.phase = CompetitionPhase::SeasonComplete;
                } else {
                    self.enter_phase(CompetitionPhase::Setup);
                }
            }
        }
    }

    /// Store a jump result for the current participant and advance
    /// to the next jumper within the current phase (without phase
    /// transition — call `advance()` separately for that).
    pub fn record_jump(&mut self, jump_points: i32, length: i32) {
        let Some(&idx) = self.start_list.get(self.start_pos) else {
            return;
        };
        match self.phase {
            CompetitionPhase::Training(_) | CompetitionPhase::Qualification => {
                self.field.get_mut(idx).points = jump_points;
                self.field.get_mut(idx).qual_len = length;
            }
            CompetitionPhase::Round1 => {
                self.field.get_mut(idx).points = jump_points;
                self.field.get_mut(idx).round1_len = length;
            }
            CompetitionPhase::Round2 => {
                self.field.get_mut(idx).points += jump_points;
                self.field.get_mut(idx).round2_len = length;
            }
            _ => {}
        }
        self.start_pos += 1;
    }

    /// Prepare standings for a result-list screen without entering the next phase.
    pub fn prepare_display_list(&mut self) {
        if self.current_jumper().is_some() {
            return;
        }

        match self.phase {
            CompetitionPhase::Qualification => self.resolve_qualification(),
            CompetitionPhase::Round1 => self.cut_to_round2(),
            CompetitionPhase::Round2 => self.field.sort_field(SortBy::EventPoints),
            _ => {}
        }
    }

    // ── internal ───────────────────────────────────────────────

    fn enter_phase(&mut self, phase: CompetitionPhase) {
        self.phase = phase;
        self.start_pos = 0;
        self.start_list = self.field.build_start_list(phase);

        // Only jump phases can be skipped when there is nobody to jump.
        // Non-jump phases (Results/EventComplete) must be observable by UI.
        if self.start_list.is_empty()
            && matches!(
                phase,
                CompetitionPhase::Training(_)
                    | CompetitionPhase::Qualification
                    | CompetitionPhase::Round1
                    | CompetitionPhase::Round2
            )
        {
            self.advance();
        }
    }

    fn enter_setup(&mut self) {
        self.field.reset_event();
        self.field.tick_injuries();
        self.field.sort_field(SortBy::WcPoints);

        // Top 10 in overall WC classification skip qualification
        if self.current_event > 0 {
            for idx in 0..self.field.len() {
                if self.field.get(idx).rank <= 10 && self.field.get(idx).injury == 0 {
                    self.field.get_mut(idx).qual = QualificationStatus::PreQualified;
                }
            }
        }

        if self.trainrounds > 0 {
            self.enter_phase(CompetitionPhase::Training(1));
        } else {
            self.enter_phase(CompetitionPhase::Qualification);
        }
    }

    fn resolve_qualification(&mut self) {
        self.field.sort_field(SortBy::EventPoints);

        let pre_qualified = self
            .field
            .iter()
            .filter(|p| p.qual == QualificationStatus::PreQualified)
            .count();

        let spots = QUALIFICATION_SPOTS.saturating_sub(pre_qualified);
        let mut taken = 0usize;

        for idx in self.field.event_order.clone() {
            let p = self.field.get(idx);
            if p.injury > 0 || p.qual == QualificationStatus::PreQualified {
                continue;
            }
            if taken < spots {
                self.field.get_mut(idx).qual = QualificationStatus::Qualified;
                taken += 1;
            } else {
                self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
            }
        }
    }

    fn cut_to_round2(&mut self) {
        self.field.sort_field(SortBy::EventPoints);
        for idx in 0..self.field.len() {
            let rank = self.field.get(idx).rank;
            if rank <= ROUND2_SPOTS && self.field.get(idx).injury == 0 {
                self.field.get_mut(idx).qual = QualificationStatus::Qualified;
            } else {
                self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
            }
        }
    }

    fn award_points(&mut self) {
        match self.style {
            CupStyle::WorldCup => {
                scoring::award_wc_points(&mut self.field);
            }
            CupStyle::FourHills => {
                for idx in 0..self.field.len() {
                    let pts = self.field.get(idx).points;
                    if pts != -5555 {
                        self.field.get_mut(idx).four_hills_points += pts;
                    }
                }
            }
            CupStyle::CustomCup => {
                for idx in 0..self.field.len() {
                    self.field.get_mut(idx).four_hills_points += self.field.get(idx).points;
                }
            }
            CupStyle::TeamCup => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_50_participants() -> Vec<Participant> {
        (0..50)
            .map(|i| Participant::computer(i, format!("J {i}")))
            .collect()
    }

    fn make_season(hills: usize) -> Competition {
        let mut c = Competition::new(
            CupStyle::WorldCup,
            make_50_participants(),
            (0..hills).collect(),
        );
        c.trainrounds = 0;
        c
    }

    fn run_all_jumps(machine: &mut Competition) {
        let mut guard = 0;
        while !machine.is_over() {
            guard += 1;
            assert!(guard < 100, "competition state machine did not terminate");

            if machine.current_jumper().is_none() {
                machine.advance();
                continue;
            }

            while machine.current_jumper().is_some() {
                let pts = 150 + (machine.start_pos as i32 % 100);
                let len = 80 + (machine.start_pos as i32 % 50);
                machine.record_jump(pts, len);
            }
        }
    }

    #[test]
    fn single_event_full_cycle() {
        let mut m = make_season(1);
        run_all_jumps(&mut m);
        assert!(m.is_over());

        // Winners should have WC points
        let winner = m.field.get(0);
        assert!(winner.wc_points > 0, "winner should have WC points");
        assert_eq!(winner.rank, 1);

        // Everyone should have a valid rank
        for i in 0..m.field.len() {
            assert!(
                m.field.get(i).rank >= 1 && m.field.get(i).rank <= 50,
                "participant {i} has invalid rank {}",
                m.field.get(i).rank
            );
        }
    }

    #[test]
    fn three_event_season() {
        let mut m = make_season(3);
        run_all_jumps(&mut m);
        assert!(m.is_over());
        assert_eq!(m.current_event, 3);

        // After 3 events, season points should be higher than single event
        let total: i32 = (0..m.field.len()).map(|i| m.field.get(i).wc_points).sum();
        assert!(total > 0, "total WC points should be positive");
    }

    #[test]
    fn pre_qualification_works() {
        let mut m = make_season(2);

        // Event 1: give specific scores
        while m.phase != CompetitionPhase::EventComplete {
            if m.current_jumper().is_none() {
                m.advance();
                continue;
            }
            while let Some(idx) = m.current_jumper() {
                m.record_jump(200 - idx as i32, 90);
            }
        }

        // Advance to event 2 setup
        m.advance();
        m.advance();
        let pre_qualified = (0..m.field.len())
            .filter(|&i| m.field.get(i).qual == QualificationStatus::PreQualified)
            .count();
        assert_eq!(pre_qualified, 10);
    }

    #[test]
    fn top_30_advance_to_round2() {
        let mut m = make_season(1);
        let mut total_r1 = 0usize;
        let mut total_r2 = 0usize;

        let mut guard = 0;
        while m.phase != CompetitionPhase::Results {
            guard += 1;
            assert!(guard < 20, "competition did not reach results");

            if m.current_jumper().is_none() {
                m.advance();
                continue;
            }
            while let Some(_) = m.current_jumper() {
                match m.phase {
                    CompetitionPhase::Round1 => total_r1 += 1,
                    CompetitionPhase::Round2 => total_r2 += 1,
                    _ => {}
                }
                let score = match m.phase {
                    CompetitionPhase::Round1 => 200 - total_r1 as i32,
                    _ => 150,
                };
                m.record_jump(score, 90);
            }
        }

        assert_eq!(total_r1, 50);
        assert_eq!(total_r2, 30);
    }
}
