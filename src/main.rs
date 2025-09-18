mod app;
mod core;

use app::MainApp;

fn main() {
    let main_app = MainApp::new();
    main_app.run();
}
