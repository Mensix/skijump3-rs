use crate::competition::team_cup::types::{TeamCupResultsKind, TeamCupStandingsKind};
use crate::gfx::theme::{BG_TEAM, BLACK, FILL_GOLD, FILL_GRAY, FONT_BODY, FONT_GOLD, FONT_GRAY};
use crate::store::{GameState, ResourcesRef};
use crate::text::format::format_decimal;
use crate::text::layout::shorten_name;
use engine::oxide::PaintCx;

pub(crate) fn render(
    cx: &mut PaintCx<'_>,
    resources: &ResourcesRef,
    state: &GameState,
    results_kind: TeamCupResultsKind,
) {
    cx.fill((0, 0, 320, 200), BLACK);
    cx.fill((0, 0, 320, 19), FILL_GRAY);
    cx.pattern_fill((0, 20, 320, 180), BG_TEAM);
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
                        // Pascal: lstr(92) = "The Team Cup is over!"
                        resources.langbase.lstr(92).to_string()
                    } else {
                        format!(
                            "{} {} {} 6 - R {} - {} {}",
                            resources.langbase.lstr(81),
                            leg,
                            resources.langbase.lstr(8),
                            round,
                            resources.langbase.lstr(88),
                            jumper,
                        )
                    }
                }
                TeamCupResultsKind::Standings => {
                    let text = resources.langbase.lstr(91);
                    format!("{} {} 6", text, leg)
                }
            };
            Some((header, standings))
        })
        .unwrap_or_default();

    cx.text((30, 6), FONT_BODY, header);

    let mut last_rank = 0usize;
    let mut y = 23i32;
    for (i, entry) in standings.iter().enumerate() {
        if i >= 15 {
            break;
        }
        let is_human = entry.is_human;

        // Pascal Entry: rank in col2, name in col1, points in col1
        if entry.rank != last_rank && entry.rank > 0 {
            let c = if is_human { FONT_GOLD } else { FILL_GOLD };
            cx.right_text((24, y), c, format!("{}.", entry.rank));
        }
        last_rank = entry.rank;

        let nc = if is_human { FONT_BODY } else { FONT_GRAY };
        let name = shorten_name(&entry.name, &resources.font, 122);
        cx.text((32, y), nc, name);

        // Points as raw integer (no DOS tenths quirk)
        cx.right_text((184, y), nc, format_decimal(entry.primary_score));

        y += 10;
    }

    // Pascal WaitForKey: Done-) at bottom right
    cx.right_text(
        (319, 13),
        FONT_GRAY,
        format!("{}-)", resources.langbase.lstr(248)),
    );
}
