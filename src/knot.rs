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
    let direction = spring_a - spring_b;
    let distance = direction.magnitude();
    let force = Vector3::zero();

    // Avoid division by zero
    if distance.abs() < constants::EPSILON {
        return force;
    }

    // Hooke's law: `F = -k * (x - d)`
    direction.normalize();
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

        knot
    }

    pub fn get_rope(&self) -> &Polyline {
        &self.rope
    }

    pub fn relax(&mut self) {
        // The (average?) length of each line segment ("stick"), prior to relaxation
        let starting_length = 0.05;

        // The mass of each node ("bead"): we leave this unchanged for now
        let mass = 3.0;

        // Two coefficients of friction, which serve to slow down the simulation
        let friction_spring = 0.0;
        let friction_air = 0.0;

        // The spring constant in Hooke's law
        let k = 0.5;

        // The length that each stick will try to relax to
        let d = starting_length * 0.95;

        // Velocity damping factor
        let damping = 0.995;

        // How much each bead wants to stay near its original position (`0.0` means that
        // we ignore this force)
        let anchor_weight = 0.0;

        // The maximum distance a bead can travel per time-step
        let d_max = starting_length * 0.125;

        // The closest any two sticks can be
        let d_close = starting_length * 0.5;

        // Calculate forces
        for index in 0..self.p.len() {
            let index_a = index + 0;
            let mut index_b = index + 1;

            // Wrap indices
            if index_a == self.p.len() - 1 {
                index_b = 0;
            }

            let bead_a = self.p[index_a];
            let bead_b = self.p[index_b];

            let neighbor_force = calculate_spring_force(&bead_a, &bead_b, k, d);
            let anchor_force = calculate_spring_force(&bead_a, &self.anchors[index_a], k, d);

            let mut force = Vector3::zero();
            force += neighbor_force;

            // Add equal but opposite force to neighbor node
            self.a[index_b] += -force / mass;

            // Apply friction force
            // ...
            //force += -(self.v[index_a] - self.v[index_b]) * friction_spring;

            // Apply pseudo air resistance
            // ...
            //force += -self.v[index_a] * friction_air;

            // Apply gravity
            // ...
            //force += Vector3::new(0.0, -9.8, 0.0) * mass;

            // Apply anchor force
            force += anchor_force * anchor_weight;

            // Apply force to both springs: `F = m * a`
            self.a[index_a] += force / mass
        }

        // Integrate velocity (with damping)
        for index in 0..self.v.len() {
            self.v[index] += self.a[index];
            //self.v[index] *= damping;

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
            if index > 0 && index < (self.rope.get_number_of_vertices() - 1) {

                let segment_a = self.rope.get_segment(index);

                for j in 0..self.rope.get_number_of_vertices() - 1 {

                    // Don't test the current segment against itself or its immediate neighbors
                    if j != index && j != (index - 1) && j != (index + 1)
                    {
                        let segment_b = self.rope.get_segment(j);

                        let vector_between = segment_a.shortest_distance_between(&segment_b);
                        if vector_between.magnitude() <= d_close {
                            self.p[index] = old;

                            //println!("Segment {} is too close to segment {}, with distance: {}", index, j, vector_between.magnitude());
                            // Push segment A away from segment B: `to - from`
                            //repulsion += vector_between;
                            //number_of_interactions += 1;
                        }

                    }
                }
            }
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