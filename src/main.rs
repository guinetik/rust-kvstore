mod db;
mod log;
mod app;
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let app:app::App = app::App::new(arguments);
    app.init();
}