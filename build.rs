extern crate lalrpop;

fn main() {
    // FIXME: Ensure that SDL2.dll is placed when this is used
    // println!("cargo:rerun-if-changed=src/parse/parser.lalrpop");

    // Link SDL2 library
    let sdl2_path = r"C:/Development/SDL2/lib/x64";
    println!(r"cargo:rustc-link-search={}", sdl2_path);


    // Build the sdf-lang parser
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process_file("src/parse/parser.lalrpop")
        .unwrap();


    // Copy SDL2 runtime library into executable's path
    #[cfg(runtime)]
    {
        #[cfg(target_os = "windows")]
        {
            let sdl2_dll = std::path::Path::new("./SDL2.dll");
            if !sdl2_dll.exists() {
                std::fs::copy(format!("{}/{}", sdl2_path, "SDL2.dll"), sdl2_dll).unwrap();
            }
        }

        // TODO: Other operating systems
    }
}