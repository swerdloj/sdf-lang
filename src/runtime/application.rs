use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use super::opengl;

use std::path::PathBuf;

pub struct Application {
    sdl2_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,

    timer: sdl2::TimerSubsystem,

    runtime: super::Runtime,
}

impl Application {
    /// Set up an SDL2 application using OpenGL
    pub fn new<P: Into<PathBuf>>(gl_version: (u8, u8), fragment_shader_path: P) -> Self {
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

        let timer = sdl2_context.timer().expect("Failed to initialize timer subsystem");

        Application {
            sdl2_context,
            video_subsystem,
            window,
            gl_context,
            timer,
            runtime: super::Runtime::new(fragment_shader_path),
        }
    }

    /// Start the windowed application
    pub fn run(&mut self) {
        // Initialize the fragment shader
        self.runtime.reload_shader();
        self.runtime.set_window_dimensions(800, 600);

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

                    Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                        println!("Reloading shader");

                        self.runtime.reload_shader();
                        let (width, height) = self.window.size();
                        self.runtime.set_window_dimensions(width as i32, height as i32);
                    }

                    Event::Window { win_event: WindowEvent::SizeChanged(width, height), .. } => {
                        println!("Window resized to {}x{}", &width, &height);

                        opengl::set_viewport(width, height);
                        self.runtime.set_window_dimensions(width, height);
                    }

                    _ => {}
                }
            }

            // Time is in seconds
            self.runtime.set_time(self.timer.ticks() as f32 / 1000f32);

            self.runtime.render();

            // Render screen buffer
            self.window.gl_swap_window();
        }
    }
}