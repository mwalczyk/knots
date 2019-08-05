use cgmath::{EuclideanSpace, InnerSpace, Matrix3, Point3, Vector2, Vector3, Zero};
use crate::polyline::Polyline;
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
    pub fn get_index(&self) -> usize {
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

    pub fn get_element_count(&self) -> usize {
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
    pub fn new(positions: &Vec<Vector3<f32>>,
               colors: Option<&Vec<Vector3<f32>>>,
               normals: Option<&Vec<Vector3<f32>>>,
               texcoords: Option<&Vec<Vector2<f32>>>) -> Mesh {

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

            // Then, set up vertex buffers
            // #0: positions
            // #1: colors
            // #2: normals
            // #3: texture coordinates
            const ATTRIBUTE_POSITION: u32 = 0;
            const BINDING_POSITION: u32 = 0;
            const ATTRIBUTE_COLOR: u32 = 1;
            const BINDING_COLOR: u32 = 0;
            const ATTRIBUTE_NORMALS: u32 = 2;
            const BINDING_NORMALS: u32 = 0;
            const ATTRIBUTE_TEXCOORDS: u32 = 3;
            const BINDING_TEXCOORDS: u32 = 0;

            // Enable the `0`th attribute (positions), which is required
            gl::EnableVertexArrayAttrib(self.vao, ATTRIBUTE_POSITION);

            // Set the attribute format:
            // positions -> 3 floats (x, y, z)
            // colors -> 3 floats (r, g, b)
            // normals -> 3 floats (x, y, z)
            // texture coordinates -> 2 floats (u, v)
            gl::VertexArrayAttribFormat(
                self.vao,
                ATTRIBUTE_POSITION,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
            );
            gl::VertexArrayAttribBinding(self.vao, ATTRIBUTE_POSITION, BINDING_POSITION);

            let mut total_size = mem::size_of::<Vector3<f32>>() * self.positions.len();
            let mut actual_stride = mem::size_of::<Vector3<f32>>();

            // Do the same for the other 3 attributes (if they are enabled), whilst calculating
            // the actual stride in between each vertex
            if let Some(colors) = &self.colors {
                assert_eq!(self.positions.len(), colors.len());
                total_size += mem::size_of::<Vector3<f32>>() * colors.len();
                actual_stride += mem::size_of::<Vector3<f32>>();
                gl::VertexArrayAttribFormat(self.vao, ATTRIBUTE_COLOR, 3, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(self.vao, ATTRIBUTE_COLOR, BINDING_COLOR);
            }
            if let Some(normals) = &self.normals {
                assert_eq!(self.positions.len(), normals.len());
                total_size += mem::size_of::<Vector3<f32>>() * normals.len();
                actual_stride += mem::size_of::<Vector3<f32>>();
                gl::VertexArrayAttribFormat(self.vao, ATTRIBUTE_NORMALS, 3, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(self.vao, ATTRIBUTE_NORMALS, BINDING_NORMALS);
            }
            if let Some(texcoords) = &self.texcoords {
                assert_eq!(self.positions.len(), texcoords.len());
                total_size += mem::size_of::<Vector2<f32>>() * texcoords.len();
                actual_stride += mem::size_of::<Vector2<f32>>();
                gl::VertexArrayAttribFormat(self.vao, ATTRIBUTE_TEXCOORDS, 2, gl::FLOAT, gl::FALSE, 0);
                gl::VertexArrayAttribBinding(self.vao, ATTRIBUTE_TEXCOORDS, BINDING_TEXCOORDS);
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
                actual_stride as i32
            );
        }
    }

    /// Generates a single buffer of floats that will contain all of the interleaved
    /// vertex attributes.
    fn generate_vertex_data(&mut self) {
        self.vertex_data = vec![];

        for index in 0..self.positions.len() {
            self.vertex_data.extend_from_slice(&[self.positions[index].x, self.positions[index].y, self.positions[index].z]);

            if let Some(colors) = &self.colors {
                self.vertex_data.extend_from_slice(&[colors[index].x, colors[index].y, colors[index].z]);
            }
            if let Some(normals) = &self.normals {
                self.vertex_data.extend_from_slice(&[normals[index].x, normals[index].y, normals[index].z]);
            }
            if let Some(texcoords) = &self.texcoords {
                self.vertex_data.extend_from_slice(&[texcoords[index].x, texcoords[index].y]);
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
            gl::NamedBufferSubData(self.vbo, 0, size, self.vertex_data.as_ptr() as *const GLvoid);
        }
    }
}

pub enum Plane {
    XY,
    YZ,
    ZX,
}

pub struct Renderer {
    polyline_cache: Mesh,
    tube_cache: Mesh,
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
        let renderer = Renderer{
            polyline_cache: Mesh::new(&vec![], None, None, None),
            tube_cache: Mesh::new(&vec![], None, None, None),
        };

        renderer
    }

    pub fn draw_circle(&mut self, radius: f32) {
        unimplemented!()
    }

    pub fn draw_rectangle(&mut self, width: f32, height: f32) {
        unimplemented!()
    }

    pub fn draw_square(&mut self, size: f32) {
        self.draw_rectangle(size, size);
    }

    pub fn draw_line(&mut self, a: &Point3<f32>, b: &Point3<f32>) {
        unimplemented!()
    }

    pub fn draw_polyline(&mut self, line: &Polyline) {
        self.polyline_cache.set_positions(line.get_vertices());
        self.polyline_cache.draw(gl::LINE_LOOP);
        self.polyline_cache.draw(gl::POINTS);
    }

    /// Reference: `https://github.com/openframeworks/openFrameworks/blob/master/libs/openFrameworks/graphics/ofPolyline.inl#L1069`
    pub fn draw_tube(&mut self, line: &Polyline) {
        // Path guided extrusion
        let mut circle_vertices = vec![];
        let circle_center = Vector3::new(0.0, 0.0, 0.0);
        let circle_radius = 0.15;
        let number_of_segments = 6;

        // First, gather the vertices for a circle centered at the origin on the XY-plane
        for index in 0..number_of_segments {
            let theta = 2.0 * 3.1415926 * (index as f32 / number_of_segments as f32);
            let x = circle_radius * theta.cos();
            let y = circle_radius * theta.sin();
            circle_vertices.push(Vector3::new(x + circle_center.x, 0.0, y + circle_center.y));//0.0));
        }

        let mut tube_vertices = vec![];

        // Then, at each vertex of the polyline, do the following:
        //
        // 1. Calculate the tangent vector
        // 2. Calculate the normal vector
        // 3. Use (1) and (2) to calculate the binormal vector
        // 4. Translate the circle "stamp" to the current vertex
        // 5. Rotate the circle "stamp" to lie in the XY-plane of this coordinate system
        // 6. Emit these vertices and connect them to the previous "stamp"
        for center_index in 0..line.get_number_of_vertices() {
            let (neighbor_l_index, neighbor_r_index) = line.get_neighboring_indices_wrapped(center_index);

            let center = line.get_vertices()[center_index];
            let neighbor_l = line.get_vertices()[neighbor_l_index];
            let neighbor_r = line.get_vertices()[neighbor_r_index];

            let v1 = (neighbor_l - center).normalize(); // Vector that points towards the left neighbor
            let v2 = (neighbor_r - center).normalize(); // Vector that points towards the right neighbor

            // TODO: this probably won't work as we begin to move the polyline off of the XY-plane
            let right = Vector3::new(0.0, 0.0, -1.0);

            // Construct a basis around `center`, using the tangent, normal, and binormal vectors
            let tangent = if (v2 - v1).magnitude2() > 0.0 {
                (v2 - v1).normalize()
            } else {
                -v1
            };
            let normal = right.cross(tangent);
            let binormal = tangent.cross(normal);

            for point in circle_vertices.iter() {
                // TODO: not sure about the column order here...also, this could be done with a single `Matrix4`
                let transformation = Matrix3::from_cols(binormal, tangent, normal);
                let mut transformed_point = transformation * point;
                transformed_point += center;

                tube_vertices.push(transformed_point);
            }

            if center_index > 0 {
                // Connect to previous "stamp"
            }
        }

        self.tube_cache.set_positions(&tube_vertices);
        self.tube_cache.draw(gl::POINTS);
    }
}