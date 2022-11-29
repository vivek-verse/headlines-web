use eframe::epaint::Vec2;
use headlines::Headlines;

fn main() {
    tracing_subscriber::fmt::init();
    let app = Headlines::new();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(540.0, 960.0));
    eframe::run_native("Headlines", native_options, Box::new(|_| Box::new(app)));
}
