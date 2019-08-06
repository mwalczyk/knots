#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_assignments)]
#![allow(unreachable_code)]
#![allow(unreachable_patterns)]
#![allow(non_snake_case)]

#![feature(clamp)]

// Should be able to do this, but the Intellij plugin doesn't support it yet...
//mod gl { include!(concat!(env!("OUT_DIR"), "/bindings.rs")); }
//mod gl { include!("../target/debug/build/gl-c987f7e774ed107e/out/bindings.rs"); }

extern crate gl;
extern crate cgmath;
extern crate csv;
extern crate glutin;

mod constants;
mod diagram;
mod interaction;
mod knot;
mod polyline;
mod program;
mod renderer;
mod tangle;

use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector3};
use crate::diagram::Diagram;
use crate::interaction::InteractionState;
use crate::polyline::Polyline;
use crate::program::Program;
use crate::renderer::Renderer;
use glutin::GlContext;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Clears the default OpenGL framebuffer (color and depth)
fn clear() {
    unsafe {
        gl::ClearColor(0.1, 0.05, 0.05, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

fn set_draw_state() {
    unsafe {
        gl::LineWidth(1.0);
        gl::PointSize(4.0);
        gl::Enable(gl::DEPTH_TEST);

        gl::DepthFunc(gl::LESS);
        gl::Disable(gl::CULL_FACE);
    }
}

/// Returns the string contents of the file at `path`.
pub fn load_file_as_string(path: &Path) -> String {
    let mut file = File::open(path).expect("File not found");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Something went wrong reading the file");

    contents
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(constants::WIDTH, constants::HEIGHT)
        .with_title("knots")
        .with_decorations(true);
    let context = glutin::ContextBuilder::new().with_multisampling(8);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // Load a knot diagram from a .csv file
    let file = Path::new("src/example_diagrams/test.csv");
    let diagram = Diagram::from_path(file);
    let mut knot = diagram.generate_knot();

    // Set up OpenGL shader programs for rendering
    let draw_program = Program::two_stage(
        load_file_as_string(Path::new("shaders/draw.vert")),
        load_file_as_string(Path::new("shaders/draw.frag")),
    ).unwrap();
    let mut renderer = Renderer::new();

    // Interaction
    let mut interaction = InteractionState::new();

    // Set up the model-view-projection (MVP) matrices
    let mut model = Matrix4::identity();
    let view = Matrix4::look_at(Point3::new(0.0, 0.0, 15.0), Point3::origin(), Vector3::unit_y());
    let fov = cgmath::Rad(std::f32::consts::FRAC_PI_4);
    let aspect = constants::WIDTH as f32 / constants::HEIGHT as f32;
    let projection = cgmath::perspective(fov, aspect, 0.1, 1000.0);

    // Turn on depth testing, etc.
    set_draw_state();

    let mut frame_count = 0;

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => {
                    println!("Shutting down the program...");
                },
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    // Store the normalized mouse position.
                    interaction.cursor_prev = interaction.cursor_curr;
                    interaction.cursor_curr.x = position.0 as f32 / constants::WIDTH as f32;
                    interaction.cursor_curr.y = position.1 as f32 / constants::HEIGHT as f32;

                    if interaction.lmouse_pressed {
                        let delta = interaction.get_mouse_delta() * constants::MOUSE_SENSITIVITY;

                        let rot_xz = Matrix4::from_angle_y(cgmath::Rad(delta.x));
                        let rot_yz = Matrix4::from_angle_x(cgmath::Rad(delta.y));

                        model = rot_xz * rot_yz * model;
                    }
                },
                glutin::WindowEvent::MouseInput { state, button, .. } => match button {
                    glutin::MouseButton::Left => {
                        if let glutin::ElementState::Pressed = state {
                            interaction.cursor_pressed = interaction.cursor_curr;
                            interaction.lmouse_pressed = true;
                        } else {
                            interaction.lmouse_pressed = false;
                        }
                    }
                    glutin::MouseButton::Right => {
                        if let glutin::ElementState::Pressed = state {
                            interaction.rmouse_pressed = true;
                        } else {
                            interaction.rmouse_pressed = false;
                        }
                    }
                    _ => (),
                },
                //glutin::WindowEvent::MouseInput { state, button, .. } => (),
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            glutin::ElementState::Pressed => match key {
                                glutin::VirtualKeyCode::R => {
                                    knot.reset();
                                    frame_count = 0;
                                },
                                glutin::VirtualKeyCode::S => {
                                   // knot.relax();
                                }
                                _ => (),
                            }
                            // Key released...
                            _ => (),
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        });
        clear();

        draw_program.bind();
        draw_program.uniform_matrix_4f("u_model", &model);
        draw_program.uniform_matrix_4f("u_view", &view);
        draw_program.uniform_matrix_4f("u_projection", &projection);
        //renderer.draw_polyline(knot.get_rope());

        if frame_count < 100 {
            knot.relax();
        }

        renderer.draw_tube(knot.get_rope());

        gl_window.swap_buffers().unwrap();

        frame_count += 1;
    }
}
