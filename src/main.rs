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

mod constants;
mod diagram;
mod graphics;
mod interaction;
mod knot;
mod tangle;
mod utils;

use crate::diagram::{Axis, Cardinality, CromwellMove, Diagram, Direction};
use crate::graphics::program::Program;
use crate::interaction::InteractionState;
use cgmath::{EuclideanSpace, Matrix4, Point3, SquareMatrix, Vector3};
use glutin::GlContext;
use std::path::Path;

/// Clears the default OpenGL framebuffer (color and depth)
fn clear() {
    unsafe {
        gl::ClearColor(0.12, 0.1, 0.1, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
}

/// Sets the draw state (enables depth testing, etc.)
fn set_draw_state() {
    unsafe {
        // Allow us to set the point size programmatically in our vertex shaders
        gl::Enable(gl::PROGRAM_POINT_SIZE);

        // Turn on depth testing
        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LESS);

        // Turn on back-face culling
        gl::Enable(gl::CULL_FACE);
    }
}

fn main() {
    // Setup the windowing environment
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
    let path = Path::new("diagrams/legendrian.csv");
    let mut knots = vec![
        Diagram::from_path(path)
            .unwrap()
            .apply_move(CromwellMove::Stabilization {
                cardinality: Cardinality::SW,
                i: 3,
                j: 2,
            })
            .unwrap()
            .apply_move(CromwellMove::Translation(Direction::Left))
            .unwrap()
            .generate_knot(),
        Diagram::from_path(path)
            .unwrap()
            .apply_move(CromwellMove::Stabilization {
                cardinality: Cardinality::SE,
                i: 3,
                j: 2,
            })
            .unwrap()
            .generate_knot(),
        Diagram::from_path(path)
            .unwrap()
            .apply_move(CromwellMove::Stabilization {
                cardinality: Cardinality::NW,
                i: 3,
                j: 2,
            })
            .unwrap()
            .apply_move(CromwellMove::Translation(Direction::Up))
            .unwrap()
            .generate_knot(),
    ];

    // Set up OpenGL shader programs for rendering
    let draw_program = Program::two_stage(
        utils::load_file_as_string(Path::new("shaders/draw.vert")),
        utils::load_file_as_string(Path::new("shaders/draw.frag")),
    )
    .unwrap();

    // Interaction (mouse clicks, etc.)
    let mut interaction = InteractionState::new();

    // Set up the model-view-projection (MVP) matrices
    let mut models = vec![
        Matrix4::from_translation(Vector3::new(-15.0, 0.0, 0.0)),
        Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
        Matrix4::from_translation(Vector3::new(15.0, 0.0, 0.0)),
    ];
    let view = Matrix4::look_at(
        Point3::new(0.0, 0.0, 45.0),
        Point3::origin(),
        Vector3::unit_y(),
    );
    let projection = cgmath::perspective(
        cgmath::Rad(std::f32::consts::FRAC_PI_4),
        constants::WIDTH as f32 / constants::HEIGHT as f32,
        0.1,
        1000.0,
    );

    // Turn on depth testing, etc. then bind the shader program
    set_draw_state();
    draw_program.bind();
    draw_program.uniform_matrix_4f("u_view", &view);
    draw_program.uniform_matrix_4f("u_projection", &projection);

    loop {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => {
                    println!("Shutting down the program...");
                }
                glutin::WindowEvent::MouseMoved { position, .. } => {
                    interaction.cursor_prev = interaction.cursor_curr;
                    interaction.cursor_curr.x = position.0 as f32 / constants::WIDTH as f32;
                    interaction.cursor_curr.y = position.1 as f32 / constants::HEIGHT as f32;

                    if interaction.lmouse_pressed {
                        let delta = interaction.get_mouse_delta() * constants::MOUSE_SENSITIVITY;

                        let rot_xz = Matrix4::from_angle_y(cgmath::Rad(delta.x));
                        let rot_yz = Matrix4::from_angle_x(cgmath::Rad(delta.y));

                        for model in models.iter_mut() {
                            *model = rot_xz * rot_yz * *model;
                        }
                    }
                }
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
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        match input.state {
                            glutin::ElementState::Pressed => match key {
                                glutin::VirtualKeyCode::R => {
                                    for knot in knots.iter_mut() {
                                        knot.reset();
                                    }
                                }
                                glutin::VirtualKeyCode::S => {
                                    let path = Path::new("frame.png");
                                    utils::save_frame(path, constants::WIDTH, constants::HEIGHT);
                                }
                                glutin::VirtualKeyCode::F => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                                },
                                glutin::VirtualKeyCode::W => unsafe {
                                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                                },
                                glutin::VirtualKeyCode::H => {
                                    models = vec![
                                        Matrix4::from_translation(Vector3::new(-14.0, 0.0, 0.0)),
                                        Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
                                        Matrix4::from_translation(Vector3::new(14.0, 0.0, 0.0)),
                                    ];
                                }
                                _ => (),
                            },
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

        draw_program.uniform_2f("u_mouse", &interaction.cursor_curr);

        // Relax each knot and draw it
        for (knot, model) in knots.iter_mut().zip(models.iter()) {
            draw_program.uniform_matrix_4f("u_model", model);
            knot.relax();
            knot.draw(true);
        }

        gl_window.swap_buffers().unwrap();
    }
}
