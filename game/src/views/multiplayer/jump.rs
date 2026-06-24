use crate::gfx::theme::BLACK;
use crate::jump::types::JumpOutcome;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::views::jump::scene::JumpScene;
use crate::views::multiplayer::results as multiplayer_results;
use engine::oxide::{Blinker, Key, PaintCx, ScreenEventCx, UiEvent};
use net::protocol::{ClientMsg, MPStandingEntry, NetEvent, ServerMsg};

pub(crate) struct MultiplayerJumpView {
    resources: ResourcesRef,
    blinker: Blinker,
    scene: Option<JumpScene>,
    round_done: bool,
}

impl MultiplayerJumpView {
    pub(crate) fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            blinker: Blinker::new(),
            scene: None,
            round_done: false,
        }
    }

    fn apply_result(entries: &mut Vec<MPStandingEntry>, outcome: &JumpOutcome, player_id: usize) {
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.player_id == player_id)
        {
            if entry.round1_len == 0.0 && entry.round1_score == 0.0 {
                entry.round1_len = outcome.distance;
                entry.round1_score = outcome.score;
            } else {
                entry.round2_len = outcome.distance;
                entry.round2_score = outcome.score;
            }
            entry.total_points = entry.round1_score + entry.round2_score;
        }
    }

    fn stop_session(cx: &mut GameCx<'_>) {
        cx.state.mp_jump = None;
        if let Some(ref host) = cx.state.net_host {
            host.stop();
            cx.state.net_host.take();
        }
        if cx.state.net_client.is_some() {
            cx.state.net_client.take();
        }
    }

    fn start_local_scene(&mut self, cx: &mut GameCx<'_>) {
        let Some(mp) = cx.state.mp_jump.as_ref() else {
            return;
        };
        let p = &cx.state.profiles.profiles[0];
        let participant = crate::jump::config::JumpParticipant {
            id: mp.my_player_id,
            ai_id: 0,
            name: p.name.clone(),
            real_name: p.real_name.clone(),
            suit_color: p.suit_color,
            ski_color: p.ski_color,
            team: Some(0),
            control: crate::jump::policy::JumperControl::Human,
        };
        self.scene = Some(JumpScene::new(
            self.resources.clone(),
            cx.state,
            mp.hill_idx,
            15,
            participant,
            crate::jump::policy::JumpPolicy::competition(),
        ));
    }

    fn poll_events(&mut self, cx: &mut GameCx<'_>) {
        let client_events: Vec<NetEvent> = cx
            .state
            .net_client
            .as_ref()
            .map(|client| std::iter::from_fn(|| client.event_rx.try_recv().ok()).collect())
            .unwrap_or_default();

        for event in &client_events {
            if let NetEvent::ServerMsg(msg) = event {
                match msg {
                    ServerMsg::JumpRound(round) => {
                        let mp = cx.state.mp_jump.as_mut().unwrap();
                        mp.round = round.round;
                        mp.hill_idx = round.hill_idx;
                        self.scene = None;
                        self.round_done = false;
                    }
                    ServerMsg::StandingsUpdate { round, entries } => {
                        let mp = cx.state.mp_jump.as_mut().unwrap();
                        mp.round = *round;
                        mp.entries = entries.clone();
                    }
                    ServerMsg::CompetitionDone { entries } => {
                        cx.state.mp_jump.as_mut().unwrap().entries = entries.clone();
                    }
                    _ => {}
                }
            }
        }

        let host_events: Vec<NetEvent> = cx
            .state
            .net_host
            .as_ref()
            .map(|host| std::iter::from_fn(|| host.event_rx.try_recv().ok()).collect())
            .unwrap_or_default();

        for event in &host_events {
            if let NetEvent::ClientJumpComplete {
                id,
                distance,
                score,
                landing_style: _,
                style_points: _,
                fall_type: _,
            } = event
            {
                let mp = cx.state.mp_jump.as_mut().unwrap();
                if let Some(entry) = mp.entries.iter_mut().find(|entry| entry.player_id == *id) {
                    if entry.round1_len == 0.0 && entry.round1_score == 0.0 {
                        entry.round1_len = *distance;
                        entry.round1_score = *score;
                    } else {
                        entry.round2_len = *distance;
                        entry.round2_score = *score;
                    }
                    entry.total_points = entry.round1_score + entry.round2_score;
                }

                if let Some(ref host) = cx.state.net_host {
                    host.broadcast(ServerMsg::StandingsUpdate {
                        round: mp.round,
                        entries: mp.entries.clone(),
                    });
                }
            }
        }
    }

    fn handle_finished_jump(&mut self, cx: &mut GameCx<'_>, outcome: &JumpOutcome) {
        let jump_complete = ClientMsg::JumpComplete {
            distance: outcome.distance,
            score: outcome.score,
            style_points: outcome.style_points,
            landing_style: outcome.landing_style as u8,
            fall_type: outcome.fall_type as u8,
        };

        if cx.state.net_host.is_some() {
            let mp = cx.state.mp_jump.as_mut().unwrap();
            Self::apply_result(&mut mp.entries, outcome, mp.my_player_id);
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::StandingsUpdate {
                    round: mp.round,
                    entries: mp.entries.clone(),
                });
            }
        }
        if let Some(ref client) = cx.state.net_client {
            client.send(jump_complete);
        }

        self.scene = None;
        self.round_done = true;
    }

    fn advance_round_if_ready(&mut self, cx: &mut GameCx<'_>) {
        if cx.state.net_host.is_none() {
            return;
        }
        let mp = cx.state.mp_jump.as_mut().unwrap();
        let all_done = mp.entries.iter().all(|entry| {
            if mp.round == 0 {
                entry.round1_len > 0.0
            } else {
                entry.round2_len > 0.0
            }
        });
        if !all_done {
            return;
        }

        if mp.round == 0 {
            mp.round = 1;
            let seed = cx.state.rng.random_i32(i32::MAX) as u32;
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::JumpRound(net::protocol::JumpRound {
                    hill_idx: mp.hill_idx,
                    round: 1,
                    wind_seed: seed,
                    wind_position: cx.state.config.wind_position as u8,
                    start_gate: 15,
                }));
            }
            self.scene = None;
            self.round_done = false;
        } else if let Some(ref host) = cx.state.net_host {
            host.broadcast(ServerMsg::CompetitionDone {
                entries: mp.entries.clone(),
            });
        }
    }
}

