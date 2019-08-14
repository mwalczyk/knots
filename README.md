# knots
➰ A program for manipulating and playing with knot diagrams.

<p align="center">
  <img src="https://github.com/mwalczyk/knots/blob/master/screenshots/knots.png" alt="screenshot" width="400" height="auto"/>
</p>

## Description
### Grid Diagrams
One interesting way of representing a knot is via a so-called `n`x`n` _grid diagram_. Each cell of the grid is either an `x`, an `o`, or "blank." Each row and column is restricted to have _exactly_ one `x` and one `o`. We can construct the knot corresponding to a particular grid diagram using the following procedure:

1. In each column, connect each `x` to the corresponding `o` 
2. In each row, connect each `o` to the corresponding `x`
3. Whenever a horizontal segment intersects a vertical segment, assume that the vertical segment passes _over_ the horizontal segment (i.e. a grid diagram _only_ consists of over-crossings)



### Cromwell Moves
The Cromwell Moves are similar to the 3 [Reidemeister Moves](https://en.wikipedia.org/wiki/Reidemeister_move), specifically applied to grid diagrams. They all us to obtain isotopic knots, i.e. knots that have the same underlying topology but "look" different. This gives us a way to systematically explore a given knot invariant.

```rust
enum CromwellMove {
    // A move that cyclically translates a row or column in one of four directions: up, down, left, or right
    Translation(Direction),

    // A move that exchanges to adjacent, non-interleaved rows or columns
    Commutation { axis: Axis, start_index: usize, },

    // A move that replaces an `x` with a 2x2 sub-grid
    Stabilization { cardinality: Cardinality, i: usize, j: usize, },
    
    // A move that replaces a 2x2 sub-grid with an `x` (the opposite of an x-stabilization): currently not supported
    Destabilization { cardinality: Cardinality, i: usize, j: usize, },
}
```

## Tested On
- Windows 8.1
- NVIDIA GeForce GT 750M
- Rust compiler version `1.38.0-nightly` (nightly may not be required)

NOTE: this project will only run on graphics cards that support OpenGL [Direct State Access](https://www.khronos.org/opengl/wiki/Direct_State_Access) (DSA).

## To Build
1. Clone this repo.
2. Make sure 🦀 [Rust](https://www.rust-lang.org/en-US/) installed and `cargo` is in your `PATH`.
3. Inside the repo, run: `cargo build --release`.

## To Use
All grid diagrams must be "square" `.csv` files (the same number of rows as columns). Each row and column must have _exactly_ one `x` and one `o`: all other entries should be spaces ("blank"). The grid diagram will be validated upon construction, but the program will `panic!` if one of the conditions above is not met.

To rotate the camera around the object in 3-dimensions, press + drag the left mouse button. Press `h` to "home" (i.e. reset) the camera.

You can change between wireframe and filled modes by pressing `w` and `f`. You can save out a screenshot by pressing `s`.

## To Do
- [ ] Implement a knot "drawing" tool
- [ ] Add segment-segment intersection test for more robust topological refinement
- [ ] Add a GUI for viewing the current grid diagram
- [ ] Generate the Dowker notation for a given knot diagram
- [ ] Explore planar graphs and their relationship to knot diagrams

## Credits
This project was largely inspired by and based on previous work done by Dr. Robert Scharein, whose PhD [thesis](https://knotplot.com/thesis/) and [software](https://knotplot.com/) were vital towards my understanding of the relaxation / meshing procedures.

### License
[Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)
