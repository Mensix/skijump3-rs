mod consts;

use std::time::{Duration, Instant};

use sdl2::EventPump;
use sdl2::Sdl;
use sdl2::keyboard::Keycode;

pub struct Engine {
    #[allow(dead_code)]
    sdl: Sdl,
    canvas: sdl2::render::WindowCanvas,
    event_pump: EventPump,
    running: bool,
    last_tick: Instant,
}

impl Engine {
    pub fn new() -> Result<Self, String> {
        let sdl = sdl2::init().map_err(|e| e.to_string())?;

        let video = sdl.video().map_err(|e| e.to_string())?;

        let window = video
            .window(
                "Ski Jump International v3",
                consts::WINDOW_W,
                consts::WINDOW_H,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        let event_pump = sdl.event_pump().map_err(|e| e.to_string())?;

        Ok(Self {
            sdl,
            canvas,
            event_pump,
            running: true,
            last_tick: Instant::now(),
        })
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn poll_input(&mut self) {
        use sdl2::event::Event;
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => self.running = false,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn wait_frame(&mut self) {
        let elapsed = self.last_tick.elapsed();
        let frame_time = Duration::from_secs_f64(1.0 / consts::TARGET_FPS as f64);
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
        self.last_tick = Instant::now();
    }

    pub fn render(&mut self) -> Result<(), String> {
        use sdl2::pixels::Color;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut engine = Engine::new()?;

    while engine.running() {
        engine.poll_input();
        engine.render()?;
        engine.wait_frame();
    }

    Ok(())
}