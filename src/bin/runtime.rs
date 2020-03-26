extern crate sdf_lang;

fn main() {
    let mut app = sdf_lang::runtime::application::Application::new((4, 5));
    app.run();
}