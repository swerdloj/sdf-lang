extern crate sdf_lang;
use sdf_lang::runtime::application::Application;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // TODO: Better errors
    // TODO: Specify OpenGL version
    if args.len() != 2 {
        sdf_lang::exit!(format!("Error: Expected only one argument\nUsage: 'runtime PATH'"));
    }

    let mut app = Application::new((4, 5),
        &args[1]);

    app.run();
}