use crate::polyline::Polyline;
use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Point3, Vector2, Vector3, Zero};
use gl;
use gl::types::*;
use std::mem;
use std::ptr;

enum Attribute {
    POSITIONS,
    COLORS,
    NORMALS,
    TEXCOORDS,
}

impl Attribute {
    pub fn get_index(&self) -> u32 {
        match *self {
            Attribute::POSITIONS => 0,
            Attribute::COLORS => 1,
            Attribute::NORMALS => 2,
            Attribute::TEXCOORDS => 3,
        }
    }

    pub fn get_memory_size(&self) -> usize {
        match *self {
            Attribute::TEXCOORDS => mem::size_of::<Vector2<f32>>(),
            _ => mem::size_of::<Vector3<f32>>(),
        }
    }

    pub fn get_element_count(&self) -> i32 {
        match *self {
            Attribute::TEXCOORDS => 2,
            _ => 3,
        }
    }
}

pub struct Mesh {
    vao: u32,
    vbo: u32,
    vertex_data: Vec<f32>,
    positions: Vec<Vector3<f32>>,
    colors: Option<Vec<Vector3<f32>>>,
    normals: Option<Vec<Vector3<f32>>>,
    texcoords: Option<Vec<Vector2<f32>>>,
}

impl Mesh {
    pub fn new(
        positions: &Vec<Vector3<f32>>,
        colors: Option<&Vec<Vector3<f32>>>,
        normals: Option<&Vec<Vector3<f32>>>,
        texcoords: Option<&Vec<Vector2<f32>>>,
    ) -> Mesh {
        let mut mesh = Mesh {
            vao: 0,
            vbo: 0,
            vertex_data: vec![],
            positions: positions.clone(),
            colors: None,
            normals: None,
            texcoords: None,
        };

        mesh.allocate();
        mesh
    }

    pub fn allocate(&mut self) {
        unsafe {
            // First, initialize the vertex array object
            gl::CreateVertexArrays(1, &mut self.vao);

            // Enable the `0`th attribute (positions), which is required
            //
            // Then, set the attribute format:
            // positions -> 3 floats (x, y, z)
            // colors -> 3 floats (r, g, b)
            // normals -> 3 floats (x, y, z)
            // texture coordinates -> 2 floats (u, v)
            self.enable_attribute(Attribute::POSITIONS);

            let mut total_size = mem::size_of::<Vector3<f32>>() * self.positions.len();
            let mut actual_stride = mem::size_of::<Vector3<f32>>();

            // Do the same for the other 3 attributes (if they are enabled), whilst calculating
            // the actual stride in between each vertex
            if let Some(colors) = &self.colors {
                assert_eq!(self.positions.len(), colors.len());
                total_size += mem::size_of::<Vector3<f32>>() * colors.len();
                actual_stride += mem::size_of::<Vector3<f32>>();
                self.enable_attribute(Attribute::COLORS);
            }
            if let Some(normals) = &self.normals {
                assert_eq!(self.positions.len(), normals.len());
                total_size += mem::size_of::<Vector3<f32>>() * normals.len();
                actual_stride += mem::size_of::<Vector3<f32>>();
                self.enable_attribute(Attribute::NORMALS);
            }
            if let Some(texcoords) = &self.texcoords {
                assert_eq!(self.positions.len(), texcoords.len());
                total_size += mem::size_of::<Vector2<f32>>() * texcoords.len();
                actual_stride += mem::size_of::<Vector2<f32>>();
                self.enable_attribute(Attribute::TEXCOORDS);
            }

            // Create the vertex buffer that will hold all interleaved vertex attributes
            self.generate_vertex_data();

            gl::CreateBuffers(1, &mut self.vbo);
            gl::NamedBufferData(
                self.vbo,
                total_size as isize,
                self.vertex_data.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW, // TODO: this shouldn't always be set to dynamic
            );

            gl::VertexArrayVertexBuffer(
                self.vao,
                0, // Binding index
                self.vbo,
                0, // Offset
                actual_stride as i32,
            );
        }
    }

    fn enable_attribute(&mut self, attribute: Attribute) {
        unsafe {
            gl::EnableVertexArrayAttrib(self.vao, attribute.get_index());
            gl::VertexArrayAttribFormat(self.vao, attribute.get_index(), attribute.get_element_count(), gl::FLOAT, gl::FALSE, 0);

            // All attributes are bound to index `0`
            gl::VertexArrayAttribBinding(self.vao, attribute.get_index(), 0);
        }
    }

    /// Generates a single buffer of floats that will contain all of the interleaved
    /// vertex attributes.
    fn generate_vertex_data(&mut self) {
        self.vertex_data = vec![];

        for index in 0..self.positions.len() {
            self.vertex_data.extend_from_slice(&[
                self.positions[index].x,
                self.positions[index].y,
                self.positions[index].z,
            ]);

            if let Some(colors) = &self.colors {
                self.vertex_data.extend_from_slice(&[
                    colors[index].x,
                    colors[index].y,
                    colors[index].z,
                ]);
            }
            if let Some(normals) = &self.normals {
                self.vertex_data.extend_from_slice(&[
                    normals[index].x,
                    normals[index].y,
                    normals[index].z,
                ]);
            }
            if let Some(texcoords) = &self.texcoords {
                self.vertex_data
                    .extend_from_slice(&[texcoords[index].x, texcoords[index].y]);
            }
        }
    }

    pub fn get_number_of_vertices(&self) -> usize {
        self.positions.len()
    }

    pub fn draw(&self, mode: GLenum) {
        // TODO: may need to vary the `count` parameter, based on the draw mode

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(mode, 0, self.get_number_of_vertices() as GLsizei);
        }
    }

    pub fn set_positions(&mut self, positions: &Vec<Vector3<f32>>) {
        // TODO: if other attributes are enabled this won't work

        let size_changed = self.get_number_of_vertices() != positions.len();

        // Always copy new positions to CPU-side buffer (small penalty here)
        self.positions = positions.clone();

        // Re-allocate GPU memory, if needed
        if size_changed {
            // The `generate_vertex_data()` function will be called automatically inside of `allocate()`,
            // so no need to do that again here...
            self.allocate();
        } else {
            self.generate_vertex_data();
        }

        // Upload new data to the GPU
        let size = (self.vertex_data.len() * (mem::size_of::<f32>() as usize)) as GLsizeiptr;

        unsafe {
            gl::NamedBufferSubData(
                self.vbo,
                0,
                size,
                self.vertex_data.as_ptr() as *const GLvoid,
            );
        }
    }
}

pub enum Plane {
    XY,
    YZ,
    ZX,
}
