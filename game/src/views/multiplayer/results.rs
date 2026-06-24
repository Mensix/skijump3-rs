use crate::competition::types::{CompetitionPhase, CupStyle, QualificationStatus};
use crate::gfx::theme::{FONT_BODY, FONT_GRAY, FONT_TEAL};
use crate::store::ResourcesRef;
use crate::views::jump::world_cup::results::{self, ResultsEntry, ResultsPage};
use engine::oxide::PaintCx;
use net::protocol::MPStandingEntry;

pub(crate) fn render_standings(
    cx: &mut PaintCx<'_>,
    resources: &ResourcesRef,
    entries: &[MPStandingEntry],
    own_player_id: usize,
    round: usize,
    is_host: bool,
) {
    let page = standings_page(entries, own_player_id);
    results::render_results_page(cx, &page, resources);

    cx.text(
        (30, 6),
        FONT_BODY,
        format!("MULTIPLAYER ROUND {}", round + 1),
    );
    cx.right_text((316, 5), FONT_GRAY, "Esc leave");
    if is_host && round < 1 && round_complete(entries, round) {
        cx.text((30, 190), FONT_TEAL, "Enter - next round");
    }
}

fn standings_page(entries: &[MPStandingEntry], own_player_id: usize) -> ResultsPage {
    let mut sorted: Vec<&MPStandingEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        b.total_points
            .partial_cmp(&a.total_points)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut last_points: Option<f64> = None;
    let mut last_rank = 0;
    let items = sorted
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let rank = if last_points == Some(entry.total_points) {
                last_rank
            } else {
                last_points = Some(entry.total_points);
                last_rank = idx + 1;
                last_rank
            };

            ResultsEntry {
                is_own: entry.player_id == own_player_id,
                rank,
                name: entry.name.clone(),
                points: entry.total_points,
                distance: entry.round1_len,
                distance2: entry.round2_len,
                qual: QualificationStatus::NotQualified,
                injury: 0,
                use_tenths: true,
            }
        })
        .collect();

    ResultsPage {
        // Round2Results gives the existing jump-results layout without WC round-1 qualification markers.
        phase: CompetitionPhase::Round2Results,
        style: CupStyle::WorldCup,
        page: 0,
        total_pages: 1,
        items,
        compact: false,
        prev_last_rank: 0,
    }
}

fn round_complete(entries: &[MPStandingEntry], round: usize) -> bool {
    entries.iter().all(|entry| {
        if round == 0 {
            entry.round1_len > 0.0
        } else {
            entry.round2_len > 0.0
        }
    })
}
