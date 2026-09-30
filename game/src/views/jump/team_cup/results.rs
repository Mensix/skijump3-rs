use crate::competition::team_cup::types::{
    TeamCupJumpResult, TeamCupResultsKind, TeamCupRuntime, TeamCupStandingsKind,
};
use crate::components::page_nav::{differential_score, render_page_hints, PageHintLayout};
use crate::gfx::theme::{
    BG_TEAM, BLACK, FILL_GOLD, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY, FONT_TEAL,
};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::format_decimal;
use crate::text::layout::shorten_name;
use crate::ui::UiCanvas;

fn paint_results_background(cx: &mut dyn UiCanvas, hill_background: bool) {
    if !hill_background {
        cx.fill((0, 0, 320, 200), BLACK);
        cx.pattern_fill((0, 20, 320, 180), BG_TEAM);
    }
    cx.fill((0, 0, 320, 19), FILL_GRAY);
}

pub(crate) fn render(
    cx: &mut dyn UiCanvas,
    resources: &ResourcesRef,
    state: &GameState,
    results_kind: TeamCupResultsKind,
    hill_background: bool,
) {
    let lang = &resources.langbase;
    paint_results_background(cx, hill_background);
    let standings_kind = match results_kind {
        TeamCupResultsKind::Standings => TeamCupStandingsKind::Overall,
        TeamCupResultsKind::LegResults => TeamCupStandingsKind::Leg,
    };
    let (header, standings) = state
        .active_competition
        .as_ref()
        .and_then(|active| {
            let tc = active.team_cup_runtime()?;
            let standings = tc.standings(standings_kind);
            let leg = tc.current_leg + 1;
            let round = tc.current_round + 1;
            let jumper = tc.current_jumper_slot + 1;
            let header = match results_kind {
                TeamCupResultsKind::LegResults => {
                    if tc.current_leg == 5 {
                        lang.tr(92).to_string()
                    } else {
                        format!(
                            "{} {} {} 6 - R {} - {} {}",
                            lang.tr(81),
                            leg,
                            lang.tr(8),
                            round,
                            lang.tr(88),
                            jumper,
                        )
                    }
                }
                TeamCupResultsKind::Standings => {
                    let text = lang.tr(91);
                    format!("{} {} 6", text, leg)
                }
            };
            Some((header, standings))
        })
        .unwrap_or_default();

    cx.text((30, 6), FONT_BODY, &header);

    let mut last_rank = 0usize;
    let mut y = 23i32;
    for (i, entry) in standings.iter().enumerate() {
        if i >= 15 {
            break;
        }
        let is_human = entry.is_human;

        if entry.rank != last_rank && entry.rank > 0 {
            let c = if is_human { FONT_GOLD } else { FILL_GOLD };
            cx.right_text((24, y), c, &format!("{}.", entry.rank));
        }
        last_rank = entry.rank;

        let nc = if is_human { FONT_BODY } else { FONT_GRAY };
        let name = shorten_name(&entry.name, &resources.font, 122);
        cx.text((32, y), nc, &name);

        let gap_enabled = match results_kind {
            TeamCupResultsKind::LegResults => state.config.event_gap != 0,
            TeamCupResultsKind::Standings => state.config.wc_gap != 0,
        };
        let leader = standings.first().map_or(0.0, |leader| leader.primary_score);
        let value = differential_score(leader, entry.primary_score, entry.rank, gap_enabled);
        cx.right_text((184, y), nc, &format_decimal(value));

        y += 10;
    }

    render_page_hints(cx, 0, 1, lang, PageHintLayout::Top);
}

#[derive(Debug)]
pub(crate) struct TeamStatsMember<'a> {
    name: &'a str,
    rounds: [Option<TeamCupJumpResult>; 2],
}

#[derive(Debug)]
pub(crate) struct TeamStatsPage<'a> {
    team_name: &'a str,
    leg: usize,
    hill_idx: usize,
    event_rank: usize,
    event_score: f64,
    cumulative_points: i32,
    cumulative_rank: usize,
    members: Vec<TeamStatsMember<'a>>,
}

pub(crate) fn build_stats_pages(runtime: &TeamCupRuntime) -> Vec<TeamStatsPage<'_>> {
    let current_leg_ranks = runtime.current_leg_standings();
    let overall_ranks = runtime.overall_standings();
    let mut pages = Vec::new();

    for (team_idx, team) in runtime.teams.iter().enumerate() {
        if !team.is_human_team {
            continue;
        }
        for leg in 0..runtime.schedule.len() {
            let snapshot = runtime
                .event_snapshots
                .iter()
                .find(|snapshot| snapshot.team_idx == team_idx && snapshot.leg == leg);
            let has_jumps = team
                .members
                .iter()
                .any(|member| member.jumps.iter().any(|jump| jump.leg == leg));
            if snapshot.is_none() && !has_jumps {
                continue;
            }

            let event_rank = snapshot.map_or_else(
                || {
                    current_leg_ranks
                        .iter()
                        .find(|entry| entry.name == team.name)
                        .map_or(0, |entry| entry.rank)
                },
                |snapshot| snapshot.event_rank,
            );
            let cumulative_rank = snapshot.map_or_else(
                || {
                    overall_ranks
                        .iter()
                        .find(|entry| entry.name == team.name)
                        .map_or(0, |entry| entry.rank)
                },
                |snapshot| snapshot.cumulative_rank,
            );
            pages.push(TeamStatsPage {
                team_name: &team.name,
                leg,
                hill_idx: snapshot.map_or_else(
                    || runtime.schedule.get(leg).copied().unwrap_or(0),
                    |snapshot| snapshot.hill_idx,
                ),
                event_rank,
                event_score: snapshot.map_or(team.leg_score, |snapshot| snapshot.event_score),
                cumulative_points: snapshot
                    .map_or(team.cup_points, |snapshot| snapshot.cumulative_points),
                cumulative_rank,
                members: team
                    .members
                    .iter()
                    .map(|member| TeamStatsMember {
                        name: &member.competitor.name,
                        rounds: [0, 1].map(|round| {
                            member
                                .jumps
                                .iter()
                                .find(|jump| jump.leg == leg && jump.round == round)
                                .copied()
                        }),
                    })
                    .collect(),
            });
        }
    }
    pages
}

