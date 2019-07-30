use cgmath::{EuclideanSpace, Point3, Vector3};
use crate::constants;
use std::cmp::max;

/// A point of intersection along with a scalar `t` value.
type Intersection = (Vector3<f32>, f32, f32);

pub struct Segment {
    a: Vector3<f32>,
    b: Vector3<f32>,
}

impl Segment {
    pub fn new(a: &Vector3<f32>, b: &Vector3<f32>) -> Segment {
        Segment {
            a: *a,
            b: *b,
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

    pub fn intersect_2d(&self, other: &Segment) -> Option<Intersection> {
        let p0_x = self.a.x;
        let p0_y = self.a.y;
        let p1_x = self.b.x;
        let p1_y = self.b.y;

        let p2_x = other.a.x;
        let p2_y = other.a.y;
        let p3_x = other.b.x;
        let p3_y = other.b.y;

        let s1_x = p1_x - p0_x;
        let s1_y = p1_y - p0_y;
        let s2_x = p3_x - p2_x;
        let s2_y = p3_y - p2_y;

        let s = (-s1_y * (p0_x - p2_x) + s1_x * (p0_y - p2_y)) / (-s2_x * s1_y + s1_x * s2_y);
        let t = ( s2_x * (p0_y - p2_y) - s2_y * (p0_x - p2_x)) / (-s2_x * s1_y + s1_x * s2_y);

        // We use an epsilon here to avoid heads-to-tails intersections between segments
        if s >= constants::EPSILON && s <= (1.0 - constants::EPSILON) && t >= constants::EPSILON && t <= (1.0 - constants::EPSILON) {
            return Some((Vector3::new(p0_x + (t * s1_x), p0_y + (t * s1_y), 0.0), s, t));
        }

        // No collision was found
        None
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
        self.vertices.pop();
    }

    pub fn get_number_of_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the line segment between vertex `index` and `index + 1`.
    pub fn get_segment(&self, index: usize) -> Segment {
        Segment::new(&self.vertices[index], &self.vertices[index + 1])
    }

    pub fn find_intersections(&self) {
        unimplemented!();
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