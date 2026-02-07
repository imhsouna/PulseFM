mod app;

use app::App;
use iced::Application;

fn main() -> iced::Result {
    println!("Pulse FM RDS Encoder");
    println!("--------------------");
    println!("Live MPX output at 192 kHz (float32). Select an output device in the UI.");
    println!("If you see no devices, click Refresh under Audio > Devices.");
    println!("WAV export is available under the Export tab (228 kHz float WAV).");
    println!("CLI: cargo run --bin pulse-fm-rds-cli -- --help");
    println!();
    App::run(iced::Settings::default())
}
