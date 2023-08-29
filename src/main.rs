use live_watch::live_watch::live_watch_app::LiveWatch;

// #[tokio::main]
fn main() {
    let option = eframe::NativeOptions::default();
    eframe::run_native(
        "live_watch",
        option,
        Box::new(|_cc| Box::new(LiveWatch::default())),
    )
    .unwrap();
}
