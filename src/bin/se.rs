extern crate semantic_editor;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    match semantic_editor::core::run() {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Fatal error: {}", e);
            for cause in e.causes() {
                eprintln!("   caused by {}", cause);
            }
        }
    }
}
