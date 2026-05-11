pub extern crate engine;

pub mod parsers;
pub mod loaders;

pub fn run() -> Result<(), String> {
    let mut renderer = engine::video::Renderer::new()?;

    let assets = loaders::main_menu::MainMenuAssets::load()?;
    let (pixels, palette) = assets.compose();
    renderer.set_palette(palette);

    while renderer.running() {
        renderer.poll_input();
        renderer.blit(&pixels);
        renderer.present()?;
        renderer.wait_frame();
    }

    Ok(())
}