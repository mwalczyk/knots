use cgmath::{EuclideanSpace, Point3, Vector3};
use std::cmp::max;

pub struct Segment {
    a: Vector3<f32>,
    b: Vector3<f32>,
}

impl Segment {
    pub fn new() -> Segment {
        Segment {
            // TODO: this isn't correct
            a: Vector3::unit_x(),
            b: Vector3::unit_y(),
        }
    }

    pub fn midpoint(&self) -> Vector3<f32> {
        (self.a + self.b) / 2.0
    }

    pub fn point_at(&self, t: f32) -> Vector3<f32> {
        assert!(t > 0.0 && t < 1.0);

        // Get the direction vector pointing from `a` to `b`
        let d = self.b - self.a;

        self.a + d * t
    }

    pub fn intersect(&self, other: &Segment) -> Option<Vector3<f32>> {
        unimplemented!()
    }

    pub fn closest_points(&self, other: &Segment) -> Option<Segment> {
        unimplemented!()
    }
}

type BoundingBox = (Point3<f32>, Point3<f32>);

#[derive(Clone)]
pub struct Polyline {
    pub vertices: Vec<Vector3<f32>>,
}

impl Polyline {
    pub fn new() -> Polyline {
        Polyline {
            vertices: vec![],
        }
    }

    pub fn push_vertex(&mut self, v: &Vector3<f32>) {
        self.vertices.push(*v);
    }

    pub fn pop_vertex(&mut self) {
        unimplemented!()
    }

    /// Returns the line segment between vertex `index` and `index + 1`.
    pub fn get_segment(&self, index: usize) -> Segment {
        unimplemented!()
    }

    /// Reference: `https://github.com/openframeworks/openFrameworks/blob/master/libs/openFrameworks/graphics/ofPolyline.inl#L504`
    pub fn refine(&mut self, minimum_segment_length: f32) {
        unimplemented!()
    }

    /// Returns an AABB that encloses this polyline.
    pub fn bounding_box(&self) -> BoundingBox {
        unimplemented!()
    }
}