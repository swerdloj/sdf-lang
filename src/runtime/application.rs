use sdl2::event::{Event, WindowEvent};
use super::opengl;

pub struct Application {
    sdl2_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
}

impl Application {
    /// Set up an SDL2 application using OpenGL
    pub fn new(gl_version: (u8, u8)) -> Self {
        let sdl2_context = sdl2::init().expect("Failed to initialize SDL2");
        let video_subsystem = sdl2_context.video().expect("Failed to initialize video subsystem");
    
        let window = video_subsystem.window("sdf-lang", 800, 600)
                                    .position_centered()
                                    .resizable()
                                    .opengl()
                                    .build()
                                    .expect("Failed to create window");

        let gl_attributes = video_subsystem.gl_attr();

        // TODO: Allow the user to configure these
        gl_attributes.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attributes.set_context_version(gl_version.0, gl_version.1);

        let gl_context = window.gl_create_context().expect("Failed to create OpenGL context");
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

        // Default stuff
        opengl::set_clear_color(0.1, 0.1, 0.2);
        opengl::set_viewport(800, 600);

        Application {
            sdl2_context,
            video_subsystem,
            window,
            gl_context,
        }
    }

    /// Start the windowed application
    pub fn run(&mut self) {
        let mut event_pump = self.sdl2_context.event_pump().expect("Failed to obtain event pump");

        loop {
            // Clear screen buffer
            opengl::clear(gl::COLOR_BUFFER_BIT);

            // Update
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        crate::exit!("Quitting...".to_owned());
                    }

                    Event::Window { win_event: WindowEvent::SizeChanged(width, height), .. } => {
                        opengl::set_viewport(width, height);
                    }

                    _ => {}
                }
            }

            // Render screen buffer
            self.window.gl_swap_window();
        }
    }
}