pub(crate) fn render_stats(
    cx: &mut dyn UiCanvas,
    resources: &ResourcesRef,
    runtime: &TeamCupRuntime,
    page: usize,
    hill_background: bool,
) {
    let pages = build_stats_pages(runtime);
    let page_idx = page.min(pages.len().saturating_sub(1));
    let Some(stats) = pages.get(page_idx) else {
        return render_empty_stats(cx, resources, hill_background);
    };

    paint_results_background(cx, hill_background);
    let lang = &resources.langbase;
    cx.text(
        (8, 6),
        FONT_BODY,
        &format!("{} {}", lang.tr(93), stats.team_name),
    );

    let hill = resources
        .hills
        .hill(stats.hill_idx)
        .map(|hill| format!("{} K{}", hill.name, hill.kr))
        .unwrap_or_default();
    cx.text(
        (8, 23),
        FONT_TEAL,
        &format!("EVENT {}  {hill}", stats.leg + 1),
    );
    cx.text(
        (8, 34),
        FONT_BODY,
        &format!(
            "{}: {}. / {}   {}: {}. / {}",
            lang.tr(107),
            stats.event_rank,
            format_decimal(stats.event_score),
            lang.tr(110),
            stats.cumulative_rank,
            stats.cumulative_points
        ),
    );
    cx.text((8, 49), FONT_TEAL, lang.tr(106));
    cx.right_text(
        (200, 49),
        FONT_TEAL,
        &format!("R1 {} / {}", lang.tr(109), lang.tr(108)),
    );
    cx.right_text(
        (310, 49),
        FONT_TEAL,
        &format!("R2 {} / {}", lang.tr(109), lang.tr(108)),
    );

    for (idx, member) in stats.members.iter().enumerate() {
        let y = 65 + idx as i32 * 27;
        cx.text(
            (8, y),
            FONT_BODY,
            &shorten_name(member.name, &resources.font, 92),
        );
        render_round(cx, member.rounds[0], 200, y);
        render_round(cx, member.rounds[1], 310, y);
    }

    render_page_hints(
        cx,
        page_idx,
        pages.len(),
        &resources.langbase,
        PageHintLayout::Bottom,
    );
}

fn render_round(cx: &mut dyn UiCanvas, round: Option<TeamCupJumpResult>, x: i32, y: i32) {
    let text = match round {
        Some(round) if round.distance > 0.0 => format!(
            "{} / {}",
            format_decimal(round.distance),
            format_decimal(round.score)
        ),
        Some(_) => "DQ/DNS".to_string(),
        None => "-".to_string(),
    };
    cx.right_text((x, y), FONT_BODY, &text);
}

fn render_empty_stats(cx: &mut dyn UiCanvas, resources: &ResourcesRef, hill_background: bool) {
    paint_results_background(cx, hill_background);
    cx.text((8, 6), FONT_BODY, resources.langbase.tr(93));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::core::competitor::Competitor;
    use crate::competition::team_cup::types::{TeamCupJumpResult, TeamCupMember, TeamCupTeam};

    fn runtime(human_teams: usize) -> TeamCupRuntime {
        let teams = (0..2)
            .map(|team_idx| TeamCupTeam {
                name: format!("Team {team_idx}"),
                members: (0..4)
                    .map(|member_idx| TeamCupMember {
                        competitor: Competitor::computer(
                            team_idx * 4 + member_idx,
                            member_idx,
                            format!("Member {member_idx}"),
                            Some(team_idx),
                        ),
                        jumps: vec![TeamCupJumpResult {
                            leg: 0,
                            round: 0,
                            distance: 100.0,
                            score: 120.0,
                            gate: 15,
                        }],
                    })
                    .collect(),
                leg_score: 480.0,
                cup_points: 0,
                is_human_team: team_idx < human_teams,
            })
            .collect();
        TeamCupRuntime::new(teams, vec![0; 6], vec![0, 1])
    }

    #[test]
    fn stats_pages_cover_one_human_team() {
        let runtime = runtime(1);
        let pages = build_stats_pages(&runtime);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].team_name, "Team 0");
        assert_eq!(pages[0].members.len(), 4);
    }

    #[test]
    fn stats_pages_cover_two_human_teams() {
        let runtime = runtime(2);
        let pages = build_stats_pages(&runtime);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].team_name, "Team 0");
        assert_eq!(pages[1].team_name, "Team 1");
    }
}
