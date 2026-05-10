pub extern crate engine;

pub mod parsers;
pub mod loaders;

pub fn run() -> Result<(), String> {
    let mut renderer = engine::video::Renderer::new()?;

    let main_menu = loaders::main_menu::MainMenu::load()?;
    renderer.set_palette(main_menu.palette);

    while renderer.running() {
        renderer.poll_input();
        renderer.blit(&main_menu.pixels);
        renderer.present()?;
        renderer.wait_frame();
    }

    Ok(())
}