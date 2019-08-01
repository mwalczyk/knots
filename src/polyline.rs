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

    fn clamp_point_to(&self, point_to_clamp: &Vector3<f32>) -> Vector3<f32> {
        let mut clamped_point = Vector3::zero();

        let min_x;
        let min_y;
        let min_z;
        let max_x;
        let max_y;
        let max_z;

        if self.a.x <= self.b.x
        {
            min_x = self.a.x;
            max_x = self.b.x;
        }
        else
        {
            min_x = self.b.x;
            max_x = self.a.x;
        }

        if self.a.y <= self.b.y
        {
            min_y = self.a.y;
            max_y = self.b.y;
        }
        else
        {
            min_y = self.b.y;
            max_y = self.a.y;
        }

        if self.a.z <= self.b.z
        {
            min_z = self.a.z;
            max_z = self.b.z;
        }
        else
        {
            min_z = self.b.z;
            max_z = self.a.z;
        }

        clamped_point.x = if point_to_clamp.x < min_x {
            min_x
        } else {
            if point_to_clamp.x > max_x {
                max_x
            }
            else {
                point_to_clamp.x
            }
        };

        clamped_point.y = if point_to_clamp.y < min_y {
            min_y
        } else {
            if point_to_clamp.y > max_y {
                max_y
            }
            else {
                point_to_clamp.y
            }
        };

        clamped_point.x = if point_to_clamp.z < min_z {
            min_z
        } else {
            if point_to_clamp.z > max_z {
                max_z
            }
            else {
                point_to_clamp.z
            }
        };

        clamped_point
    }

    pub fn distance_between(&self, other: &Segment) -> Option<Segment> {
        let p1 = self.a;
        let p2 = self.b;
        let p3 = other.a;
        let p4 = other.b;
        let d1 = p2 - p1;
        let d2 = p4 - p3;

        let eq1nCoeff = (d1.x * d2.x) + (d1.y * d2.y) + (d1.z * d2.z);
        let eq1mCoeff = (-(d1.x * d1.x) - (d1.y * d1.y) - (d1.z * d1.z));
        let eq1Const = ((d1.x * p3.x) - (d1.x * p1.x) + (d1.y * p3.y) - (d1.y * p1.y) + (d1.z * p3.z) - (d1.z * p1.z));
        let eq2nCoeff = ((d2.x * d2.x) + (d2.y * d2.y) + (d2.z * d2.z));
        let eq2mCoeff = -(d1.x * d2.x) - (d1.y * d2.y) - (d1.z * d2.z);
        let eq2Const = ((d2.x * p3.x) - (d2.x * p1.x) + (d2.y * p3.y) - (d2.y * p2.y) + (d2.z * p3.z) - (d2.z * p1.z));

        let mut M = vec![
            vec![eq1nCoeff, eq1mCoeff, -eq1Const],
            vec![eq2nCoeff, eq2mCoeff, -eq2Const]
        ];

        let  rowCount: usize = 2;

        // Pivoting
        for col in 0..(rowCount - 1) // was: for(int col = 0; col + 1 < rowCount; col++)
        {
            if M[col][col] == 0.0
            // check for zero coefficients
            {
                // find non-zero coefficient
                let mut swapRow = 0;

                for checkRow in (col + 1)..rowCount {
                    if M[checkRow][col] != 0.0 {
                        swapRow = checkRow;
                        break;
                    }
                }

                // found a non-zero coefficient?
                if M[swapRow][col] != 0.0 {
                    // yes, then swap it with the above
                    let mut tmp = vec![0.0; rowCount + 1];
                    for i in 0..rowCount + 1 {
                        tmp[i] = M[swapRow][i];
                        M[swapRow][i] = M[col][i];
                        M[col][i] = tmp[i];
                    }
                }
                else {
                    println!("Matrix has no unique solution");
                    return None;
                }
            }
        }

        // elimination
        for sourceRow in 0..(rowCount - 1) { // was: for (int sourceRow = 0; sourceRow + 1 < rowCount; sourceRow++)
            for destRow in (sourceRow + 1)..rowCount { // was: for (int destRow = sourceRow + 1; destRow < rowCount; destRow++)
                let df = M[sourceRow][sourceRow];
                let sf = M[destRow][sourceRow];

                for i in 0..(rowCount + 1) {
                    M[destRow][i] = M[destRow][i] * df - M[sourceRow][i] * sf;
                }
            }
        }

        println!("{:?}", M);

        // back-insertion
        for row in (0..=(rowCount - 1)).rev() { // was: for (int row = rowCount - 1; row >= 0; row--)
            let f = M[row][row];

            if f == 0.0 {
                println!("Returning none");
                return None;
            }

            for i in 0..(rowCount + 1) {
                M[row][i] /= f;
            }
            for destRow in 0..row {
                M[destRow][rowCount] -= M[destRow][row] * M[row][rowCount];
                M[destRow][row] = 0.0;
            }
        }

        let n = M[0][2];
        let m = M[1][2];
        let i1 = Vector3::new(p1.x + (m * d1.x), p1.y + (m * d1.y), p1.z + (m * d1.z));
        let i2 = Vector3::new(p3.x + (n * d2.x), p3.y + (n * d2.y), p3.z + (n * d2.z));
        let i1Clamped = self.clamp_point_to(&i1);
        let i2Clamped = other.clamp_point_to(&i1);

        Some(Segment::new(&i1Clamped, &i2Clamped))
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
    fn test_distance_between() {
        let segment_a = Segment::new(&Vector3::new(-1.0, 1.0, 0.0), &Vector3::new(-1.0, -1.0, 0.0));
        let segment_b = Segment::new(&Vector3::new(1.0, 1.0, 0.0), &Vector3::new(1.0, -1.0, 0.0));

        let joining = segment_a.distance_between(&segment_b);

        assert_eq!(joining.unwrap().length(), 2.0);
    }
}