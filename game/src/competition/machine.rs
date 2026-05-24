use crate::competition::field::{CompetitionField, SortBy};
use crate::competition::scoring;
use crate::competition::types::{CompetitionPhase, CupStyle, Participant, QualificationStatus};

const QUALIFICATION_SPOTS: usize = 50;
const ROUND2_SPOTS: usize = 30;
const PRE_QUALIFIED_COUNT: usize = 10;

/// Pure decision returned by `Competition::decide_next()`.
/// No mutation, no IO — just describes what the caller should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepDecision {
    /// Season is over.
    Done,
    /// Show a results/standings screen (then caller must `advance`).
    ShowResults,
    /// Auto-advance through trivial phases (Training, Setup, EventComplete).
    AdvancePhase,
    /// A specific participant must jump.
    Jump {
        idx: usize,
        hill_idx: usize,
        is_human: bool,
    },
    /// Skip the current jumper (pre-qualified human with skipquali in quali).
    Skip,
}

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
    pub(crate) field: CompetitionField,
    pub(crate) style: CupStyle,
    pub(crate) hill_order: Vec<usize>,
    pub(crate) current_event: usize,
    pub(crate) phase: CompetitionPhase,
    pub(crate) trainrounds: usize,

    start_list: Vec<usize>,
    start_pos: usize,

    /// Pascal mcluett: saved seed-pairing order for KO results display.
    ko_pairings: Vec<usize>,
}

