use crate::competition::runtime::CompetitionRuntime;
use crate::competition::team_cup::types::{TeamCupResultsKind, TeamCupStandingsKind};
use crate::components::screen::new_screen_with_bg;
use crate::gfx::palette::{BG_TEAMCUP, FILL_HIGHLIGHT, FONT_DEFAULT, FONT_GOLD, FONT_HELP};
use crate::store::{ResourcesRef, StoreRef};
use crate::text::format::format_decimal;
use crate::text::layout::shorten_name;
use engine::ui::Element;

pub(crate) fn render(
    resources: &ResourcesRef,
    store: &StoreRef,
    results_kind: TeamCupResultsKind,
) -> Vec<Element> {
    let mut els = new_screen_with_bg(1, BG_TEAMCUP);
    let standings_kind = match results_kind {
        TeamCupResultsKind::Standings => TeamCupStandingsKind::Overall,
        TeamCupResultsKind::LegResults => TeamCupStandingsKind::Leg,
    };
    let (header, standings) = store
        .try_with_team_cup(|tc| {
            let standings = tc.standings_runtime(standings_kind);
            let leg = tc.current_leg + 1;
            let round = tc.current_round + 1;
            let jumper = tc.current_jumper_slot + 1;
            let header = match results_kind {
                TeamCupResultsKind::LegResults => format!(
                    "{} {} {} 6 - R {} - {} {}",
                    resources.langbase.lstr(81),
                    leg,
                    resources.langbase.lstr(8),
                    round,
                    resources.langbase.lstr(88),
                    jumper,
                ),
                TeamCupResultsKind::Standings => {
                    let text = resources.langbase.lstr(91);
                    format!("{} {} 6", text, leg)
                }
            };
            (header, standings)
        })
        .unwrap_or_default();

    els.push(Element::text(header, 30, 6, FONT_DEFAULT, false));

    let mut last_rank = 0usize;
    let mut y = 23i32;
    for (i, entry) in standings.iter().enumerate() {
        if i >= 15 {
            break;
        }
        let is_human = entry.is_human;

        // Pascal Entry: rank in col2, name in col1, points in col1
        if entry.rank != last_rank && entry.rank > 0 {
            let c = if is_human { FONT_GOLD } else { FILL_HIGHLIGHT };
            els.push(Element::text(format!("{}.", entry.rank), 24, y, c, true));
        }
        last_rank = entry.rank;

        let nc = if is_human { FONT_DEFAULT } else { FONT_HELP };
        let name = shorten_name(&entry.name, &resources.font, 122);
        els.push(Element::text(name, 32, y, nc, false));

        // Points as raw integer (no DOS tenths quirk)
        els.push(Element::right_text(
            format_decimal(entry.primary_score),
            184,
            y,
            nc,
        ));

        y += 10;
    }

    // Pascal WaitForKey: Done-) at bottom right
    els.push(Element::right_text(
        format!("{}-)", resources.langbase.lstr(248)),
        319,
        13,
        FONT_HELP,
    ));
    els
}
