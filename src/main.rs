#[cfg(not(target_arch = "wasm32"))]
fn main() {
    game::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {}
