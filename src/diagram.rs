use crate::diagram::CromwellMove::{Commutation, Destabilization, Stabilization, Translation};
use crate::knot::Knot;
use crate::polyline::Polyline;
use cgmath::Vector3;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::ffi::OsStr;
use std::io;
use std::path::Path;

/// An enum representing a direction (see `CromwellMove::Translation`).
#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// An enum representing an axial direction (either rows or columns).
pub enum Axis {
    Row,
    Column,
}

/// An enum representing the Cromwell moves, which are essentially Reidemeister
/// moves for grid diagrams. A sequence of Cromwell moves does not change the
/// knot invariant but rather, produces a new projection of the same knot.
///
/// Reference: `https://www.math.ucdavis.edu/~slwitte/research/BlackwellTapiaPoster.pdf`
pub enum CromwellMove {
    Translation(Direction),
    Commutation { axis: Axis, i: usize, j: usize },
    Stabilization,
    Destabilization,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            _ => Direction::Right,
        }
    }
}

impl Distribution<Axis> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Axis {
        match rng.gen_range(0, 2) {
            0 => Axis::Row,
            _ => Axis::Column,
        }
    }
}

impl Distribution<CromwellMove> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CromwellMove {
        match rng.gen_range(0, 4) {
            _ => Translation(rand::random()),
            //            1 => Commutation { axis: rand::random(), i: 0, j: 0 },
            //            2 => Stabilization,
            //            _ => Destabilization,
        }
    }
}

trait KnotGenerator {
    fn generate(&self) -> Knot;
}

/// A struct representing a grid diagram corresponding to a particular knot invariant (or
/// the unknot).
pub struct Diagram {
    // The number of rows and columns in the grid diagram (we assume all diagrams are square)
    resolution: usize,

    // The grid data (i.e. a 2D array of x's, o's, and blank cells)
    data: Vec<Vec<char>>,
}

impl Diagram {
    /// Generates a grid diagram from a .csv file, where each entry is either ` `, `x`, or `o`.
    pub fn from_path(path: &Path) -> Result<Diagram, &'static str> {
        if let Some(".csv") = path.extension().and_then(OsStr::to_str) {
            return Err("Only .csv grid files are supported at the moment");
        }

        let mut resolution = 0;
        let mut data: Vec<Vec<char>> = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .unwrap();
        let mut number_of_rows = 0;

        for result in reader.records() {
            let record = result.unwrap();
            resolution = record.len();
            number_of_rows += 1;

            // Push this row of data
            data.push(record.as_slice().chars().collect());
        }

        // Verify that the grid is square
        if resolution != number_of_rows {
            return Err("Provided grid file is not square: the number of rows should equal the number of columns");
        }

        println!("Building a {}x{} grid diagram", resolution, resolution);
        let diagram = Diagram { resolution, data };

