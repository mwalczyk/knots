use crate::constants;
use crate::graphics::mesh::Mesh;
use crate::graphics::polyline::{Polyline, Segment};
use cgmath::{InnerSpace, Vector3, Zero};

pub trait Notation {
    fn generate(&self) -> &str;
}

pub enum Crossing {
    Under,
    Over,
    Neither,
}

struct Stick<'a> {
    start: &'a Bead,
    end: &'a Bead,
    //k: f32,
    //d: f32,
}

#[derive(PartialEq)]
struct Bead {
    // The position of the bead in 3-space
    position: Vector3<f32>,

    // The velocity of the bead
    velocity: Vector3<f32>,

    // The acceleration of the bead
    acceleration: Vector3<f32>,

    // The index of the polyline vertex corresponding to this bead
    index: usize,

    // The cached index of this bead's left neighbor in the underlying polyline
    neighbor_l_index: usize,

    // The cached index of this bead's right neighbor in the underlying polyline
    neighbor_r_index: usize,

    // Whether or not this bead is active in the physics simulation
    is_stuck: bool,
}

impl Bead {
    fn new(
        position: &Vector3<f32>,
        index: usize,
        neighbor_l_index: usize,
        neighbor_r_index: usize,
    ) -> Bead {
        Bead {
            position: *position,
            velocity: Vector3::zero(),
            acceleration: Vector3::zero(),
            index,
            neighbor_l_index,
            neighbor_r_index,
            is_stuck: false,
        }
    }

    /// Returns `true` if this bead and `other` are neighbors and `false` otherwise.
    fn are_neighbors(&self, other: &Bead) -> bool {
        self.index == other.neighbor_l_index || self.index == other.neighbor_r_index
    }

    /// Set the left and right neighbor indices for this bead.
    fn set_neighbor_indices(&mut self, left: usize, right: usize) {
        self.neighbor_l_index = left;
        self.neighbor_r_index = right;
    }

    /// Apply forces to this bead and update its position, velocity, and acceleration, accordingly.
    fn apply_forces(&mut self, force: &Vector3<f32>) {
        // The (average?) length of each line segment ("stick"), prior to relaxation
        let starting_length = 0.5;

        // The maximum distance a bead can travel per time-step
        let d_max = starting_length * 0.025;

        // The closest any two sticks can be (note that this should be larger than `d_max`)
        let d_close = starting_length * 0.25;

        // The mass of each node ("bead"): we leave this unchanged for now
        let mass = 1.0;

        // Velocity damping factor
        let damping = 0.5;

        // Integrate acceleration and velocity (with damping)
        self.acceleration += force / mass;
        self.velocity += self.acceleration;
        self.velocity *= damping;

        // Zero out the acceleration for the next time step
        self.acceleration = Vector3::zero();

        // Set new position
        let old = self.position;

        // Each particle can travel (at most) `d_max` units each time step
        let clamped = if self.velocity.magnitude() > d_max {
            self.velocity.normalize() * d_max
        } else {
            self.velocity
        };

        self.position += clamped;

        // TODO: prevent segments from intersecting
    }
}

/// A struct representing a knot, which is a polyline embedded in 3-dimensional space
/// with a particular set of over- / under-crossings. In this program, a "knot" also
/// refers to a dynamical model, where the underlying polyline is treated as a mass-spring
/// system.
pub struct Knot {
    // The "rope" (polygonal line segment) that is knotted and will be animated
    rope: Polyline,

    // Anchor (starting) positions
    anchors: Polyline,

    // All of the "beads" (i.e. points with a position, velocity, and acceleration) that make up this knot
    beads: Vec<Bead>,

    // The GPU-side mesh used to render this knot
    mesh: Mesh,
}

impl Knot {
    pub fn new(rope: &Polyline, topology: Option<&Vec<Crossing>>) -> Knot {
        let mut beads = vec![];
        for (index, position) in rope.get_vertices().iter().enumerate() {
            let (neighbor_l_index, neighbor_r_index) = rope.get_neighboring_indices_wrapped(index);

            beads.push(Bead::new(
                position,
                index,
                neighbor_l_index,
                neighbor_r_index,
            ));
        }

        Knot {
            rope: rope.clone(),
            anchors: rope.clone(),
            beads,
            mesh: Mesh::new(&vec![], None, None, None),
        }
    }

    /// Returns an immutable reference to the polyline that formed this knot, prior
    /// to relaxation.
    pub fn get_rope(&self) -> &Polyline {
        &self.rope
    }

    /// Performs a pseudo-physical form of topological refinement, based on spring
    /// physics.
    pub fn relax(&mut self) {
        // How much each bead wants to stay near its original position (`0.0` means that
        // we ignore this force)
        let anchor_weight = 0.0;

        // Calculate forces
        let mut forces = vec![];

        for bead in self.beads.iter() {
            // Sum all of the forces acting on this particular bead
            let mut force = Vector3::zero();

            // Iterate over all potential neighbors
            for other in self.beads.iter() {
                // Don't accumulate forces on itself
                if other != bead {
                    // Grab the "other" bead, which may or may not be a neighbor to "bead"
                    if bead.are_neighbors(other) {
                        // This is a neighboring bead: calculate the (attractive) mechanical spring force that
                        // will pull this bead towards `other`
                        let mut direction = other.position - bead.position;
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
                        let mut direction = bead.position - other.position; // Reversed direction
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

            forces.push(force);
        }

        // Because of the borrow checker, we can't use an inner-loop above: instead, we
        // apply forces here
        for (bead, force) in self.beads.iter_mut().zip(forces.iter()) {
            bead.apply_forces(force);
        }

        // Update polyline positions for rendering
        self.rope.set_vertices(&self.gather_position_data());
    }

    /// Resets the physics simulation.
    pub fn reset(&mut self) {
        // First, reset the polyline
        self.rope = self.anchors.clone();

        // Reset all bead positions
        for (bead, position) in self
            .beads
            .iter_mut()
            .zip(self.anchors.get_vertices().iter())
        {
            bead.position = *position;
        }
    }

    /// Draws this knot. If `extrude` is set to `true`, then the knot will be drawn
    /// as an extruded tube (i.e. with "thickness"). Otherwise, it will be drawn as
    /// a thin line loop.
    pub fn draw(&mut self, extrude: bool) {
        if extrude {
            let vertices = self.rope.generate_tube(
                0.5,
                12,
                Some(&|pct| (pct as f32 * std::f32::consts::PI).sin() * 0.5 + 0.5),
            );

            self.mesh.set_positions(&vertices);
            self.mesh.draw(gl::TRIANGLES);
            self.mesh.draw(gl::POINTS);
        } else {
            self.mesh.set_positions(self.rope.get_vertices());
            self.mesh.draw(gl::LINE_LOOP);
            self.mesh.draw(gl::POINTS);
        }
    }

    /// Aggregates all of the beads' position vectors.
    fn gather_position_data(&self) -> Vec<Vector3<f32>> {
        self.beads.iter().map(|bead| bead.position).collect()
    }

    pub fn find_crossings(&self) {
        unimplemented!()
    }

    pub fn get_number_of_crossings(&self) {
        unimplemented!()
    }

    pub fn get_dowker_notation(&self) {
        unimplemented!()
    }

    pub fn get_conway_notation(&self) {
        unimplemented!()
    }
}
