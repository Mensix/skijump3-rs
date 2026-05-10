fn main() {
    if let Err(e) = game::run() {
        eprintln!("Error: {}", e);
    }
}