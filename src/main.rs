mod app;

use app::App;
use iced::Application;

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}
