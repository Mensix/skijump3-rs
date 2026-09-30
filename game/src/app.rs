mod assets;
mod embedded;
mod fixed_step;
mod rendering;
mod router;
mod state;

use crate::app::router::{create_router, AppRouter};
use crate::files::{default_save_dir, find_asset_dir, FileStore};
use crate::route::RouteTarget;
use crate::ui::{Font, UiEvent};
use engine::context::Context;
use engine::oxide::UiEvent as EngineUiEvent;
use engine::sprite::BakedSpriteTextures;
use engine::video::TextureId;
use std::ffi::OsString;
use std::process::ExitCode;
use std::rc::Rc;

use self::assets::LoadedAssets;
use self::fixed_step::FixedStepClock;
use self::rendering::{FrameAssets, FrameRenderer};
use self::state::load_app;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupReset {
    None,
    Records,
    ZeroRecords,
    Config,
}

impl StartupReset {
    fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "-r" => Some(Self::Records),
            "-z" => Some(Self::ZeroRecords),
            "-c" => Some(Self::Config),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StartupOptions {
    reset: StartupReset,
    software_rendering: bool,
}

impl StartupOptions {
    fn from_args(args: impl Iterator<Item = OsString>) -> Result<StartupCommand, String> {
        let mut options = Self {
            reset: StartupReset::None,
            software_rendering: false,
        };
        let mut informational = None;
        let mut has_runtime_option = false;
        for arg in args.skip(1) {
            let arg = arg
                .into_string()
                .map_err(|_| "argument is not valid UTF-8".to_string())?;
            match arg.as_str() {
                "-h" | "--help" => set_informational(&mut informational, StartupCommand::Help)?,
                "-V" | "--version" => {
                    set_informational(&mut informational, StartupCommand::Version)?
                }
                "-s" | "--sw-rendering" => {
                    options.software_rendering = true;
                    has_runtime_option = true;
                }
                _ => {
                    let reset = StartupReset::from_arg(&arg)
                        .ok_or_else(|| format!("unknown option '{arg}'"))?;
                    if options.reset != StartupReset::None && options.reset != reset {
                        return Err("only one of '-r', '-z', and '-c' may be used".to_string());
                    }
                    options.reset = reset;
                    has_runtime_option = true;
                }
            }
        }
        if informational.is_some() && has_runtime_option {
            return Err(
                "'--help' and '--version' cannot be combined with other options".to_string(),
            );
        }
        Ok(informational.unwrap_or(StartupCommand::Run(options)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupCommand {
    Run(StartupOptions),
    Help,
    Version,
}

fn set_informational(
    current: &mut Option<StartupCommand>,
    command: StartupCommand,
) -> Result<(), String> {
    if current.is_some_and(|value| value != command) {
        return Err("'--help' and '--version' cannot be used together".to_string());
    }
    *current = Some(command);
    Ok(())
}

const HELP: &str = "Ski Jump 3 SDL port

Usage: skijump3 [OPTIONS]

Options:
  -r                    Reset hill records to bundled defaults
  -z                    Reset all hill records to zero
  -c                    Reset configuration to defaults
  -s, --sw-rendering    Use SDL software rendering
  -h, --help            Print help
  -V, --version         Print version";

pub struct Game {
    engine: Context,
    font: Font,
    router: AppRouter,
    baked_sprites: BakedSpriteTextures,
    main_background: TextureId,
    pattern_texture: TextureId,
    frame_renderer: FrameRenderer,
    update_clock: FixedStepClock,
}

impl Game {
    pub fn new() -> Option<Self> {
        Self::new_with_options(StartupOptions {
            reset: StartupReset::None,
            software_rendering: false,
        })
    }

    fn new_with_options(options: StartupOptions) -> Option<Self> {
        let mut ctx = match Context::new(options.software_rendering) {
            Ok(ctx) => ctx,
            Err(msg) => {
                eprintln!("error: {msg}");
                return None;
            }
        };

        let files = Rc::new(FileStore::new(find_asset_dir(), default_save_dir()));

        let Some(LoadedAssets {
            content_store,
            font,
            main_background,
            baked_sprites,
            pattern_texture,
        }) = assets::load(&files, &mut ctx.renderer)
        else {
            eprintln!(
                "error: failed to load game assets from '{}'.\n\nTry 'skijump3 --help' for more information.",
                find_asset_dir().display()
            );
            return None;
        };
        let app_load = match load_app(
            Rc::clone(&files),
            font.clone(),
            content_store,
            options.reset,
        ) {
            Ok(app_load) => app_load,
            Err(msg) => {
                eprintln!("error: {msg}");
                return None;
            }
        };

        let start_route = {
            if app_load
                .resources
                .langbase
                .is_saved_language_valid(app_load.state.config.language)
            {
                RouteTarget::MainMenu
            } else {
                RouteTarget::Welcome
            }
        };
        let router = create_router(
            app_load.resources,
            app_load.state,
            start_route,
            app_load.save_manager,
        );

        Some(Self {
            engine: ctx,
            font,
            router,
            baked_sprites,
            main_background,
            pattern_texture,
            frame_renderer: FrameRenderer::new(),
            update_clock: FixedStepClock::new(70, 5),
        })
    }

    pub fn run(&mut self) {
        self.run_loop();
        self.router.save_persistent_state();
    }

    fn run_loop(&mut self) {
        while self.engine.input.running() {
            if self.router.should_quit() {
                break;
            }
            self.handle_input();
            self.frame_renderer.prepare_frame();
            for _ in 0..self.update_clock.tick() {
                self.router.update();
            }
            for cue in self.router.drain_audio_cues() {
                self.engine.audio.play(cue);
            }
            self.render_frame();
        }
    }

    fn handle_input(&mut self) {
        for event in self.engine.input.drain_events() {
            self.router.handle_event(map_input_event(event));
        }
    }

    fn render_frame(&mut self) {
        let font = &self.font;
        self.frame_renderer.render(
            &mut self.engine.renderer,
            FrameAssets {
                font,
                baked_sprites: &self.baked_sprites,
                main_background: self.main_background,
                pattern_texture: self.pattern_texture,
            },
            &mut self.router,
        );
    }
}

fn map_input_event(event: EngineUiEvent) -> UiEvent {
    event.into()
}

pub fn run() -> ExitCode {
    let options = match StartupOptions::from_args(std::env::args_os()) {
        Ok(StartupCommand::Run(options)) => options,
        Ok(StartupCommand::Help) => {
            println!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Ok(StartupCommand::Version) => {
            println!("skijump3 {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("error: {error}\n\nTry 'skijump3 --help' for more information.");
            return ExitCode::from(2);
        }
    };
    let Some(mut game) = Game::new_with_options(options) else {
        return ExitCode::from(1);
    };
    game.run();
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<StartupCommand, String> {
        StartupOptions::from_args(args.iter().map(OsString::from))
    }

    #[test]
    fn parses_reset_switches() {
        assert_eq!(
            parse(&["skijump3", "-r"]),
            Ok(StartupCommand::Run(StartupOptions {
                reset: StartupReset::Records,
                software_rendering: false,
            }))
        );
        assert_eq!(
            parse(&["skijump3", "-z"]),
            Ok(StartupCommand::Run(StartupOptions {
                reset: StartupReset::ZeroRecords,
                software_rendering: false,
            }))
        );
        assert_eq!(
            parse(&["skijump3", "-c"]),
            Ok(StartupCommand::Run(StartupOptions {
                reset: StartupReset::Config,
                software_rendering: false,
            }))
        );
    }

    #[test]
    fn parses_software_rendering_short_and_long_options() {
        for args in [&["skijump3", "-s"][..], &["skijump3", "--sw-rendering"][..]] {
            assert_eq!(
                parse(args),
                Ok(StartupCommand::Run(StartupOptions {
                    reset: StartupReset::None,
                    software_rendering: true,
                }))
            );
        }
    }

    #[test]
    fn parses_reset_and_software_rendering_in_either_order() {
        assert_eq!(
            parse(&["skijump3", "-r", "-s"]),
            Ok(StartupCommand::Run(StartupOptions {
                reset: StartupReset::Records,
                software_rendering: true,
            }))
        );
        assert_eq!(
            parse(&["skijump3", "--sw-rendering", "-c"]),
            Ok(StartupCommand::Run(StartupOptions {
                reset: StartupReset::Config,
                software_rendering: true,
            }))
        );
    }

    #[test]
    fn rejects_unknown_and_conflicting_options() {
        assert!(parse(&["skijump3", "--unknown"]).is_err());
        assert!(parse(&["skijump3", "-r", "-c"]).is_err());
        assert!(parse(&["skijump3", "--help", "-s"]).is_err());
        assert!(parse(&["skijump3", "--help", "--version"]).is_err());
    }

    #[test]
    fn parses_help_and_version() {
        assert_eq!(parse(&["skijump3", "--help"]), Ok(StartupCommand::Help));
        assert_eq!(
            parse(&["skijump3", "--version"]),
            Ok(StartupCommand::Version)
        );
    }
}
