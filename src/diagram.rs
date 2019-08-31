use crate::diagram::CromwellMove::{Commutation, Stabilization, Translation};
use crate::graphics::polyline::Polyline;
use crate::knot::Knot;
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
#[derive(Debug)]
pub enum Axis {
    Row,
    Column,
}

/// An enum representing a cardinal direction (as on a compass).
#[derive(Debug)]
pub enum Cardinality {
    NW,
    SW,
    NE,
    SE,
}

/// An enum representing the Cromwell moves, which are essentially Reidemeister
/// moves for grid diagrams. A sequence of Cromwell moves does not change the
/// knot invariant but rather, produces a new projection of the same knot.
///
/// Reference: `https://www.math.ucdavis.edu/~slwitte/research/BlackwellTapiaPoster.pdf`
pub enum CromwellMove {
    // A move that cyclically translates a row or column in one of four directions: up, down, left, or right
    Translation(Direction),

    // A move that exchanges to adjacent, non-interleaved rows or columns
    Commutation {
        axis: Axis,
        start_index: usize,
    },

    // A move that replaces an `x` with a 2x2 sub-grid
    Stabilization {
        cardinality: Cardinality,
        i: usize,
        j: usize,
    },
    // A move that replaces a 2x2 sub-grid with an `x` (the opposite of an x-stabilization): currently not supported
    //Destabilization,
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
    /// Internally, a grid diagram maintains a 2D array of `char`s, where the first axis is the rows
    /// and the second axis is the columns.
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
    ///
    /// Reference: `https://arxiv.org/pdf/1903.05893.pdf`
    pub fn apply_move(&mut self, cromwell: CromwellMove) -> Result<&mut Self, &'static str> {
        println!("Grid diagram before Cromwell move:");
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
            CromwellMove::Commutation { axis, start_index } => {
                // The last row (or column) doesn't have any adjacent row (or column) to swap with
                if start_index == self.resolution - 1 {
                    return Err("Cannot exchange row or column at `start_index` with non-existing adjacent row or column");
                }

                // Grab the two rows (or columns) that will be exchanged
                let (row_or_column_a, row_or_column_b) = match axis {
                    Axis::Row => (self.get_row(start_index + 0), self.get_row(start_index + 1)),
                    _ => (
                        self.get_column(start_index + 0),
                        self.get_column(start_index + 1),
                    ),
                };

                // Commutation is only valid if the two rows (or columns) are not interleaved
                if !self.are_interleaved(&row_or_column_a, &row_or_column_b) {
                    match axis {
                        Axis::Row => self.exchange_rows(start_index + 0, start_index + 1),
                        _ => self.exchange_columns(start_index + 0, start_index + 1),
                    }
                } else {
                    return Err(
                        "The specified rows (or columns) are interleaved and cannot be exchanged",
                    );
                }
            }
            CromwellMove::Stabilization { cardinality, i, j } => {
                if self.data[i][j] != 'x' {
                    return Err("There is no `x` at the specified grid position: stabilization cannot be performed");
                }

                // The cardinal directions below designate the corner of the new 2x2 sub-grid
                // that contains a "blank" cell (i.e. where the original `x` resided, for an
                // x-stabilization)
                match cardinality {
                    // Add column to the right of the column in question
                    Cardinality::NW | Cardinality::SW => {
                        for row in self.data.iter_mut() {
                            row.insert(j + 1, ' ');
                        }
                    }
                    // Add column to the left of the column in question
                    _ => {
                        for row in self.data.iter_mut() {
                            row.insert(j + 0, ' ');
                        }
                    }
                }
                self.resolution += 1;

                match cardinality {
                    Cardinality::NW => {
                        self.data[i][j + 0] = ' ';
                        self.data[i][j + 1] = 'x';
                        let mut extra_row = vec![' '; self.resolution];
                        extra_row[j + 0] = 'x';
                        extra_row[j + 1] = 'o';
                        self.data.insert(i + 1, extra_row);
                    }
                    Cardinality::SW => {
                        self.data[i][j + 0] = ' ';
                        self.data[i][j + 1] = 'x';
                        let mut extra_row = vec![' '; self.resolution];
                        extra_row[j + 0] = 'x';
                        extra_row[j + 1] = 'o';
                        self.data.insert(i + 0, extra_row);
                    }
                    Cardinality::NE => {
                        self.data[i][j + 0] = 'x'; // Technically, this is unnecessary
                        self.data[i][j + 1] = ' ';
                        let mut extra_row = vec![' '; self.resolution];
                        extra_row[j + 0] = 'o';
                        extra_row[j + 1] = 'x';
                        self.data.insert(i + 1, extra_row);
                    }
                    Cardinality::SE => {
                        self.data[i][j + 0] = 'x'; // Technically, this is unnecessary
                        self.data[i][j + 1] = ' ';
                        let mut extra_row = vec![' '; self.resolution];
                        extra_row[j + 0] = 'o';
                        extra_row[j + 1] = 'x';
                        self.data.insert(i + 0, extra_row);
                    }
                }
            }
        }
        println!("Grid diagram after Cromwell move:");
        println!("{:?}", self);
        Ok(self)
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

    /// Returns an immutable reference to this grid diagram's internal data store.
    pub fn get_data(&self) -> &Vec<Vec<char>> {
        &self.data
    }

    /// Sets the values of the `i`th row to `row`.
    fn set_row(&mut self, i: usize, row: &Vec<char>) {
        self.data[i] = row.clone();
    }

    /// Sets the values of the `i`th column to `col`.
    fn set_column(&mut self, i: usize, col: &Vec<char>) {
        for (entry, row) in col.iter().zip(self.data.iter_mut()) {
            row[i] = *entry;
        }
    }

    /// Returns the `i`th row of the grid diagram.
    fn get_row(&self, i: usize) -> Vec<char> {
        self.data[i].clone()
    }

    /// Returns the `i`th column of the grid diagram.
    fn get_column(&self, i: usize) -> Vec<char> {
        self.data.iter().map(|row| row[i]).collect()
    }

    /// Swaps row `a` and `b`.
    fn exchange_rows(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    /// Swaps column `a` and `b`.
    fn exchange_columns(&mut self, a: usize, b: usize) {
        // Swap each of the two corresponding column entries
        for row in self.data.iter_mut() {
            row.swap(a, b);
        }
    }

    /// Checks whether two rows (or columns) are interleaved, i.e. their projections
    /// onto the x-axis (or y-axis, respectively) overlap.
    fn are_interleaved(&self, row_or_column_a: &Vec<char>, row_or_column_b: &Vec<char>) -> bool {
        // Find where the `x` and `o` occur in each row / column: `is_alphabetic()` returns `false`
        // for spaces
        let string_a = row_or_column_a.iter().collect::<String>();
        let string_b = row_or_column_b.iter().collect::<String>();
        let matches_a: Vec<(usize, &str)> = string_a.match_indices(char::is_alphabetic).collect();
        let matches_b: Vec<(usize, &str)> = string_b.match_indices(char::is_alphabetic).collect();

        assert_eq!(matches_a.len(), 2);
        assert_eq!(matches_b.len(), 2);

        let (a_start, a_end) = (matches_a[0].0, matches_a[1].0);
        let (b_start, b_end) = (matches_b[0].0, matches_b[1].0);

        if a_start > b_start && a_end < b_end {
            // `a` is completely contained in `b`
            return false;
        } else if b_start > a_start && b_end < a_end {
            // `b` is completely contained in `a`
            return false;
        } else if a_end < b_start {
            // `a` is totally "above" `b`
            return false;
        } else if a_start > b_end {
            // `a` is totally "below" `b`
            return false;
        } else if b_end < a_start {
            // `b` is totally "above" `a`
            return false;
        } else if b_start > a_end {
            // `b` is totally "below" `a`
            return false;
        }

        // `a` and `b` must be interleaved
        true
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
        println!(
            "Total vertices in refined path: {}",
            path.get_number_of_vertices()
        );

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
