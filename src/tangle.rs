pub enum Tangle {
    // Two vertical strands
    Infinity,

    // Two horizontal strands
    Zero,

    // Two twisted strands (left-handed or right-handed)
    N(isize),
}

pub enum PointOfCrossing {
    NW,
    NE,
    SW,
    SE,
}

impl Tangle {
    /// Returns `true` if this is a rational tangle and `false` otherwise.
    pub fn is_rational(&self) -> bool {
        unimplemented!()
    }

    pub fn product(&self, other: &Tangle) -> Tangle {
        unimplemented!()
    }

    pub fn sum(&self, other: &Tangle) -> Tangle {
        unimplemented!()
    }

    pub fn equivalent(&self, other: &Tangle) -> bool {
        unimplemented!()
    }

    /// Reflects this tangle across the NW-SE diagonal.
    pub fn reflect(&self) -> Tangle {
        unimplemented!()
    }
}
