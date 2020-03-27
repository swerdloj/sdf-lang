extern crate sdf_lang;
use sdf_lang::runtime::application::Application;

fn main() {
    let mut app = Application::new((4, 5),
        "./tests/fragment.sdf");
    app.run();
}