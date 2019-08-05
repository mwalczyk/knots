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

    /// Returns the first endpoint of this line segment.
    pub fn get_start(&self) -> &Vector3<f32> {
        &self.a
    }

    /// Returns the second endpoint of this line segment.
    pub fn get_end(&self) -> &Vector3<f32> {
        &self.b
    }

    /// Returns the (scalar) length of this line segment.
    pub fn length(&self) -> f32 {
        (self.b - self.a).magnitude()
    }

    /// Returns the midpoint of this line segment.
    pub fn midpoint(&self) -> Vector3<f32> {
        (self.a + self.b) / 2.0
    }

    /// Returns the point at `t` along this line segment, where a value
    /// of `0.0` corresponds to `self.a` and a value of `1.0` corresponds
    /// to `self.b`.
    pub fn point_at(&self, t: f32) -> Vector3<f32> {
        assert!(t >= 0.0 && t <= 1.0);

        // Get the direction vector pointing from `a` to `b`
        let d = self.b - self.a;

        self.a + d * t
    }

    /// Returns the shortest distance between this segment and `other`.
    ///
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

    /// Returns the indices of the "left" and "right" neighbors to the vertex at
    /// index `center_index`. The polyline is assumed to be "closed" so that the
    /// "left" neighbor of the vertex at index `0` is the index of the last vertex
    /// in this polyline, etc.
    pub fn get_neighboring_indices_wrapped(&self, center_index: usize) -> (usize, usize) {
        let neighbor_l_index = if center_index == 0 {
            self.get_number_of_vertices() - 1
        } else {
            center_index - 1
        };
        let neighbor_r_index = if center_index == self.get_number_of_vertices() - 1 {
            0
        } else {
            center_index + 1
        };

        (neighbor_l_index, neighbor_r_index)
    }

    pub fn set_vertices(&mut self, vertices: &Vec<Vector3<f32>>) {
        self.vertices = vertices.clone();
    }

    /// Adds a new vertex `v` to the end of the polyline.
    pub fn push_vertex(&mut self, v: &Vector3<f32>) {
        self.vertices.push(*v);
    }

    /// Removes the last vertex from the polyline.
    pub fn pop_vertex(&mut self) {
        self.vertices.pop();
    }

    /// Returns the number of vertices that make up this polyline.
    pub fn get_number_of_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn point_at(&self, t: f32) -> Vector3<f32> {
        assert!(t >= 0.0 && t <= 1.0);

        let desired_length = self.length() * t;
        let mut traversed = 0.0;
        let mut point = Vector3::zero();

        for index in 0..self.get_number_of_vertices() - 1 {
            let segment = self.get_segment(index);
            traversed += segment.length();

            if traversed >= desired_length {
                // We know that the point lies on this segment somewhere...
                // o ----- o -x---- o ------ o
                let along_segment = traversed - desired_length;

                point = segment.point_at((segment.length() - along_segment) / segment.length());
                break;
            }
        }
        point
    }

    pub fn length(&self) -> f32 {
        let mut total = 0.0;

        for index in 0..self.get_number_of_vertices() - 1 {
            let segment = self.get_segment(index);
            total += segment.length();
        }
        total
    }

    /// Returns the line segment between vertex `index` and `index + 1`.
    pub fn get_segment(&self, index: usize) -> Segment {
        Segment::new(
            &self.vertices[(index + 0)],// % self.vertices.len()],
            &self.vertices[(index + 1)])// % self.vertices.len()])
    }

    /// Returns the average length of the line segments that make up this
    /// polyline.
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

        assert_eq!(shortest_distance.magnitude(), 1.0);
    }

    #[test]
    fn test_point_at_0() {
        let mut polyline = Polyline::new();
        polyline.push_vertex(&Vector3::new(0.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(1.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(2.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(3.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(4.0, 0.0, 0.0));

        // 0 --- 1 --- 2 --- 3 --- 4
        let point = polyline.point_at(0.25);
        assert_eq!(point, Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_point_at_1() {
        let mut polyline = Polyline::new();
        polyline.push_vertex(&Vector3::new(0.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(1.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(2.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(3.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(4.0, 0.0, 0.0));

        // 0 --- 1 --- 2 --- 3 --- 4
        let point = polyline.point_at(0.125);
        assert_eq!(point, Vector3::new(0.5, 0.0, 0.0));
    }

    #[test]
    fn test_point_at_2() {
        let mut polyline = Polyline::new();
        polyline.push_vertex(&Vector3::new(0.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(1.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(2.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(3.0, 0.0, 0.0));
        polyline.push_vertex(&Vector3::new(4.0, 0.0, 0.0));

        // 0 --- 1 --- 2 --- 3 --- 4
        let point = polyline.point_at(1.0);
        assert_eq!(point, Vector3::new(4.0, 0.0, 0.0));
    }
}