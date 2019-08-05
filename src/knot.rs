use cgmath::{InnerSpace, Vector3, Zero};
use crate::constants;
use crate::polyline::{Polyline, Segment};

pub trait Notation {

}

pub enum Crossing {
    Under,
    Over,
    Neither,
}

pub struct Spring {
    segment: Segment,
    k: f32,
    d: f32,
}

fn calculate_spring_force(spring_a: &Vector3<f32>, spring_b: &Vector3<f32>, k: f32, d: f32) -> Vector3<f32> {
    let mut direction = spring_a - spring_b;
    let distance = direction.magnitude();
    let force = Vector3::zero();

    // Avoid division by zero
    if distance.abs() < constants::EPSILON {
        return force;
    }

    // Hooke's law: `F = -k * (x - d)`
    direction = direction.normalize();
    direction * -k * (distance - d)
}

/// A struct representing a knot, which is a polyline embedded in 3-dimensional space
/// with a particular set of over- / under-crossings.
pub struct Knot {
    // The "rope" (polygonal line segment) that is knotted
    rope: Polyline,

    // Anchor (starting) positions
    anchors: Vec<Vector3<f32>>,

    // Positions
    p: Vec<Vector3<f32>>,

    // Velocities
    v: Vec<Vector3<f32>>,

    // Accelerations
    a: Vec<Vector3<f32>>,
}

impl Knot {
    pub fn new(rope: &Polyline, topology: Option<&Vec<Crossing>>) -> Knot {
        // Initialize buffers for physics simulation
        let anchors = rope.get_vertices().clone();
        let p = rope.get_vertices().clone();
        let v = vec![Vector3::zero(); rope.get_number_of_vertices()];
        let a = vec![Vector3::zero(); rope.get_number_of_vertices()];

        let knot = Knot {
            rope: rope.clone(),
            anchors,
            p,
            v,
            a
        };
        println!("Building knot with average segment length: {}", knot.rope.get_average_segment_length());
        knot
    }

    pub fn get_rope(&self) -> &Polyline {
        &self.rope
    }

    pub fn relax(&mut self) {
        // The (average?) length of each line segment ("stick"), prior to relaxation
        let starting_length = 0.5;

        // The mass of each node ("bead"): we leave this unchanged for now
        let mass = 1.0;

        // Velocity damping factor
        let damping = 0.5;

        // How much each bead wants to stay near its original position (`0.0` means that
        // we ignore this force)
        let anchor_weight = 0.0;

        // The maximum distance a bead can travel per time-step
        let d_max = starting_length * 0.025;

        // The closest any two sticks can be (note that this should be larger than `d_max`)
        let d_close = starting_length * 0.25;

        // Calculate forces
        for center_index in 0..self.p.len() {
            let mut force = Vector3::zero();

            // Get the indices of the left and right neighbors
            let (neighbor_l_index, neighbor_r_index) = self.rope.get_neighboring_indices_wrapped(center_index);

            // The "center" (i.e. current) bead
            let center = self.p[center_index];

            // Iterate over all potential neighbors
            for other_index in 0..self.p.len() {
                if other_index != center_index {
                    // `other_index` is not the same bead as `center_index`, so continue
                    // ...

                    // Grab the "other" bead, which may or may not be a neighbor to "center"
                    let other = self.p[other_index];

                    if other_index == neighbor_l_index  || other_index == neighbor_r_index {
                        // This is a neighboring bead: calculate the (attractive) mechanical spring force that
                        // will pull this bead towards `other`
                        let mut direction = other - center;
                        let r = direction.magnitude();
                        direction = direction.normalize();

                        if r.abs() < constants::EPSILON {
                            continue;
                        }

                        let beta = 1.0;
                        let H = 1.0;
                        force += direction * H * r.powf(1.0 + beta);
                    } else {
                        // This is NOT a neighboring bead: calculate the (repulsive) electrostatic force
                        let mut direction = center - other; // Reversed direction
                        let r = direction.magnitude();
                        direction = direction.normalize();

                        if r.abs() < constants::EPSILON {
                            continue;
                        }

                        let alpha = 4.0;
                        let K = 0.5;
                        force += direction * K * r.powf(-(2.0 + alpha));
                    }
                }
            }

            // Apply anchor force
            // ...
            //force += anchor_force * anchor_weight;

            // Apply force(s) to this bead: `F = m * a`
            self.a[center_index] += force / mass
        }

        // Integrate velocity (with damping)
        for index in 0..self.v.len() {
            self.v[index] += self.a[index];
            self.v[index] *= damping;

            // Zero out the acceleration for the next time step
            self.a[index] = Vector3::zero();
        }

        // Integrate positions
        for index in 0..self.p.len() {
            let old = self.p[index];
            let mut clamped = self.v[index];

            // Each particle can travel (at most) `d_max` units each time step
            if clamped.magnitude() > d_max {
                clamped = self.v[index].normalize() * d_max;
            }

            self.p[index] += clamped;

            // TODO: if moving this vertex is illegal, reset its position to `old`
            // Apply repulsive force away from neighboring segments
//            let mut repulsion = Vector3::new(0.0, 0.0, 0.0);
//            let mut number_of_interactions = 0;

            // Don't worry about the last (wrapped) segment for now...
//            if index > 0 && index < (self.rope.get_number_of_vertices() - 1) && false {
//
//                let segment_a = self.rope.get_segment(index);
//
//                for j in 0..self.rope.get_number_of_vertices() - 1 {
//
//                    // Don't test the current segment against itself or its immediate neighbors
//                    if j != index && j != (index - 1) && j != (index + 1)
//                    {
//                        let segment_b = self.rope.get_segment(j);
//
//                        let vector_between = segment_a.shortest_distance_between(&segment_b);
//                        if vector_between.magnitude() <= d_close {
//                            self.p[index] = old;
//
//                            //println!("Segment {} is too close to segment {}, with distance: {}", index, j, vector_between.magnitude());
//                            // Push segment A away from segment B: `to - from`
//                            //repulsion += vector_between;
//                            //number_of_interactions += 1;
//                        }
//
//                    }
//                }
//            }
//            if number_of_interactions >= 1 {
//                force += (repulsion / number_of_interactions as f32);
//            }
        }

        // Set new positions
        self.rope.set_vertices(&self.p);
    }

    /// Resets the physics simulation.
    pub fn reset(&mut self) {
        self.rope.set_vertices(&self.anchors);
        self.p = self.rope.get_vertices().clone();
        self.v = vec![Vector3::zero(); self.rope.get_number_of_vertices()];
        self.a = vec![Vector3::zero(); self.rope.get_number_of_vertices()];
    }

    pub fn find_crossings(&self) { unimplemented!() }

    pub fn get_number_of_crossings(&self) { unimplemented!() }

    pub fn get_dowker_notation(&self) { unimplemented!() }

    pub fn get_conway_notation(&self) { unimplemented!() }
}