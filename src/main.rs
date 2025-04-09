mod config;
mod lootitems;
mod savedata;
mod app;
mod apothrecipes;



// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Kynseed Save Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
