use crate::competition::field::{CompetitionField, SortBy};
use crate::competition::scoring;
use crate::competition::types::{
    CompetitionJumpOutcome, CompetitionPhase, CupStyle, CustomCupScoring, EventHistoryReason,
    EventJumpHistory, IndividualEventHistory, Participant, QualificationStatus,
};
use crate::jump::types::{FallType, JumpOutcome};
use serde::{Deserialize, Serialize};

const QUALIFICATION_SPOTS: usize = 50;
const ROUND2_SPOTS: usize = 30;
const PRE_QUALIFIED_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepDecision {
    ShowResults,
    AdvancePhase,
    Jump {
        idx: usize,
        hill_idx: usize,
        is_human: bool,
    },
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub(crate) field: CompetitionField,
    pub(crate) style: CupStyle,
    pub(crate) hill_order: Vec<usize>,
    pub(crate) current_event: usize,
    pub(crate) phase: CompetitionPhase,
    pub(crate) training_rounds: usize,
    pub(crate) custom_cup_scoring: CustomCupScoring,
    pub(crate) custom_cup_file: Option<String>,

    start_list: Vec<usize>,
    start_pos: usize,

    pub(crate) ko_system: bool,

    ko_pairings: Vec<usize>,
}

impl Competition {
    pub fn new(style: CupStyle, participants: Vec<Participant>, hill_order: Vec<usize>) -> Self {
        Self {
            field: CompetitionField::new(participants),
            style,
            hill_order,
            current_event: 0,
            phase: CompetitionPhase::Setup,
            training_rounds: 2,
            custom_cup_scoring: CustomCupScoring::default(),
            custom_cup_file: None,
            start_list: Vec::new(),
            start_pos: 0,
            ko_system: false,
            ko_pairings: Vec::new(),
        }
    }

    pub fn phase(&self) -> CompetitionPhase {
        self.phase
    }

    pub fn style(&self) -> CupStyle {
        self.style
    }

    pub fn uses_aggregate_standings(&self) -> bool {
        self.style == CupStyle::FourHills
            || self.style == CupStyle::CustomCup
                && matches!(
                    self.custom_cup_scoring,
                    CustomCupScoring::AggregateJumpPoints
                )
    }

    pub fn custom_cup_file(&self) -> Option<&str> {
        self.custom_cup_file.as_deref()
    }

    pub fn current_hill(&self) -> usize {
        self.hill_order
            .get(self.current_event)
            .copied()
            .unwrap_or(0)
    }

    pub fn participant(&self, idx: usize) -> &Participant {
        self.field.get(idx)
    }

    pub const fn current_start_order_pos(&self) -> usize {
        self.start_pos
    }

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

        let Some(idx) = self.current_jumper() else {
            return StepDecision::ShowResults;
        };
        let hill_idx = self.current_hill();
        let is_human = !self.participant(idx).is_computer;

