use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use engine::oxide::input::{Key, UiEvent};
use engine::video::Renderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::app::assets::{self, LoadedAssets};
use crate::app::rendering::{FrameAssets, FrameRenderer};
use crate::app::router::{create_router, AppRouter};
use crate::app::state::load_app;
use crate::files::FileStore;
use crate::route::RouteTarget;

pub fn start(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let game = Rc::new(RefCell::new(WebGame::new(canvas_id).map_err(js_err)?));
    install_keyboard_events(Rc::clone(&game))?;
    start_animation_loop(game)
}

struct WebGame {
    renderer: Renderer,
    font: engine::oxide::Font,
    router: AppRouter,
    baked_sprites: engine::sprite::BakedSpriteTextures,
    main_background: engine::video::TextureId,
    pattern_texture: engine::video::TextureId,
    frame_renderer: FrameRenderer,
    events: Vec<UiEvent>,
}

impl WebGame {
    fn new(canvas_id: &str) -> Result<Self, String> {
        let mut renderer = Renderer::new_canvas(canvas_id)?;
        let files = Rc::new(FileStore::new(
            PathBuf::from("game/assets"),
            PathBuf::from("."),
        ));
        let LoadedAssets {
            content_store,
            font,
            main_background,
            baked_sprites,
            pattern_texture,
        } = assets::load(&files, &mut renderer)?;
        let app_load = load_app(Rc::clone(&files), font.clone(), content_store)?;
        let start_route = if app_load.state.config.language >= 0
            && (app_load.state.config.language as usize)
                < app_load.resources.langbase.languages.len()
        {
            RouteTarget::MainMenu
        } else {
            RouteTarget::Welcome
        };
        let router = create_router(
            app_load.resources,
            app_load.state,
            start_route,
            app_load.save_manager,
        );
        Ok(Self {
            renderer,
            font,
            router,
            baked_sprites,
            main_background,
            pattern_texture,
            frame_renderer: FrameRenderer::new(),
            events: Vec::new(),
        })
    }

    fn push_event(&mut self, event: UiEvent) {
        self.events.push(event);
    }

    fn frame(&mut self) -> Result<bool, String> {
        if self.router.current_route() == Some(&RouteTarget::Quit) {
            return Ok(false);
        }
        for event in self.events.drain(..) {
            self.router.handle_event(event);
        }
        self.router.update();
        self.frame_renderer.render(
            &mut self.renderer,
            FrameAssets {
                font: &self.font,
                baked_sprites: &self.baked_sprites,
                main_background: self.main_background,
                pattern_texture: self.pattern_texture,
            },
            &mut self.router,
        )?;
        Ok(true)
    }
}

fn install_keyboard_events(game: Rc<RefCell<WebGame>>) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| js_err("window not available"))?;
    let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(move |event| {
        if let Some(ui_event) = keyboard_event_to_ui_event(&event) {
            event.prevent_default();
            game.borrow_mut().push_event(ui_event);
        }
    }));
    window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn start_animation_loop(game: Rc<RefCell<WebGame>>) -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None::<Closure<dyn FnMut()>>));
    let g = Rc::clone(&f);
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let keep_running = match game.borrow_mut().frame() {
            Ok(keep_running) => keep_running,
            Err(err) => {
                web_sys::console::error_1(&JsValue::from_str(&err));
                false
            }
        };
        if keep_running {
            if let Err(err) = request_animation_frame(f.borrow().as_ref().unwrap()) {
                web_sys::console::error_1(&err);
            }
        }
    }) as Box<dyn FnMut()>));
    let result = request_animation_frame(g.borrow().as_ref().unwrap());
    result
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> Result<(), JsValue> {
    web_sys::window()
        .ok_or_else(|| js_err("window not available"))?
        .request_animation_frame(f.as_ref().unchecked_ref())
        .map(|_| ())
}

fn keyboard_event_to_ui_event(event: &web_sys::KeyboardEvent) -> Option<UiEvent> {
    let key = event.key();
    match key.as_str() {
        "ArrowUp" => Some(UiEvent::KeyDown(Key::Up)),
        "ArrowDown" => Some(UiEvent::KeyDown(Key::Down)),
        "ArrowLeft" => Some(UiEvent::KeyDown(Key::Left)),
        "ArrowRight" => Some(UiEvent::KeyDown(Key::Right)),
        "Home" => Some(UiEvent::KeyDown(Key::Home)),
        "End" => Some(UiEvent::KeyDown(Key::End)),
        "PageUp" => Some(UiEvent::KeyDown(Key::PageUp)),
        "PageDown" => Some(UiEvent::KeyDown(Key::PageDown)),
        "Enter" => Some(UiEvent::KeyDown(Key::Enter)),
        "Escape" => Some(UiEvent::KeyDown(Key::Escape)),
        "Backspace" => Some(UiEvent::KeyDown(Key::Backspace)),
        "Delete" => Some(UiEvent::KeyDown(Key::Delete)),
        "Tab" => Some(UiEvent::KeyDown(Key::Tab)),
        "F1" => Some(UiEvent::KeyDown(Key::F1)),
        "F2" => Some(UiEvent::KeyDown(Key::F2)),
        "F3" => Some(UiEvent::KeyDown(Key::F3)),
        "F4" => Some(UiEvent::KeyDown(Key::F4)),
        "F5" => Some(UiEvent::KeyDown(Key::F5)),
        "F6" => Some(UiEvent::KeyDown(Key::F6)),
        "F7" => Some(UiEvent::KeyDown(Key::F7)),
        "F8" => Some(UiEvent::KeyDown(Key::F8)),
        "F9" => Some(UiEvent::KeyDown(Key::F9)),
        "F10" => Some(UiEvent::KeyDown(Key::F10)),
        _ => key
            .chars()
            .next()
            .filter(|_| key.chars().count() == 1)
            .map(UiEvent::Text),
    }
}

fn js_err(msg: impl AsRef<str>) -> JsValue {
    JsValue::from_str(msg.as_ref())
}
