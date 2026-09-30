use crate::data::profile::Profile;
use crate::jump::types::JumpOutcome;

pub(crate) fn apply_profile_jump_side_effects(
    profile: &mut Profile,
    hill_idx: usize,
    hill_file: String,
    hill_display: String,
    outcome: JumpOutcome,
    started: bool,
    world_cup_hill_display: Option<String>,
) {
    if !started || outcome.aborted {
        return;
    }

    profile.total_jumps += 1;
    let distance = outcome.distance.max(0.0);
    if let Some(world_cup_hill_display) = world_cup_hill_display {
        if distance > profile.best_wc_jump {
            profile.best_wc_jump = distance;
            profile.best_wc_hill_idx = hill_idx;
            profile.best_wc_hill_display = world_cup_hill_display;
        }
    }
    if distance > profile.best_jump {
        profile.best_jump = distance;
        profile.besthill_idx = hill_idx;
        profile.best_hill_file = hill_file;
        profile.best_hill_display = hill_display;
    }
}
