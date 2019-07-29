use cgmath::Vector3;
use std::ffi::OsStr;
use std::io;
use crate::knot::Knot;
use std::path::Path;
use crate::polyline::Polyline;

/// Reference: `https://www.math.ucdavis.edu/~slwitte/research/BlackwellTapiaPoster.pdf`
enum CromwellMove {
    Translation,
    Commutation,
    Stabilization,
    Destabilization,
}

/// A struct representing a grid diagram corresponding to a particular knot invariant (or
/// the unknot).
pub struct Diagram {
    // The width and height of the grid (for now, we assume all grid diagrams are square)
    resolution: usize,

    // The grid data (i.e. a 2D array of x's, o's, and blank cells)
    data: Vec<Vec<char>>
}

impl Diagram {
    /// Generates a grid diagram from a .csv file, where each entry is either ` `, `x`, or `o`.
    pub fn from_path(path: &Path) -> Diagram {
        if let Some(".csv") = path.extension().and_then(OsStr::to_str) {
            panic!("Only .csv grid files are supported at the moment");
        }

        let mut resolution = 0;
        let mut data: Vec<Vec<char>> = vec![];
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_path(path).unwrap();

        for result in reader.records() {
            let record = result.unwrap();
            resolution = record.len();
            data.push(record.as_slice().chars().collect());
        }

        // TODO: verify that the grid is square

        println!("Returning {}x{} grid:", resolution, resolution);

        let diagram = Diagram {
            resolution,
            data
        };
        // TODO: diagram.validate();
        diagram
    }

    /// Generates a random, valid grid diagram that may or may not be the unknot.
    pub fn random() {
        unimplemented!()
    }

    /// Validates the grid diagram, ensuring that there is only one `x` and one `o`
    /// per column and row.
    fn validate(&self) {
        unimplemented!()
    }

    pub fn get_resolution(&self) -> usize {
        self.resolution
    }

    pub fn get_data(&self) -> &Vec<Vec<char>> {
        &self.data
    }

    /// Returns the `i`th row of the grid diagram.
    pub fn get_row(&self, i: usize) -> Vec<char> {
        self.data[i].clone()
    }

    /// Returns the `i`th column of the grid diagram.
    pub fn get_column(&self, i: usize) -> Vec<char> {
        let mut column = vec![];
        for row in self.data.iter() {
            column.push(row[i]);
        }
        column
    }

    fn convert_to_absolute_index(&self, i: usize, j: usize) -> usize {
        i + j * self.resolution
    }

    fn convert_to_grid_indices(&self, absolute_index: usize) -> (usize, usize) {
        (absolute_index % self.resolution, absolute_index / self.resolution)
    }

    /// Generates the knot corresponding to this grid diagram.
    pub fn generate_knot(&self) -> Knot {
        // We begin traversing the knot at the first column...
        // "Start", (relative) index of the `x` in the first column (there will always be one)
        // "End", (relative) index of the `o` in the first column (there will always be one)
        let mut s = self.get_column(0).iter().collect::<String>().find('x').unwrap();
        let mut e = self.get_column(0).iter().collect::<String>().find('o').unwrap();
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
            let slice = if traverse_horizontal {
                self.get_row(e)
            } else {
                self.get_column(e)
            };

            // We just found an `o` (in the last column), so find the `x` in this row
            let next_index = if traverse_horizontal {
                slice.iter().collect::<String>().find('x').unwrap()
            } else {
                slice.iter().collect::<String>().find('o').unwrap()
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
        println!("Knot topology: {:?}", knot_topology);

        // This should always be true, i.e. for a 6x6 grid there should be 6 pairs of x's and o's (12
        // indices total)
        assert_eq!(knot_topology.len(), self.resolution * 2 + 1);

        // Convert indices to actual 3D positions so that we can
        // (eventually) draw a polyline corresponding to this knot
        let mut path = Polyline::new();
        let w = 1.0;
        let h = 1.0;
        for absolute_index in knot_topology {
            let (i, j) = self.convert_to_grid_indices(absolute_index);

            // `i` is the row
            // `j` is the col
            let x = (j as f32 / self.resolution as f32) * w - 0.5 * w;
            let y = (i as f32 / self.resolution as f32) * h - 0.5 * h;
            let z = 0.0;
            path.push_vertex(&Vector3::new(x, y, z));
        }

        Knot::new(&path)
    }
}