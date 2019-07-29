use cgmath::Vector3;
use crate::polyline::Polyline;

pub enum Crossing {
    Under,
    Over,
    Neither,
}

pub struct Knot {
    path: Polyline
}

impl Knot {
    pub fn new(path: &Polyline, topology: &Option<Vec<Crossing>>) -> Knot {
        let mut knot = Knot {
            path: path.clone()
        };
        
        knot
    }

    pub fn get_path(&self) -> &Polyline {
        &self.path
    }

    pub fn find_crossings(&self) {

    }

    pub fn draw_projection(&self) {
        unimplemented!()
    }

    pub fn relax(&self) {
        unimplemented!()
    }
}