impl GameScreen for MultiplayerJumpView {
    fn update(&mut self, cx: &mut GameCx<'_>) {
        self.poll_events(cx);

        if let Some(outcome) = self.scene.as_ref().and_then(|scene| scene.outcome()) {
            self.handle_finished_jump(cx, &outcome);
            return;
        }

        if self.scene.is_none() && !self.round_done {
            self.start_local_scene(cx);
        }

        if let Some(ref mut scene) = self.scene {
            scene.update(cx.state);
        }
    }

    fn event(&mut self, cx: &mut GameCx<'_>, nav: &mut ScreenEventCx<RouteTarget>, event: UiEvent) {
        if let UiEvent::KeyDown(Key::Escape) = event {
            Self::stop_session(cx);
            nav.back();
            return;
        }

        if self.scene.is_some() {
            self.blinker.reset();
            if let Some(ref mut scene) = self.scene {
                let _ = scene.handle_jump_input(cx.state, event);
            }
            return;
        }

        if matches!(event, UiEvent::KeyDown(Key::Enter) | UiEvent::Text(' ')) {
            self.advance_round_if_ready(cx);
        }
    }

    fn paint(&mut self, cx: &mut GameCx<'_>, paint: &mut PaintCx<'_>) {
        let Some(ref mp) = cx.state.mp_jump else {
            return;
        };

        if let Some(ref mut scene) = self.scene {
            paint.fill((0, 0, 320, 200), BLACK);
            scene.render(paint, cx.state);
            return;
        }

        multiplayer_results::render_standings(
            paint,
            &self.resources,
            &mp.entries,
            mp.my_player_id,
            mp.round,
            cx.state.net_host.is_some(),
        );
    }
}
