use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3, Zero};
use crate::constants;
use std::cmp::max;

/// A point of intersection along with scalar `s` and `t` values.
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

    pub fn get_start(&self) -> &Vector3<f32> {
        &self.a
    }

    pub fn get_end(&self) -> &Vector3<f32> {
        &self.b
    }

    /// Returns the (scalar) length of this line segment.
    pub fn length(&self) -> f32 {
        (self.b - self.a).magnitude()
    }

    pub fn midpoint(&self) -> Vector3<f32> {
        (self.a + self.b) / 2.0
    }

    pub fn point_at(&self, t: f32) -> Vector3<f32> {
        assert!(t >= 0.0 && t <= 1.0);

        // Get the direction vector pointing from `a` to `b`
        let d = self.b - self.a;

        self.a + d * t
    }

    /// Reference: `http://geomalgorithms.com/a07-_distance.html#dist3D_Segment_to_Segment`
    pub fn shortest_distance_between(&self, other: &Segment) -> Vector3<f32> {
        let u = self.b - self.a;
        let v = other.b - other.a;
        let w = self.a - other.a;
        let a = u.dot(u); // always >= 0
        let b = u.dot(v);
        let c = v.dot(v); // always >= 0
        let d = u.dot(w);
        let e = v.dot(w);
        let D = a*c - b*b; // always >= 0

        let mut sc: f32 = 0.0;
        let mut sN: f32 = 0.0;
        let mut sD = D;       // sc = sN / sD, default sD = D >= 0
        let mut tc: f32 = 0.0;
        let mut tN: f32= 0.0;
        let mut tD = D;       // tc = tN / tD, default tD = D >= 0

        // compute the line parameters of the two closest points
        if D < constants::EPSILON {
            // the lines are almost parallel
            sN = 0.0; // force using point P0 on segment self
            sD = 1.0; // to prevent possible division by 0.0 later
            tN = e;
            tD = c;
        }
        else {
            // get the closest points on the infinite lines
            sN = b * e - c * d;
            tN = a * e - b * d;
            if sN < 0.0 {
                // sc < 0 => the s = 0 edge is visible
                sN = 0.0;
                tN = e;
                tD = c;
            }
            else if sN > sD {
                // sc > 1  => the s = 1 edge is visible
                sN = sD;
                tN = e + b;
                tD = c;
            }
        }

        if tN < 0.0 {
            // tc < 0 => the t = 0 edge is visible
            tN = 0.0;
            // Recompute `sc` for this edge
            if -d < 0.0 {
                sN = 0.0;
            }
            else if -d > a {
                sN = sD;
            }
            else {
                sN = -d;
                sD = a;
            }
        }
        else if tN > tD {
            // tc > 1  => the t = 1 edge is visible
            tN = tD;
            // Recompute `sc` for this edge
            if (-d + b) < 0.0 {
                sN = 0.0;
            }
            else if (-d + b) > a {
                sN = sD;
            }
            else {
                sN = (-d +  b);
                sD = a;
            }
        }
        // finally do the division to get sc and tc
        sc = if sN.abs() < constants::EPSILON {
            0.0
        } else {
            sN / sD
        };

        tc = if tN.abs() < constants::EPSILON {
            0.0
        } else {
            tN / tD
        };

        // Get the vector difference of the two closest points
        let vector_between_closest_points = w + (sc * u) - (tc * v);  // = self(sc) - other(tc)

        //println!("Closest point on first segment: {:?}", self.a + sc * u);
        //println!("Closest point on second segment: {:?}", other.a + tc * v);

        vector_between_closest_points
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

    pub fn get_vertices(&self) -> &Vec<Vector3<f32>> {
        &self.vertices
    }

    pub fn set_vertices(&mut self, vertices: &Vec<Vector3<f32>>) {
        self.vertices = vertices.clone();
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
        Segment::new(
            &self.vertices[(index + 0)],// % self.vertices.len()],
            &self.vertices[(index + 1)])// % self.vertices.len()])
    }

    pub fn get_average_segment_length(&self) -> f32 {
        let mut total = 0.0;
        let mut count = 0;

        for index in 0..self.get_number_of_vertices() - 1 {
            let segment = self.get_segment(index);
            total += segment.length();
            count += 1;
        }

        total / count as f32
    }

    pub fn find_intersections(&self) {
        unimplemented!();
    }

    /// Reference: `https://github.com/openframeworks/openFrameworks/blob/master/libs/openFrameworks/graphics/ofPolyline.inl#L504`
    pub fn refine(&mut self, minimum_segment_length: f32)  -> Polyline {
        let mut refined = Polyline::new();

        for index in 0..self.get_number_of_vertices() - 1 {
            let segment = self.get_segment(index);

            // Add the first point
            refined.push_vertex(segment.get_start());

            // Calculate how many vertices we will need to add between the
            // start and end points of the original, unrefined segment
            let number_of_subdivisions = (segment.length() / minimum_segment_length) as usize;

            // Add extra vertices
            for division in 1..number_of_subdivisions {
                let t = division as f32 / number_of_subdivisions as f32;
                refined.push_vertex(&segment.point_at(t));
            }

            // Finally, add the second point
            refined.push_vertex(segment.get_end());
        }

        refined
    }

    /// Returns an AABB that encloses this polyline.
    pub fn bounding_box(&self) -> BoundingBox {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_between_0() {
        let segment_a = Segment::new(&Vector3::new(-1.0, 1.0, 0.0), &Vector3::new(-1.0, -1.0, 0.0));
        let segment_b = Segment::new(&Vector3::new(1.0, 1.0, 0.0), &Vector3::new(1.0, -1.0, 0.0));

        let shortest_distance = segment_a.shortest_distance_between(&segment_b);

        assert_eq!(shortest_distance.magnitude(), 2.0);
    }

    #[test]
    fn test_distance_between_1() {
        let segment_a = Segment::new(&Vector3::new(-1.0, 1.0, 0.0), &Vector3::new(0.0, -1.0, 0.0));
        let segment_b = Segment::new(&Vector3::new(1.0, 1.0, 0.0), &Vector3::new(1.0, -1.0, 0.0));

        let shortest_distance = segment_a.shortest_distance_between(&segment_b);

        assert_eq!(shortest_distance.magnitude(), 1.10);
    }
}