        return match diagram.validate() {
            Ok(_) => Ok(diagram),
            Err(e) => Err(e),
        };
    }

    /// Applies a particular Cromwell move to the grid diagram.
    pub fn apply_move(&mut self, cromwell: CromwellMove) -> &mut Self {
        println!("{:?}", self);
        match cromwell {
            CromwellMove::Translation(direction) => match direction {
                Direction::Up => {
                    let first_row = self.data.remove(0);
                    self.data.push(first_row);
                }
                Direction::Down => {
                    if let Some(last_row) = self.data.pop() {
                        self.data.insert(0, last_row);
                    }
                }
                Direction::Left => {
                    for row in self.data.iter_mut() {
                        let entry = row.remove(0);
                        row.push(entry);
                    }
                }
                Direction::Right => {
                    for row in self.data.iter_mut() {
                        if let Some(entry) = row.pop() {
                            row.insert(0, entry);
                        }
                    }
                }
            },
            _ => (),
        }

        println!("{:?}", self);
        self
    }

    /// Applies a random Cromwell move to the grid diagram.
    pub fn apply_move_random(&mut self) -> &mut Self {
        self.apply_move(rand::random());
        self
    }
    /// Generates a random, valid grid diagram that may or may not be the unknot.
    pub fn random() {
        unimplemented!()
    }

    /// Validates the grid diagram, ensuring that there is only one `x` and one `o`
    /// per column and row.
    fn validate(&self) -> Result<(), &'static str> {
        for index in 0..self.resolution {
            let current_row = self.get_row(index);
            let current_col = self.get_column(index);

            if current_row.iter().collect::<String>().matches('x').count() != 1
                || current_row.iter().collect::<String>().matches('o').count() != 1
                || current_col.iter().collect::<String>().matches('x').count() != 1
                || current_col.iter().collect::<String>().matches('o').count() != 1
            {
                return Err("Invalid grid diagram: ensure that each column / row contains exactly one `x` and one `o`");
            }
        }
        Ok(())
    }

    /// Returns the resolution of this grid diagram (i.e. the number of rows or number of columns).
    pub fn get_resolution(&self) -> usize {
        self.resolution
    }

    /// Returns a reference to this grid diagram's internal data store.
    pub fn get_data(&self) -> &Vec<Vec<char>> {
        &self.data
    }

    /// Returns the `i`th row of the grid diagram.
    pub fn get_row(&self, i: usize) -> Vec<char> {
        self.data[i].clone()
    }

    /// Returns the `i`th column of the grid diagram.
    pub fn get_column(&self, i: usize) -> Vec<char> {
        self.data.iter().map(|row| row[i]).collect()
    }

    /// Converts a pair of grid indices `<i, j>`, each of which lies in the range
    /// `[0..self.resolution]`, to an "absolute" index, ranging from `[0..self.resolution^2]`.
    fn convert_to_absolute_index(&self, i: usize, j: usize) -> usize {
        i + j * self.resolution
    }

    /// Converts an "absolute index" in the range `[0..self.resolution^2]` to a
    /// pair of grid indices `<i, j>`, each of which lies in the range `[0..self.resolution]`.
    fn convert_to_grid_indices(&self, absolute_index: usize) -> (usize, usize) {
        (
            absolute_index % self.resolution,
            absolute_index / self.resolution,
        )
    }

    /// Generates a knot corresponding to this grid diagram.
    pub fn generate_knot(&self) -> Knot {
        // We begin traversing the knot at the first column:
        // `s` = "Start", (relative) index of the `x` in the first column (there will always be one)
        // `e` = "End", (relative) index of the `o` in the first column (there will always be one)
        let mut s = self
            .get_column(0)
            .iter()
            .collect::<String>()
            .find('x')
            .unwrap();
        let mut e = self
            .get_column(0)
            .iter()
            .collect::<String>()
            .find('o')
            .unwrap();
        let tie = s;

        let mut knot_topology = vec![
            self.convert_to_absolute_index(s, 0),
            self.convert_to_absolute_index(e, 0),
        ];

        let mut keep_going = true;
        let mut traverse_horizontal = true;
        while keep_going {
            // First, get the row or column corresponding to the index where the last
            // row or column ended
            //
            // Note that:
            // Cols are connected: x -> o
            // Rows are connected: o -> x
            let (next_index, slice) = if traverse_horizontal {
                // We just found an `o` (in the last column), so find the `x` in this row
                let slice = self.get_row(e);
                (slice.iter().collect::<String>().find('x').unwrap(), slice)
            } else {
                // We just found an `x` (in the last row), so find the `o` in this column
                let slice = self.get_column(e);
                (slice.iter().collect::<String>().find('o').unwrap(), slice)
            };

            // Convert the above index to absolute indices that range from `[0..(self.resolution * self.resolution)]`,
            // taking care to modify the function parameters based on the current orientation (horizontal / vertical)
            let absolute_index = if traverse_horizontal {
                self.convert_to_absolute_index(e, next_index)
            } else {
                self.convert_to_absolute_index(next_index, e)
            };

            // Push back the new endpoint and check to see whether we have finished traversing the entire
            // knot
            if !knot_topology.contains(&absolute_index) {
                knot_topology.push(absolute_index);
            } else {
                // We are at the end
                knot_topology.push(tie);
                keep_going = false;
            }

            s = e;
            e = next_index;

            // Switch directions
            traverse_horizontal = !traverse_horizontal;
        }

        // If we want to traverse just rows or just columns, we can simply use the underlying knot
        // topology and ignore either the first or last element
        let mut rows = knot_topology.clone();
        let mut cols = knot_topology.clone();
        rows.remove(0);
        cols.pop();
//        println!(
//            "Knot topology (before inserting any crossings): {:?}",
//            knot_topology
//        );

        // This should always be true, i.e. for a 6x6 grid there should be 6 pairs of x's and o's (12
        // indices total)...note that we perform this check before checking for any crossings, which
        // will necessarily add more indices to the knot topology
        assert_eq!(knot_topology.len(), self.resolution * 2 + 1);

        // Find crossings: rows pass under any columns that they intersect, so we will
        // add additional vertex (or vertices) to any column that contains a intersection(s)
        // and "lift" this vertex (or vertices) along the z-axis
        let mut lifted = vec![];

        for col_chunk in cols.chunks(2) {
            let (mut col_s, mut col_e) = (col_chunk[0], col_chunk[1]);

            let mut oriented_upwards = false;

            // If this condition is `true`, then the column is oriented from bottom to
            // top (i.e. "upwards") - we do this so that it is "easier" to tell whether
            // or not a row intersects a column (see below)
            if col_s > col_e {
                std::mem::swap(&mut col_s, &mut col_e);
                oriented_upwards = true;
            }

            let (cs_i, cs_j) = self.convert_to_grid_indices(col_s);
            let (ce_i, ce_j) = self.convert_to_grid_indices(col_e);

            // A list of all intersections along this column
            let mut intersections = vec![];

            for row_chunk in rows.chunks(2) {
                let (mut row_s, mut row_e) = (row_chunk[0], row_chunk[1]);

                if row_s > row_e {
                    std::mem::swap(&mut row_s, &mut row_e);
                }

                let (rs_i, rs_j) = self.convert_to_grid_indices(row_s);
                let (re_i, re_j) = self.convert_to_grid_indices(row_e);

                if cs_j > rs_j && cs_j < re_j && cs_i < rs_i && ce_i > rs_i {
                    let intersect = self.convert_to_absolute_index(rs_i, cs_j);
                    intersections.push((rs_i, intersect));
                    lifted.push(intersect);
                }
            }

            // Sort on the row `i` index (i.e. sort vertically, from top to bottom of the table grid)
            intersections.sort_by_key(|k| k.0);

            // If the start / end indices of this column were flipped before, we have to reverse the
            // order in which we insert the crossings here as well
            if !oriented_upwards {
                intersections.reverse();
            }

//            println!(
//                "Intersections found for column #{}: {:?}",
//                self.convert_to_grid_indices(col_s).1,
//                intersections
//            );

            for (index, node) in knot_topology.iter().enumerate() {
                // If we have arrived at either the start or end of the column, begin insertion
                if *node == col_s || *node == col_e {
                    for (_, ix) in intersections.iter() {
                        knot_topology.insert(index + 1, *ix);
                    }
                    break;
                }
            }
            //println!("   New topology: {:?}", knot_topology);
        }

        // Ex: old topology vs. new topology (after crossings are inserted)
        //
        // `[1, 4, 28, __, 26, 8, _, 6, 18, __, 21, 33, 35, 17, __, __, 13, 1]`
        // `[1, 4, 28, 27, 26, 8, 7, 6, 18, 20, 21, 33, 35, 17, 16, 14, 13, 1]`

        // Convert indices to actual 3D positions so that we can
        // (eventually) draw a polyline corresponding to this knot: the
        // world-space width and height of the 3D grid are automatically
        // set to the resolution of the diagram so that each grid "cell"
        // is unit width / height
        let mut path = Polyline::new();
        let w = self.resolution as f32;
        let h = self.resolution as f32;

        // This value is somewhat arbitrary but should *probably* match
        // the tube radius used later on in the rendering loop...
        let lift_amount = 0.1;

        for absolute_index in knot_topology.iter() {
            // Remember:
            // `i` is the row, ranging from `[0..self.resolution]`
            // `j` is the col, ranging from `[0..self.resolution]`
            let (i, j) = self.convert_to_grid_indices(*absolute_index);

            // World-space position of the vertex corresponding to this grid index:
            // make sure that the center of the grid lies at the origin
            let x = (j as f32 / self.resolution as f32) * w - 0.5 * w;
            let y = h - (i as f32 / self.resolution as f32) * h - 0.5 * h;
            let z = if lifted.contains(absolute_index) {
                lift_amount
            } else {
                0.0
            };

            path.push_vertex(&Vector3::new(x, y, z));
        }

        // Subdivide the path
        path = path.refine(0.5);
        println!("Total vertices in refined path: {}", path.get_number_of_vertices());

        Knot::new(&path, None)
    }
}

impl std::fmt::Debug for Diagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.iter() {
            write!(f, "{:?}\n", row);
        }
        Ok(())
    }
}
