#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_assignments)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(bare_trait_objects)]
// Should be able to do this, but the Intellij plugin doesn't support it yet...
//mod gl { include!(concat!(env!("OUT_DIR"), "/bindings.rs")); }
mod gl { include!("../target/debug/build/gl-c987f7e774ed107e/out/bindings.rs"); }

extern crate cgmath;
extern crate csv;
extern crate glutin;

mod constants;
mod diagram;
mod knot;
mod program;

use glutin::GlContext;
use std::path::Path;
use crate::program::Program;
use crate::diagram::Diagram;

/// Clears the default OpenGL framebuffer (color and depth)
fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

fn main() {

    // Set up windowing and event loop
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(constants::WIDTH, constants::HEIGHT)
        .with_title("knots")
        .with_decorations(true);
    let context = glutin::ContextBuilder::new().with_multisampling(8);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let file = Path::new("src/example_diagrams/test.csv");
    let diagram = Diagram::from_path(file);

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                //glutin::WindowEvent::Closed => (),
                //glutin::WindowEvent::MouseMoved => (),
                //glutin::WindowEvent::MouseInput { state, button, .. } => (),
                //glutin::WindowEvent::KeyboardInput { input, .. } => (),
                _ => (),
            },
            _ => (),
        });
        clear();

        gl_window.swap_buffers().unwrap();
    }
}