        if is_human && self.phase == CompetitionPhase::Qualification {
            let p = self.participant(idx);
            if p.qual == QualificationStatus::PreQualified
                && (p.skip_qualification == 2
                    || (p.skip_qualification == 1 && !self.is_four_hills_event()))
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

    pub fn current_jumper(&self) -> Option<usize> {
        if self.start_pos < self.start_list.len() {
            Some(self.start_list[self.start_pos])
        } else {
            None
        }
    }

    pub fn is_over(&self) -> bool {
        self.phase == CompetitionPhase::SeasonComplete
    }

    pub const fn total_events(&self) -> usize {
        self.hill_order.len()
    }

    pub fn event_standings(&self) -> Vec<&Participant> {
        self.field
            .event_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    pub fn overall_standings(&self) -> Vec<&Participant> {
        self.field
            .master_order
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    pub fn ko_pairing_standings(&self) -> Vec<&Participant> {
        self.ko_pairings
            .iter()
            .map(|&idx| self.field.get(idx))
            .collect()
    }

    pub fn advance(&mut self) {
        match self.phase {
            CompetitionPhase::SeasonComplete => {}

            CompetitionPhase::Setup => self.enter_setup(),
            CompetitionPhase::Training(n) => {
                if self.start_pos >= self.start_list.len() {
                    let next = n + 1;
                    if next <= self.training_rounds {
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
                let event_placings: Vec<_> = (0..self.field.len())
                    .map(|idx| {
                        (self.field.get(idx).points.is_some()).then_some(self.field.get(idx).rank)
                    })
                    .collect();
                self.award_points();
                if self.is_four_hills_event() || self.uses_aggregate_standings() {
                    self.field.sort_field(SortBy::FourHillsPoints);
                    if self.current_event + 1 >= self.hill_order.len() {
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
                self.record_event_history(&event_placings);
            }
            CompetitionPhase::FourHillsStandings => {
                self.enter_phase(CompetitionPhase::EventComplete);
            }
            CompetitionPhase::WorldCupStandings => {
                self.enter_phase(CompetitionPhase::EventComplete);
            }
            CompetitionPhase::EventComplete => {
                if let Some(&winner_idx) = self.field.event_order.first() {
                    self.field.get_mut(winner_idx).leg_wins += 1;
                }
                self.current_event += 1;
                if self.current_event >= self.hill_order.len() {
                    self.phase = CompetitionPhase::SeasonComplete;
                } else {
                    self.enter_phase(CompetitionPhase::Setup);
                }
            }
        }
    }

    pub fn apply_jump_outcome(&mut self, outcome: JumpOutcome) {
        if outcome.fall_type != FallType::None {
            self.injure_current(outcome.injury);
        }
        self.record_jump(CompetitionJumpOutcome {
            score: outcome.score,
            distance: outcome.distance,
            fall_type: outcome.fall_type,
        });
    }

    pub(crate) fn record_jump(&mut self, outcome: CompetitionJumpOutcome) {
        let Some(&idx) = self.start_list.get(self.start_pos) else {
            return;
        };
        match self.phase {
            CompetitionPhase::Training(_) => {}
            CompetitionPhase::Qualification => {
                self.field.get_mut(idx).points = Some(outcome.score);
                self.field.get_mut(idx).qual_score = Some(outcome.score);
                self.field.get_mut(idx).qual_len = outcome.distance;
            }
            CompetitionPhase::Round1 => {
                self.field.get_mut(idx).points = Some(outcome.score);
                self.field.get_mut(idx).round1_score = outcome.score;
                self.field.get_mut(idx).round1_len = outcome.distance;
            }
            CompetitionPhase::Round2 => {
                let existing = self.field.get(idx).points.unwrap_or(0.0);
                self.field.get_mut(idx).points = Some(existing + outcome.score);
                self.field.get_mut(idx).round2_score = outcome.score;
                self.field.get_mut(idx).round2_len = outcome.distance;
            }
            _ => {}
        }
        self.field.sort_field(SortBy::EventPoints);
        self.start_pos += 1;
    }

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

    fn enter_phase(&mut self, phase: CompetitionPhase) {
        self.phase = phase;
        self.start_pos = 0;
        self.start_list = self.field.build_start_list(phase);

        if self.start_list.is_empty() && phase.is_jump_phase() {
            self.advance();
        }
    }

    fn enter_setup(&mut self) {
        self.field.reset_event();
        self.field.tick_injuries();
        self.sort_overall_field();

        if self.current_event > 0 {
            for idx in 0..self.field.len() {
                if self.field.get(idx).rank <= PRE_QUALIFIED_COUNT
                    && self.field.get(idx).injury == 0
                {
                    self.field.get_mut(idx).qual = QualificationStatus::PreQualified;
                }
            }
        }

        if self.training_rounds > 0 {
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
        self.ko_system && self.is_four_hills_event()
    }

    fn sort_overall_field(&mut self) {
        if self.uses_aggregate_standings() {
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

        if self.current_event == 0 {
            if self.is_ko_event() {
                self.ko_pairings = self.field.event_order.iter().take(50).copied().collect();
                for (seed, idx) in self.ko_pairings.clone().into_iter().enumerate() {
                    self.field.get_mut(idx).qual = QualificationStatus::KoSeed(seed + 1);
                }
            } else {
                for idx in self.field.event_order.clone() {
                    let qualified = self.field.get(idx).rank <= QUALIFICATION_SPOTS;
                    self.field.get_mut(idx).qual = if qualified {
                        QualificationStatus::Qualified
                    } else {
                        QualificationStatus::Eliminated
                    };
                }
            }
            self.snapshot_qualification_results();
            return;
        }

        if self.is_ko_event() {
            let mut seeds = Vec::with_capacity(50);
            for idx in self.field.master_order.clone() {
                if seeds.len() >= 50 {
                    break;
                }
                if self.field.get(idx).injury == 0
                    && self.field.get(idx).qual == QualificationStatus::PreQualified
                {
                    seeds.push(idx);
                }
            }
            for idx in self.field.event_order.clone() {
                if seeds.len() >= 50 {
                    break;
                }
                if self.field.get(idx).injury == 0
                    && self.field.get(idx).qual != QualificationStatus::PreQualified
                    && self.field.get(idx).points.is_some()
                {
                    seeds.push(idx);
                }
            }
            self.ko_pairings = seeds.clone();
            for idx in 0..self.field.len() {
                self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
            }
            for (seed, idx) in seeds.into_iter().enumerate() {
                self.field.get_mut(idx).qual = QualificationStatus::KoSeed(seed + 1);
            }
            self.snapshot_qualification_results();
            return;
        }

        let pre_qualified = self
            .field
            .iter()
            .filter(|p| p.qual == QualificationStatus::PreQualified)
            .count();

        let spots = QUALIFICATION_SPOTS.saturating_sub(pre_qualified);
        let mut taken = 0usize;
        let mut cutoff_score = None;

        for idx in self.field.event_order.clone() {
            let p = self.field.get(idx);
            if p.injury > 0 || p.qual == QualificationStatus::PreQualified {
                continue;
            }
            if taken < spots {
                cutoff_score = p.points;
                self.field.get_mut(idx).qual = QualificationStatus::Qualified;
                taken += 1;
            } else if p.points == cutoff_score {
                self.field.get_mut(idx).qual = QualificationStatus::Qualified;
            } else {
                self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
            }
        }
        self.snapshot_qualification_results();
    }

    fn snapshot_qualification_results(&mut self) {
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).qualification_result = self.field.get(idx).qual;
        }
    }

    fn prepare_round1_scores(&mut self) {
        for idx in 0..self.field.len() {
            self.field.get_mut(idx).points = None;
        }
    }

    fn apply_ko_results(&mut self) {
        let pairings = self.ko_pairings.clone();

        for idx in 0..self.field.len() {
            self.field.get_mut(idx).qual = QualificationStatus::Eliminated;
        }
        let count = pairings.len().min(50);
        let half = count / 2;

        for pair in 0..half.min(25) {
            let left = pairings[half + pair];
            let right = pairings[half - 1 - pair];
            let winner = if self.field.get(right).points.unwrap_or(0.0)
                >= self.field.get(left).points.unwrap_or(0.0)
            {
                right
            } else {
                left
            };
            self.field.get_mut(winner).qual = QualificationStatus::Qualified;
        }

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

        for idx in 0..self.field.len() {
            self.field.get_mut(idx).round1_rank = self.field.get(idx).rank;
        }
        if self.is_ko_event() {
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

    fn add_points_to_four_hills_totals(&mut self) {
        for idx in 0..self.field.len() {
            if let Some(pts) = self.field.get(idx).points {
                self.field.get_mut(idx).four_hills_points += pts;
            }
        }
    }

    fn award_points(&mut self) {
        match self.style {
            CupStyle::WorldCup => {
                scoring::award_wc_points(&mut self.field);
                if self.is_four_hills_event() {
                    self.add_points_to_four_hills_totals();
                }
            }
            CupStyle::FourHills => {
                self.add_points_to_four_hills_totals();
            }
            CupStyle::CustomCup => match self.custom_cup_scoring {
                CustomCupScoring::WorldCupPoints => {
                    scoring::award_wc_points(&mut self.field);
                }
                CustomCupScoring::AggregateJumpPoints => {
                    self.add_points_to_four_hills_totals();
                }
            },
            CupStyle::TeamCup => {}
        }
    }

    fn record_event_history(&mut self, event_placings: &[Option<usize>]) {
        if !matches!(self.style, CupStyle::WorldCup | CupStyle::FourHills) {
            return;
        }

        for idx in 0..self.field.len() {
            let p = self.field.get(idx);
            if p.is_computer {
                continue;
            }
            let placing = event_placings.get(idx).copied().flatten();
            let wc_points_awarded = if self.style == CupStyle::WorldCup {
                placing.map_or(0, scoring::wc_points_for_rank)
            } else {
                0
            };
            let running_rank = if self.style == CupStyle::FourHills {
                1 + self
                    .field
                    .iter()
                    .filter(|other| other.four_hills_points > p.four_hills_points)
                    .count()
            } else {
                1 + self
                    .field
                    .iter()
                    .filter(|other| other.wc_points > p.wc_points)
                    .count()
            };
            let history = IndividualEventHistory {
                event_index: self.current_event,
                hill_idx: self.current_hill(),
                qualification: p.qual_score.map(|points| EventJumpHistory {
                    distance: p.qual_len,
                    points,
                }),
                qualification_status: p.qualification_result,
                round1: (p.round1_len > 0.0).then_some(EventJumpHistory {
                    distance: p.round1_len,
                    points: p.round1_score,
                }),
                round2: (p.round2_len > 0.0).then_some(EventJumpHistory {
                    distance: p.round2_len,
                    points: p.round2_score,
                }),
                event_points: p.points,
                placing,
                wc_points_awarded,
                running_rank,
                status: p.qual,
                reason: (p.injury > 0).then_some(EventHistoryReason::Injury(p.injury)),
            };
            self.field.get_mut(idx).event_history.push(history);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jump::types::LandingStyle;

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
        c.training_rounds = 0;
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
                let pts = f64::from(150 + (machine.start_pos as i32 % 100));
                let len = f64::from(80 + (machine.start_pos as i32 % 50));
                machine.record_jump(CompetitionJumpOutcome {
                    score: pts,
                    distance: len,
                    fall_type: FallType::None,
                });
            }
        }
    }

    #[test]
    fn single_event_full_cycle() {
        let mut m = make_season(1);
        run_all_jumps(&mut m);
        assert!(m.is_over());

        let winner = m.field.get(0);
        assert!(winner.wc_points > 0, "winner should have WC points");
        assert_eq!(winner.rank, 1);

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

        let total: i32 = (0..m.field.len()).map(|i| m.field.get(i).wc_points).sum();
        assert!(total > 0, "total WC points should be positive");
    }

    #[test]
    fn human_event_history_survives_event_reset_and_serde() {
        let mut m = make_season(2);
        m.field.get_mut(0).is_computer = false;

        while m.phase != CompetitionPhase::EventComplete {
            if m.current_jumper().is_none() {
                m.advance();
            } else {
                m.record_jump(CompetitionJumpOutcome {
                    score: 150.0,
                    distance: 100.0,
                    fall_type: FallType::None,
                });
            }
        }

        let history = m.field.get(0).event_history.clone();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].event_index, 0);
        assert!(history[0].qualification.is_some());
        assert!(history[0].round1.is_some());

        m.advance();
        m.advance();
        assert_eq!(m.field.get(0).event_history, history);
        assert_eq!(m.field.get(0).qual_score, None);

        let encoded = toml::to_string(&m).unwrap();
        let decoded: Competition = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.field.get(0).event_history, history);
    }

    #[test]
    fn pre_qualification_works() {
        let mut m = make_season(2);

        while m.phase != CompetitionPhase::EventComplete {
            if m.current_jumper().is_none() {
                m.advance();
                continue;
            }
            while let Some(idx) = m.current_jumper() {
                m.record_jump(CompetitionJumpOutcome {
                    score: f64::from(200 - idx as i32),
                    distance: 90.0,
                    fall_type: FallType::None,
                });
            }
        }

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
            while m.current_jumper().is_some() {
                match m.phase {
                    CompetitionPhase::Round1 => total_r1 += 1,
                    CompetitionPhase::Round2 => total_r2 += 1,
                    _ => {}
                }
                let score = match m.phase {
                    CompetitionPhase::Round1 => f64::from(200 - total_r1 as i32),
                    _ => 150.0,
                };
                m.record_jump(CompetitionJumpOutcome {
                    score,
                    distance: 90.0,
                    fall_type: FallType::None,
                });
            }
        }

        assert_eq!(total_r1, 50);
        assert_eq!(total_r2, 30);
    }

    #[test]
    fn non_ko_qualification_includes_ties_at_cutoff() {
        let mut participants: Vec<_> = (0..52)
            .map(|i| Participant::computer(i, i, format!("J {i}")))
            .collect();
        for (i, participant) in participants.iter_mut().enumerate() {
            participant.points = Some(if i < 49 { 200.0 - i as f64 } else { 100.0 });
        }
        let mut competition = Competition::new(CupStyle::WorldCup, participants, vec![0]);

        competition.resolve_qualification();

        assert_eq!(
            competition
                .field
                .iter()
                .filter(|p| p.qual == QualificationStatus::Qualified)
                .count(),
            52
        );
    }

    #[test]
    fn ko_qualification_remains_exactly_fifty_when_cutoff_is_tied() {
        let mut participants: Vec<_> = (0..52)
            .map(|i| Participant::computer(i, i, format!("J {i}")))
            .collect();
        for participant in &mut participants {
            participant.points = Some(100.0);
        }
        let mut competition = Competition::new(CupStyle::FourHills, participants, vec![8]);
        competition.ko_system = true;

        competition.resolve_qualification();

        assert_eq!(
            competition
                .field
                .iter()
                .filter(|p| matches!(p.qual, QualificationStatus::KoSeed(_)))
                .count(),
            50
        );
    }

    #[test]
    fn ko_qualification_keeps_prequalified_jumper_without_qualification_score() {
        let mut participants: Vec<_> = (0..52)
            .map(|i| Participant::computer(i, i, format!("J {i}")))
            .collect();
        for participant in participants.iter_mut().take(10) {
            participant.qual = QualificationStatus::PreQualified;
            participant.points = None;
        }
        for participant in participants.iter_mut().skip(10) {
            participant.points = Some(100.0);
        }
        let mut competition = Competition::new(CupStyle::FourHills, participants, vec![8]);
        competition.ko_system = true;
        competition.current_event = 1;

        competition.resolve_qualification();

        assert_eq!(competition.ko_pairings.len(), 50);
        assert_eq!(&competition.ko_pairings[..10], &(0..10).collect::<Vec<_>>());
        assert!(competition
            .field
            .iter()
            .take(10)
            .all(|p| matches!(p.qual, QualificationStatus::KoSeed(_))));
    }

    #[test]
    fn ko_tie_advances_the_better_seed() {
        let mut participants: Vec<_> = (0..50)
            .map(|i| Participant::computer(i, i, format!("J {i}")))
            .collect();
        for (i, participant) in participants.iter_mut().enumerate() {
            participant.qual = QualificationStatus::KoSeed(i + 1);
            participant.points = Some(100.0);
        }
        let mut competition = Competition::new(CupStyle::FourHills, participants, vec![8]);
        competition.ko_system = true;
        competition.ko_pairings = (0..50).collect();

        competition.apply_ko_results();

        assert_eq!(
            competition.field.get(24).qual,
            QualificationStatus::Qualified
        );
        assert_eq!(
            competition.field.get(25).qual,
            QualificationStatus::LuckyLoser
        );
    }

    #[test]
    fn fall_uses_injury_duration_generated_by_jump_rng() {
        let mut competition = Competition::new(
            CupStyle::WorldCup,
            vec![Participant::computer(0, 0, "J".into())],
            vec![0],
        );
        competition.phase = CompetitionPhase::Round1;
        competition.start_list = vec![0];

        competition.apply_jump_outcome(JumpOutcome {
            distance: 80.0,
            score: 0.0,
            style_points: [0.0; 5],
            landing_style: LandingStyle::None,
            fall_type: FallType::Normal,
            injury: 4,
            aborted: false,
        });

        assert_eq!(competition.field.get(0).injury, 4);
    }

    #[test]
    fn result_phases_match_pascal_round_flow() {
        let mut m = make_season(1);
        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(CompetitionJumpOutcome {
                score: 100.0,
                distance: 80.0,
                fall_type: FallType::None,
            });
        }

        m.advance();
        assert_eq!(m.phase, CompetitionPhase::QualificationResults);

        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1);
        assert!((0..m.field.len()).all(|i| m.field.get(i).points.is_none()));

        while m.current_jumper().is_some() {
            let score = f64::from(200 - m.start_pos as i32);
            m.record_jump(CompetitionJumpOutcome {
                score,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1Results);
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round2);
    }

    #[test]
    fn completed_round_phases_advance_to_result_phases() {
        let mut m = make_season(1);
        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(CompetitionJumpOutcome {
                score: 100.0,
                distance: 80.0,
                fall_type: FallType::None,
            });
        }
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();

        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(CompetitionJumpOutcome {
                score: f64::from(200 - m.start_pos as i32),
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        assert_eq!(m.phase, CompetitionPhase::Round1);
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round1Results);

        m.advance();
        while m.current_jumper().is_some() {
            m.record_jump(CompetitionJumpOutcome {
                score: f64::from(200 - m.start_pos as i32),
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        assert_eq!(m.phase, CompetitionPhase::Round2);
        assert_eq!(m.decide_next(), StepDecision::AdvancePhase);
        m.advance();
        assert_eq!(m.phase, CompetitionPhase::Round2Results);
    }

    #[test]
    fn four_hills_shows_tour_standings_between_events() {
        let mut c = Competition::new(CupStyle::FourHills, make_50_participants(), vec![8, 9]);
        c.training_rounds = 0;

        while c.phase != CompetitionPhase::Round2Results {
            if c.current_jumper().is_none() {
                c.advance();
                continue;
            }
            while c.current_jumper().is_some() {
                c.record_jump(CompetitionJumpOutcome {
                    score: 150.0,
                    distance: 90.0,
                    fall_type: FallType::None,
                });
            }
        }

        c.advance();

        assert_eq!(c.phase, CompetitionPhase::FourHillsStandings);
        assert!(c.field.iter().any(|p| p.four_hills_points > 0.0));
        assert!(c.field.iter().all(|p| p.wc_points == 0));
    }

    #[test]
    fn four_hills_final_uses_tour_points() {
        let mut c = Competition::new(CupStyle::FourHills, make_50_participants(), vec![8]);
        c.training_rounds = 0;

        run_all_jumps(&mut c);

        assert_eq!(c.phase, CompetitionPhase::SeasonComplete);
        assert!(c.field.iter().any(|p| p.four_hills_points > 0.0));
        assert!(c.field.iter().all(|p| p.wc_points == 0));
        let leader = c.field.master_order[0];
        assert_eq!(c.field.get(leader).rank, 1);
    }

    #[test]
    fn custom_cup_wc_mode_accumulates_place_points_and_uses_wc_standings() {
        let mut c = Competition::new(CupStyle::CustomCup, make_50_participants(), vec![0, 1]);
        c.custom_cup_scoring = CustomCupScoring::WorldCupPoints;
        for idx in 0..c.field.len() {
            c.field.get_mut(idx).points = Some(200.0 - idx as f64);
        }
        c.field.sort_field(SortBy::EventPoints);
        c.phase = CompetitionPhase::Round2Results;

        c.advance();

        assert_eq!(c.phase, CompetitionPhase::WorldCupStandings);
        assert_eq!(c.field.get(0).wc_points, 100);
        assert_eq!(c.field.get(1).wc_points, 80);
        assert!(c.field.iter().all(|p| p.four_hills_points == 0.0));
        assert_eq!(c.overall_standings()[0].id, 0);
    }

    #[test]
    fn custom_cup_aggregate_mode_accumulates_jump_points_and_uses_total_standings() {
        let mut participants = make_50_participants();
        participants[1].four_hills_points = 150.0;
        let mut c = Competition::new(CupStyle::CustomCup, participants, vec![0, 1]);
        c.custom_cup_scoring = CustomCupScoring::AggregateJumpPoints;
        for idx in 0..c.field.len() {
            c.field.get_mut(idx).points = Some(200.0 - idx as f64);
        }
        c.field.sort_field(SortBy::EventPoints);
        c.phase = CompetitionPhase::Round2Results;

        c.advance();

        assert_eq!(c.phase, CompetitionPhase::FourHillsStandings);
        assert_eq!(c.field.get(0).four_hills_points, 200.0);
        assert_eq!(c.field.get(1).four_hills_points, 349.0);
        assert!(c.field.iter().all(|p| p.wc_points == 0));
        assert_eq!(c.overall_standings()[0].id, 1);
    }

    #[test]
    fn skip_for_prequalified_human_in_qualification() {
        let mut participants = make_50_participants();

        let human_idx = 49;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_qualification = 1;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0, 1]);
        c.training_rounds = 0;

        c.advance();
        while c.current_jumper().is_some() {
            c.record_jump(CompetitionJumpOutcome {
                score: 150.0,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        c.advance();
        c.advance();
        while c.current_jumper().is_some() {
            c.record_jump(CompetitionJumpOutcome {
                score: 150.0,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        c.advance();
        c.advance();
        while c.current_jumper().is_some() {
            c.record_jump(CompetitionJumpOutcome {
                score: 150.0,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        c.advance();
        c.advance();
        c.advance();
        c.advance();
        c.advance();

        assert_eq!(c.phase, CompetitionPhase::Qualification);

        while let Some(idx) = c.current_jumper() {
            if idx == human_idx {
                assert_eq!(c.decide_next(), StepDecision::Skip);
                return;
            }
            c.record_jump(CompetitionJumpOutcome {
                score: 0.0,
                distance: 0.0,
                fall_type: FallType::None,
            });
        }
        panic!("human never reached current jumper in qualification");
    }

    #[test]
    fn prequalified_ai_still_gets_jump() {
        let mut participants = make_50_participants();
        let human_idx = 49;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_qualification = 1;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0, 1]);
        c.training_rounds = 0;

        c.advance();
        while let Some(idx) = c.current_jumper() {
            if idx == human_idx {
                break;
            }

            assert!(
                matches!(c.decide_next(), StepDecision::Jump { .. }),
                "AI should get Jump, got {:?}",
                c.decide_next()
            );
            c.record_jump(CompetitionJumpOutcome {
                score: 0.0,
                distance: 0.0,
                fall_type: FallType::None,
            });
        }
    }

    #[test]
    fn round1_rank_is_frozen_before_round2() {
        let mut participants = make_50_participants();

        for (i, p) in participants.iter_mut().enumerate() {
            p.points = Some(f64::from(1000 - i as i32 * 10));
        }
        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.training_rounds = 0;

        c.advance();
        while c.current_jumper().is_some() {
            c.record_jump(CompetitionJumpOutcome {
                score: 0.0,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        c.advance();
        c.advance();
        while c.current_jumper().is_some() {
            c.record_jump(CompetitionJumpOutcome {
                score: 0.0,
                distance: 90.0,
                fall_type: FallType::None,
            });
        }
        c.advance();
        c.advance();

        for i in 0..c.field.len() {
            assert!(
                c.field.get(i).round1_rank > 0,
                "participant {i} should have round1_rank > 0, got {}",
                c.field.get(i).round1_rank
            );
        }

        let first_in_event = c.field.event_order[0];
        assert_eq!(c.field.get(first_in_event).round1_rank, 1);
    }

    #[test]
    fn decide_next_skip_for_manually_prequalified_human() {
        let mut participants = make_50_participants();
        let human_idx = 5;
        participants[human_idx].is_computer = false;
        participants[human_idx].skip_qualification = 1;
        participants[human_idx].qual = QualificationStatus::PreQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.training_rounds = 0;
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
        c.training_rounds = 0;
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
        participants[human_idx].skip_qualification = 1;

        participants[human_idx].qual = QualificationStatus::NotQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.training_rounds = 0;
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
        participants[human_idx].skip_qualification = 0;
        participants[human_idx].qual = QualificationStatus::PreQualified;

        let mut c = Competition::new(CupStyle::WorldCup, participants, vec![0]);
        c.training_rounds = 0;
        c.phase = CompetitionPhase::Qualification;
        c.start_list = vec![human_idx];
        c.start_pos = 0;

        assert!(matches!(c.decide_next(), StepDecision::Jump { .. }));
    }
}
