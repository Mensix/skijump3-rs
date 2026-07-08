use crate::gfx::theme::BLACK;
use crate::jump::types::JumpOutcome;
use crate::route::RouteTarget;
use crate::screen::{GameCx, GameScreen};
use crate::store::ResourcesRef;
use crate::views::jump::scene::JumpScene;
use crate::views::multiplayer::results as multiplayer_results;
use engine::oxide::{Blinker, Key, PaintCx, ScreenEventCx, UiEvent};
use net::protocol::{ClientMsg, NetEvent, ServerMsg};

pub(crate) struct MultiplayerJumpView {
    resources: ResourcesRef,
    blinker: Blinker,
    scene: Option<JumpScene>,
}

impl MultiplayerJumpView {
    pub(crate) fn new(resources: ResourcesRef) -> Self {
        Self {
            resources,
            blinker: Blinker::new(),
            scene: None,
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
        let hill_idx = mp.hill_idx;
        let player_id = mp.my_player_id;
        let round = mp.round;
        let start_gate = mp.start_gate;
        let p = &cx.state.profiles.profiles[0];
        let participant = crate::jump::config::JumpParticipant {
            id: player_id,
            ai_id: 0,
            name: p.name.clone(),
            real_name: p.real_name.clone(),
            suit_color: p.suit_color,
            ski_color: p.ski_color,
            team: Some(0),
            control: crate::jump::policy::JumperControl::Human,
        };
        let mut scene = JumpScene::new(
            self.resources.clone(),
            cx.state,
            hill_idx,
            start_gate,
            participant,
            crate::jump::policy::JumpPolicy::competition(),
        );
        scene.set_phase_label(format!("R{}", round + 1));
        self.scene = Some(scene);
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
                        mp.apply_round(round);
                        self.scene = None;
                    }
                    ServerMsg::StandingsUpdate { round, entries } => {
                        let mp = cx.state.mp_jump.as_mut().unwrap();
                        mp.sync_standings(*round, entries.clone());
                    }
                    ServerMsg::CompetitionDone { entries } => {
                        let mp = cx.state.mp_jump.as_mut().unwrap();
                        mp.apply_results(mp.round, entries.clone());
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
                mp.apply_jump_result(*id, *distance, *score);

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
            mp.apply_jump_result(mp.my_player_id, outcome.distance, outcome.score);
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::StandingsUpdate {
                    round: mp.round,
                    entries: mp.entries.clone(),
                });
            }
        }
        if let Some(ref client) = cx.state.net_client {
            if let Some(ref mut mp) = cx.state.mp_jump {
                mp.apply_jump_result(mp.my_player_id, outcome.distance, outcome.score);
            }
            client.send(jump_complete);
        }

        self.scene = None;
    }

    fn advance_round_if_ready(&mut self, cx: &mut GameCx<'_>) {
        if cx.state.net_host.is_none() {
            return;
        }
        let seed = cx.state.rng.random_i32(i32::MAX) as u32;
        let wind_position = cx.state.config.wind_position as u8;
        let mp = cx.state.mp_jump.as_mut().unwrap();
        if let Some(round) = mp.next_round(seed, wind_position) {
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::JumpRound(round));
            }
            self.scene = None;
        } else if let Some(round) = mp.next_leg(seed, wind_position) {
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::JumpRound(round));
                host.broadcast(ServerMsg::StandingsUpdate {
                    round: mp.round,
                    entries: mp.entries.clone(),
                });
            }
            self.scene = None;
        } else if mp.round_complete() {
            if let Some(ref host) = cx.state.net_host {
                host.broadcast(ServerMsg::CompetitionDone {
                    entries: mp.entries.clone(),
                });
            }
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

        let should_start_scene = cx
            .state
            .mp_jump
            .as_ref()
            .is_some_and(|mp| mp.can_local_jump());
        if self.scene.is_none() && should_start_scene {
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
            mp.hill_idx,
            mp.total_legs,
            mp.round,
            cx.state.net_host.is_some(),
        );
    }
}
