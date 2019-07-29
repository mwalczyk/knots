use cgmath::{EuclideanSpace, Point3, Vector2, Vector3};
use crate::polyline::Polyline;
use gl;
use gl::types::*;
use std::mem;
use std::ptr;

pub struct Mesh {
    vao: u32,
    vbo: u32,
    positions: Vec<Vector3<f32>>,
    colors: Vec<Vector3<f32>>,
    uvs: Vec<Vector2<f32>>,
    normals: Vec<Vector3<f32>>,
}

pub struct Renderer {
    vao: u32,
    vbo: u32,
    // See: `https://github.com/openframeworks/openFrameworks/blob/master/libs/openFrameworks/gl/ofGLProgrammableRenderer.h#L241`
    //
    // circle_mesh: Mesh,
    // rectangle_mesh: Mesh,
    // polyline_mesh: Mesh,
    //
    // shader_color: Program,
    // shader_texture: Program,
    // shader_uvs: Program,
    // etc.
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut renderer = Renderer{
            vao: 0,
            vbo: 0,
        };

        unsafe {
            gl::CreateVertexArrays(1, &mut renderer.vao);

            // Set up attribute #0: position
            const ATTR_POS: u32 = 0;
            const BINDING_POS: u32 = 0;
            gl::EnableVertexArrayAttrib(renderer.vao, ATTR_POS);
            gl::VertexArrayAttribFormat(
                renderer.vao,
                ATTR_POS,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(renderer.vao, ATTR_POS, BINDING_POS);

            // Set up attribute #1: color
            const ATTR_COL: u32 = 1;
            const BINDING_COL: u32 = 0; // TODO: both attributes are sourced from the buffer bound to index 0 right now
            gl::EnableVertexArrayAttrib(renderer.vao, ATTR_COL);
            gl::VertexArrayAttribFormat(
                renderer.vao,
                ATTR_COL,
                3,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vector3<f32>>() as u32, // Relative offset
            );
            gl::VertexArrayAttribBinding(renderer.vao, ATTR_COL, BINDING_COL);

            // Create the VBO for storing vertex data
            let max_vertices = 5000usize;
            let size = mem::size_of::<Vector3<f32>>() * max_vertices;

            gl::CreateBuffers(1, &mut renderer.vbo);
            gl::NamedBufferData(
                renderer.vbo,
                size as isize,
                ptr::null() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl::VertexArrayVertexBuffer(
                renderer.vao,
                0, // Binding index
                renderer.vbo,
                0, // Offset
                (mem::size_of::<Vector3<f32>>() * 2 as usize) as i32
            );

            gl::VertexArrayVertexBuffer(
                renderer.vao,
                0, // Binding index
                renderer.vbo,
                0, // Offset
                (mem::size_of::<Vector3<f32>>() * 2 as usize) as i32
            );
        }

        renderer
    }

    pub fn draw_polyline(&self, line: &Polyline) {
        unsafe {
            gl::BindVertexArray(self.vao);

            let size = (line.vertices.len() * mem::size_of::<Vector3<f32>>() * 2) as GLsizeiptr;
            let mut attributes = vec![];
            for (i, position) in line.vertices.iter().enumerate() {
                let pct = i as f32 / line.vertices.len() as f32;
                let color = Vector3::new(pct, pct, pct);
                attributes.push(*position);
                attributes.push(color);
            }

            gl::NamedBufferSubData(
                self.vbo,
                0,
                size,
                attributes.as_ptr() as *const GLvoid
            );

            gl::DrawArrays(gl::LINE_STRIP, 0, line.vertices.len() as GLsizei);
        }
    }
}