impl Competition {
    #[must_use]
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
            ko_pairings: Vec::new(),
        }
    }

    #[must_use]
    pub fn phase(&self) -> CompetitionPhase {
        self.phase
    }

    #[must_use]
    pub fn style(&self) -> CupStyle {
        self.style
    }

    #[must_use]
    pub fn current_hill(&self) -> usize {
        self.hill_order
            .get(self.current_event)
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn participant(&self, idx: usize) -> &Participant {
        self.field.get(idx)
    }

    // ── queries ────────────────────────────────────────────────

    /// Pure decision: what should the caller do next?
    /// No mutation, no store access — just reads current state.
    #[must_use]
    pub fn decide_next(&self) -> StepDecision {
        if self.phase.is_result_phase() {
            return StepDecision::ShowResults;
        }

        if self.current_jumper().is_none() {
            if self.phase.auto_advances_when_empty() {
                return StepDecision::AdvancePhase;
            }
            return StepDecision::ShowResults;
        }

        let idx = self.current_jumper().unwrap();
        let hill_idx = self.current_hill();
        let is_human = !self.participant(idx).is_computer;

        // Pascal SJ3.PAS:5367: skip if (skipquali=2) or ((not fourhills) and (skipquali=1))
        if is_human && self.phase == CompetitionPhase::Qualification {
            let p = self.participant(idx);
            if p.qual == QualificationStatus::PreQualified
                && (p.skip_quali == 2 || (p.skip_quali == 1 && !self.is_four_hills_event()))
            {
                return StepDecision::Skip;
            }
        }

        StepDecision::Jump {
            idx,
            hill_idx,
            is_human,
        }
    }

    #[must_use]
    pub fn current_jumper(&self) -> Option<usize> {
        if self.start_pos < self.start_list.len() {
            Some(self.start_list[self.start_pos])
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_human_current(&self) -> bool {
        self.current_jumper()
            .is_some_and(|idx| !self.field.get(idx).is_computer)
    }

    #[must_use]
    pub fn is_over(&self) -> bool {
        self.phase == CompetitionPhase::SeasonComplete
    }

    #[must_use]
    pub const fn phase_progress(&self) -> (usize, usize) {
        (self.start_pos, self.start_list.len())
    }

    /// Number of events in the season.
    #[must_use]
    pub const fn total_events(&self) -> usize {
        self.hill_order.len()
    }

    /// Participants in event-points order (for results lists).
    #[must_use]
    pub fn event_standings(&self) -> Vec<&Participant> {
        self.field
            .event_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    /// Participants in season-points order (for WC standings).
    #[must_use]
    pub fn overall_standings(&self) -> Vec<&Participant> {
        self.field
            .master_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    /// Participants in the saved KO seed-pairing order (Pascal luett/mcluett).
    /// Used for the KO pairs results display after Round 1.
    pub fn ko_pairing_standings(&self) -> Vec<&Participant> {
        self.ko_pairings
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
                    if next <= self.trainrounds {
                        self.enter_phase(CompetitionPhase::Training(next));
                    } else if self.style == CupStyle::CustomCup {
                        self.enter_custom_round1();
                    } else {
                        self.clear_event_points();
                        self.enter_phase(CompetitionPhase::Qualification);
                    }
                }
            }
            CompetitionPhase::Qualification => {
                if self.start_pos >= self.start_list.len() {
                    self.resolve_qualification();
                    self.enter_phase(CompetitionPhase::QualificationResults);
                }
            }
            CompetitionPhase::QualificationResults => {
                self.prepare_round1_scores();
                self.enter_phase(CompetitionPhase::Round1);
            }
            CompetitionPhase::Round1 => {
                if self.start_pos >= self.start_list.len() {
                    self.field.sort_field(SortBy::EventPoints);
                    // Pascal: assign KO winners/lucky losers BEFORE showing results
                    if self.is_ko_event() {
                        self.apply_ko_results();
                    }
                    self.enter_phase(CompetitionPhase::Round1Results);
                }
            }
            CompetitionPhase::Round1Results => {
                self.cut_to_round2();
                self.enter_phase(CompetitionPhase::Round2);
            }
            CompetitionPhase::Round2 => {
                if self.start_pos >= self.start_list.len() {
                    self.field.sort_field(SortBy::EventPoints);
                    self.enter_phase(CompetitionPhase::Round2Results);
                }
            }
            CompetitionPhase::Round2Results => {
                self.award_points();
                if self.is_four_hills_event() || self.style == CupStyle::CustomCup {
                    self.field.sort_field(SortBy::FourHillsPoints);
                    if self.current_event + 1 >= self.hill_order.len() {
                        // Last event: sort back to primary standings for final display
                        if self.style == CupStyle::WorldCup {
                            self.field.sort_field(SortBy::WcPoints);
                        }
                        self.phase = CompetitionPhase::SeasonComplete;
                    } else {
                        self.enter_phase(CompetitionPhase::FourHillsStandings);
                    }
                } else {
                    self.field.sort_field(SortBy::WcPoints);
                    self.enter_phase(CompetitionPhase::WorldCupStandings);
                }
            }
            CompetitionPhase::FourHillsStandings => {
                self.enter_phase(CompetitionPhase::EventComplete);
            }
            CompetitionPhase::WorldCupStandings => {
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
            CompetitionPhase::Training(_) => {}
            CompetitionPhase::Qualification => {
                self.field.get_mut(idx).points = Some(jump_points);
                self.field.get_mut(idx).qual_len = length;
            }
            CompetitionPhase::Round1 => {
                self.field.get_mut(idx).points = Some(jump_points);
                self.field.get_mut(idx).round1_score = jump_points;
                self.field.get_mut(idx).round1_len = length;
            }
            CompetitionPhase::Round2 => {
                let existing = self.field.get(idx).points.unwrap_or(0);
                self.field.get_mut(idx).points = Some(existing + jump_points);
                self.field.get_mut(idx).round2_len = length;
            }
            _ => {}
        }
        self.field.sort_field(SortBy::EventPoints);
        self.start_pos += 1;
    }

    /// Advance past the current jumper without recording a jump.
    /// Used when a pre-qualified human has skipquali enabled.
    pub fn skip_current_jumper(&mut self) {
        self.start_pos += 1;
    }

    fn clear_event_points(&mut self) {
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).points = None;
        }
    }

    pub fn injure_current(&mut self, rounds: u8) {
        let Some(&idx) = self.start_list.get(self.start_pos) else {
            return;
        };
        self.field.get_mut(idx).injury = self.field.get(idx).injury.max(rounds);
    }

    // ── internal ───────────────────────────────────────────────

    fn enter_phase(&mut self, phase: CompetitionPhase) {
        self.phase = phase;
        self.start_pos = 0;
        self.start_list = self.field.build_start_list(phase);

        // Only jump phases can be skipped when there is nobody to jump.
        // Result-list phases must be observable by UI.
        if self.start_list.is_empty() && phase.is_jump_phase() {
            self.advance();
        }
    }

    fn enter_setup(&mut self) {
        self.field.reset_event();
        self.field.tick_injuries();
        self.sort_overall_field();

        // Top 10 in overall WC classification skip qualification
        if self.current_event > 0 {
            for idx in 0..self.field.len() {
                if self.field.get(idx).rank <= PRE_QUALIFIED_COUNT
                    && self.field.get(idx).injury == 0
                {
                    self.field.get_mut(idx).qual = QualificationStatus::PreQualified;
                }
            }
        }

        if self.trainrounds > 0 {
            self.enter_phase(CompetitionPhase::Training(1));
        } else if self.style == CupStyle::CustomCup {
            self.enter_custom_round1();
        } else {
            self.enter_phase(CompetitionPhase::Qualification);
        }
    }

    pub fn is_four_hills_event(&self) -> bool {
        self.style == CupStyle::FourHills
            || (self.style == CupStyle::WorldCup && (8..=11).contains(&self.current_hill()))
    }

    fn is_ko_event(&self) -> bool {
        self.is_four_hills_event()
    }

    fn sort_overall_field(&mut self) {
        if matches!(self.style, CupStyle::FourHills | CupStyle::CustomCup) {
            self.field.sort_field(SortBy::FourHillsPoints);
        } else {
            self.field.sort_field(SortBy::WcPoints);
        }
    }

    fn enter_custom_round1(&mut self) {
        for idx in 0..self.field.len() {
            if self.field.get(idx).injury == 0 {
                self.field.get_mut(idx).qual = QualificationStatus::Qualified;
                self.field.get_mut(idx).points = None;
            } else {
                self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
                self.field.get_mut(idx).points = None;
            }
        }
        self.enter_phase(CompetitionPhase::Round1);
    }

    fn resolve_qualification(&mut self) {
        self.field.sort_field(SortBy::EventPoints);

        if self.is_ko_event() {
            // Save seed-pairing order (Pascal mcluett) for KO results display
            self.ko_pairings = self.field.event_order.iter().take(50).copied().collect();
            for (seed, idx) in self
                .field
                .event_order
                .clone()
                .into_iter()
                .take(50)
                .enumerate()
            {
                if self.field.get(idx).injury == 0 {
                    self.field.get_mut(idx).qual = QualificationStatus::KoSeed(seed + 1);
                }
            }
            return;
        }

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

    fn prepare_round1_scores(&mut self) {
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).points = None;
        }
    }

    /// Pascal lines 5487-5499: assign KO winners and lucky losers
    /// Uses saved ko_pairings (seed order, Pascal luett/mcluett) for correct pairing.
    fn apply_ko_results(&mut self) {
        let pairings = self.ko_pairings.clone();
        // Reset all to Eliminated first
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
        }
        let count = pairings.len().min(50);
        let half = count / 2;
        // Winners: qual=1 in Pascal (same pairing as display)
        // Pascal: luett[25..1] (RIGHT) vs luett[26..50] (LEFT)
        // Pascal: RIGHT.points >= LEFT.points → RIGHT wins
        for pair in 0..half.min(25) {
            let left = pairings[half + pair];
            let right = pairings[half - 1 - pair];
            let winner = if self.field.get(right).points.unwrap_or(0)
                >= self.field.get(left).points.unwrap_or(0)
            {
                right
            } else {
                left
            };
            self.field.get_mut(winner).qual = QualificationStatus::Qualified;
        }
        // 5 lucky losers: qual=2 in Pascal
        // Pascal: jarjestys(2,1,NumPl) — sort by points, then take first 5 with qual=0
        // We use event_order which is already sorted by Round 1 points
        // (sort_field was called at the start of the Round1 → Round1Results transition)
        let order = self.field.event_order.clone();
        let mut lucky = 0usize;
        for idx in &order {
            if lucky >= 5 {
                break;
            }
            let idx = *idx;
            if self.field.get(idx).qual == QualificationStatus::Eliminated
                && self.field.get(idx).injury == 0
                && self.field.get(idx).points.is_some()
            {
                self.field.get_mut(idx).qual = QualificationStatus::LuckyLoser;
                lucky += 1;
            }
        }
    }

    fn cut_to_round2(&mut self) {
        self.field.sort_field(SortBy::EventPoints);
        // Freeze Round 1 rank before Round 2 AI jumps re-sort event_order
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).round1_rank = self.field.get(idx).rank;
        }
        if self.is_ko_event() {
            // Qual already assigned by apply_ko_results — nothing more to do here
            return;
        }
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
                if self.is_four_hills_event() {
                    for idx in 0..self.field.len() {
                        if let Some(pts) = self.field.get(idx).points {
                            self.field.get_mut(idx).four_hills_points += pts;
                        }
                    }
                }
            }
            CupStyle::FourHills => {
                for idx in 0..self.field.len() {
                    if let Some(pts) = self.field.get(idx).points {
                        self.field.get_mut(idx).four_hills_points += pts;
                    }
                }
            }
            CupStyle::CustomCup => {
                for idx in 0..self.field.len() {
                    if let Some(pts) = self.field.get(idx).points {
                        self.field.get_mut(idx).four_hills_points += pts;
                    }
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
            .map(|i| Participant::computer(i, i, format!("J {i}")))
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
        while m.phase != CompetitionPhase::Round2Results {
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

    #[test]
    fn result_phases_match_pascal_round_flow() {
        let mut m = make_season(1);
        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(100, 80);
        }

        m.advance();
        assert_eq!(m.phase, CompetitionPhase::QualificationResults);

        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1);
        assert!((0..m.field.len()).all(|i| m.field.get(i).points.is_none()));

        while m.current_jumper().is_some() {
            let score = 200 - m.start_pos as i32;
            m.record_jump(score, 90);
        }
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1Results);
        assert_eq!(m.field.num_qualified(), 50);

        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round2);
        assert_eq!(m.field.num_qualified(), 30);
    }

    #[test]
    fn completed_round_phases_advance_to_result_phases() {
        let mut m = make_season(1);
        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(100, 80);
        }
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();

        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(200 - m.start_pos as i32, 90);
        }
        assert_eq!(m.phase, CompetitionPhase::Round1);
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1Results);

        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(200 - m.start_pos as i32, 90);
        }
        assert_eq!(m.phase, CompetitionPhase::Round2);
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round2Results);
    }

    #[test]
    fn four_hills_shows_tour_standings_between_events() {
        let mut c = Competition::new(CupStyle::FourHills, make_50_participants(), vec![8, 9]);
        c.trainrounds = 0;

        while c.phase != CompetitionPhase::Round2Results {
            if c.current_jumper().is_none() {
                c.advance();
                continue;
            }
            while c.current_jumper().is_some() {
                c.record_jump(150, 90);
            }
        }

        c.advance();

        assert_eq!(c.phase, CompetitionPhase::FourHillsStandings);
        assert!(c.field.iter().any(|p| p.four_hills_points > 0));
        assert!(c.field.iter().all(|p| p.wc_points == 0));
    }

    #[test]
    fn four_hills_final_uses_tour_points() {
        let mut c = Competition::new(CupStyle::FourHills, make_50_participants(), vec![8]);
        c.trainrounds = 0;

        run_all_jumps(&mut c);

        assert_eq!(c.phase, CompetitionPhase::SeasonComplete);
        assert!(c.field.iter().any(|p| p.four_hills_points > 0));
        assert!(c.field.iter().all(|p| p.wc_points == 0));
        let leader = c.field.master_order[0];
        assert_eq!(c.field.get(leader).rank, 1);
    }

    #[test]
    fn skip_for_prequalified_human_in_qualification() {
        let mut participants = make_50_participants();
        // Turn the last participant (lowest WC rank) into a human with skipquali
        let human_idx = 49;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_quali = 1;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0, 1]);
        c.trainrounds = 0;

        // Run event 1 so setup for event 2 marks PreQualified
        c.advance(); // Setup -> Qualification
        while c.current_jumper().is_some() {
            c.record_jump(150, 90);
        }
        c.advance(); // -> QualificationResults
        c.advance(); // -> Round1
        while c.current_jumper().is_some() {
            c.record_jump(150, 90);
        }
        c.advance(); // -> Round1Results
        c.advance(); // -> Round2
        while c.current_jumper().is_some() {
            c.record_jump(150, 90);
        }
        c.advance(); // -> Round2Results
        c.advance(); // -> WC standings
        c.advance(); // -> EventComplete
        c.advance(); // advance to Setup for event 2
        c.advance(); // Setup -> Qualification for event 2

        assert_eq!(c.phase, CompetitionPhase::Qualification);

        // Consume AI jumpers until we reach the human
        while let Some(idx) = c.current_jumper() {
            if idx == human_idx {
                assert_eq!(c.decide_next(), StepDecision::Skip);
                return;
            }
            c.record_jump(0, 0);
        }
        panic!("human never reached current jumper in qualification");
    }

    #[test]
    fn prequalified_ai_still_gets_jump() {
        let mut participants = make_50_participants();
        let human_idx = 49;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_quali = 1;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0, 1]);
        c.trainrounds = 0;

        c.advance(); // -> Qualification
        while let Some(idx) = c.current_jumper() {
            if idx == human_idx {
                break;
            }
            // All AI jumpers (some may be PreQualified from previous season) get Jump
            assert!(
                matches!(c.decide_next(), StepDecision::Jump { .. }),
                "AI should get Jump, got {:?}",
                c.decide_next()
            );
            c.record_jump(0, 0);
        }
    }

    #[test]
    fn round1_rank_is_frozen_before_round2() {
        let mut participants = make_50_participants();
        // Give participants varied scores so we can verify ranks
        for (i, p) in participants.iter_mut().enumerate() {
            p.points = Some(1000 - i as i32 * 10); // 1000, 990, 980, ...
        }
        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.trainrounds = 0;

        c.advance(); // -> Qualification
        while c.current_jumper().is_some() {
            c.record_jump(0, 90);
        }
        c.advance(); // -> QualificationResults
        c.advance(); // -> Round1
        while c.current_jumper().is_some() {
            c.record_jump(0, 90);
        }
        c.advance(); // -> Round1Results (event_order sorted, rank set)
        c.advance(); // -> Round2 (cut_to_round2 freezes round1_rank)

        // Round1 rank should be set for all participants, not just those in Round 2
        for i in 0..c.field.len() {
            assert!(
                c.field.get(i).round1_rank > 0,
                "participant {i} should have round1_rank > 0, got {}",
                c.field.get(i).round1_rank
            );
        }
        // Top-ranked participant should have round1_rank = 1
        let first_in_event = c.field.event_order[0];
        assert_eq!(c.field.get(first_in_event).round1_rank, 1);
    }

    #[test]
    fn decide_next_skip_for_manually_prequalified_human() {
        let mut participants = make_50_participants();
        let human_idx = 5;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_quali = 1;
        participants[human_idx].qual = QualificationStatus::PreQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.trainrounds = 0;
        c.phase = CompetitionPhase::Qualification;
        c.start_list = vec![human_idx];
        c.start_pos = 0;

        assert_eq!(c.decide_next(), StepDecision::Skip);
    }

    #[test]
    fn decide_next_jump_for_manually_prequalified_ai() {
        let mut participants = make_50_participants();
        let ai_idx = 5;
        participants[ai_idx].qual = QualificationStatus::PreQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.trainrounds = 0;
        c.phase = CompetitionPhase::Qualification;
        c.start_list = vec![ai_idx];
        c.start_pos = 0;

        assert!(matches!(c.decide_next(), StepDecision::Jump { .. }));
    }

    #[test]
    fn non_prequalified_human_gets_jump_not_skip() {
        let mut participants = make_50_participants();
        let human_idx = 5;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_quali = 1;
        // NOT prequalified — should get Jump, not Skip
        participants[human_idx].qual = QualificationStatus::NotQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.trainrounds = 0;
        c.phase = CompetitionPhase::Qualification;
        c.start_list = vec![human_idx];
        c.start_pos = 0;

        assert!(matches!(c.decide_next(), StepDecision::Jump { .. }));
    }

    #[test]
    fn prequalified_human_without_skipquali_gets_jump() {
        let mut participants = make_50_participants();
        let human_idx = 5;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_quali = 0; // no skip
        participants[human_idx].qual = QualificationStatus::PreQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.trainrounds = 0;
        c.phase = CompetitionPhase::Qualification;
        c.start_list = vec![human_idx];
        c.start_pos = 0;

        assert!(matches!(c.decide_next(), StepDecision::Jump { .. }));
    }
}
