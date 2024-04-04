mod backtracking;

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Copy, PartialEq)]
enum Shape {
    Free,
    M1,
    M2,
    M3,
    Ship,

    // actual ones
    /// Xo
    /// oo
    OneTL,
    /// oX
    /// oo
    OneTR,
    /// oo
    /// Xo
    OneBL,
    /// oo
    /// oX
    OneBR,
    /// Xo
    /// oX
    TwoDiagDown,
    /// oX
    /// Xo
    TwoDiagUp,
    /// XX
    /// oo
    TwoHorT,
    /// Xo
    /// Xo
    TwoHorL,
    /// oo
    /// XX
    TwoHorB,
    /// oX
    /// oX
    TwoHorR,
    /// yy (above actual tile)
    /// XX
    /// oo
    LargeEdgeT,
    /// v   (left of actual tile)
    /// yXo
    /// yXo
    LargeEdgeL,
    /// oo
    /// XX
    /// yy (below actual tile)
    LargeEdgeB,
    /// oXy
    /// oXy
    LargeEdgeR,
    /// yy
    /// yXo
    ///  oo
    LargeCornerTL,
    ///  yy
    /// oXy
    /// oo
    LargeCornerTR,
    ///  oo
    /// yXo
    /// yy
    LargeCornerBL,
    /// oo
    /// oXy
    /// oyy
    LargeCornerBR,
}

impl Shape {
    fn rotate(&self, clockwise: bool) -> Shape {
        match self {
            Shape::Free => Shape::Free,
            Shape::Ship => Shape::Ship, // cannot rotate
            Shape::OneTL => {
                if clockwise {
                    Shape::OneTR
                } else {
                    Shape::OneBL
                }
            }
            Shape::OneTR => {
                if clockwise {
                    Shape::OneBR
                } else {
                    Shape::OneTL
                }
            }
            Shape::OneBL => {
                if clockwise {
                    Shape::OneTL
                } else {
                    Shape::OneBR
                }
            }
            Shape::OneBR => {
                if clockwise {
                    Shape::OneBL
                } else {
                    Shape::OneTR
                }
            }
            Shape::TwoDiagDown => Shape::TwoDiagDown,
            Shape::TwoDiagUp => Shape::TwoDiagUp,
            Shape::TwoHorT => Shape::TwoHorT,
            Shape::TwoHorL => Shape::TwoHorL,
            Shape::TwoHorB => Shape::TwoHorB,
            Shape::TwoHorR => Shape::TwoHorR,
            Shape::LargeEdgeT => Shape::LargeEdgeT,
            Shape::LargeEdgeL => Shape::LargeEdgeL,
            Shape::LargeEdgeB => Shape::LargeEdgeB,
            Shape::LargeEdgeR => Shape::LargeEdgeR,
            Shape::LargeCornerTL => Shape::LargeCornerTL,
            Shape::LargeCornerTR => Shape::LargeCornerTR,
            Shape::LargeCornerBL => Shape::LargeCornerBL,
            Shape::LargeCornerBR => Shape::LargeCornerBR,
            _ => Shape::Free, // TODO: remove
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[cfg(test)]
mod test4 {
    use super::*;
    use std::hash::{DefaultHasher, Hash, Hasher};

    fn get_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_points_equal() {
        let point1 = Point { x: 1, y: 2 };
        let point2 = Point { x: 1, y: 2 };
        let point3 = Point { x: 1, y: 3 };
        assert_eq!(point1, point2);
        assert_eq!(
            get_hash(&point1),
            get_hash(&point2),
            "hashes of equal objects should be equal."
        );
        assert_eq!(point1 == point2, true);
        assert_ne!(point1, point3);
        assert_ne!(get_hash(&point1), get_hash(&point3));
        assert_eq!(point1 == point3, false);
    }
}

// Central square is an actual 2x2 square.
// Positions where an occupancy can be are indicated by x,y coordinates.
//
// ┌───────────────────────────────────────────────►
// │                                                x
// │
// │          │               │
// │    -1,-1 │ 0,-1          │
// │          │               │
// │          │               │
// │   ───────┼───────────────┼────────────
// │          │               │
// │     -1,0 │  0,0      1,0 │  2,0
// │          │               │
// │          │               │
// │          │               │
// │          │  0,1      1,1 │
// │          │               │
// │   ───────┼───────────────┼────────────
// │          │               │
// │          │               │
// │          │               │
// │          │               │
// │
// │
// ▼ y

impl Shape {
    fn get_points(&self) -> Vec<Point> {
        match self {
            Shape::Free => vec![],
            Shape::M1 => vec![Point { x: 1, y: 0 }],
            Shape::M2 => vec![Point { x: 0, y: 0 }, Point { x: 1, y: 1 }],
            Shape::M3 => vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 1 },
                Point { x: 1, y: 0 },
            ],
            Shape::Ship => vec![
                Point { x: 0, y: 0 },
                Point { x: 1, y: 0 },
                Point { x: 0, y: 1 },
                Point { x: 1, y: 1 },
                Point { x: 2, y: 0 },
                Point { x: -1, y: 0 },
            ],
        }
    }
}

struct Board {
    shapes: [Shape; 9],
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq)]
struct BoardIndex2d {
    x: i32,
    y: i32,
}

impl PartialEq for BoardIndex2d {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl BoardIndex2d {
    /// index to index the array of shapes
    fn to_index(&self) -> usize {
        (self.y * 3 + self.x) as usize
    }
    fn from_index(index: usize) -> BoardIndex2d {
        BoardIndex2d {
            x: (index % 3) as i32,
            y: (index / 3) as i32,
        }
    }

    /// Returns the neighbor in the given direction, if it exists.
    fn neighbor(&self, direction: &Direction) -> Option<BoardIndex2d> {
        let naive_index = match direction {
            Direction::Up => BoardIndex2d {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => BoardIndex2d {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => BoardIndex2d {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => BoardIndex2d {
                x: self.x + 1,
                y: self.y,
            },
        };
        if naive_index.x < 0 || naive_index.x >= 3 || naive_index.y < 0 || naive_index.y >= 3 {
            None
        } else {
            Some(naive_index)
        }
    }
}

fn has_duplicates<T: std::hash::Hash + std::cmp::Eq>(vec: &[T]) -> bool {
    let mut seen = HashSet::new();
    for value in vec {
        if !seen.insert(value) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_duplicates() {
        // Test case with duplicates
        let vec_with_duplicates = vec![1, 2, 3, 2, 4];
        assert_eq!(has_duplicates(&vec_with_duplicates), true);

        // Test case without duplicates
        let vec_without_duplicates = vec![1, 2, 3, 4, 5];
        assert_eq!(has_duplicates(&vec_without_duplicates), false);

        // Test case with empty vector
        let empty_vec: Vec<i32> = vec![];
        assert_eq!(has_duplicates(&empty_vec), false);

        // Test case with a struct
        let mut points = vec![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
        assert_eq!(has_duplicates(&points), false);
        points.push(points[0].clone());
        assert_eq!(has_duplicates(&points), true);
    }
}

/// Describe how the tile at `board_index` moves on the grid in direction `grid_dx, grid_dy`.
struct MovingTile {
    board_index: BoardIndex2d,
    grid_dx: i32,
    grid_dy: i32,
}

impl MovingTile {
    fn no_move() -> MovingTile {
        MovingTile {
            board_index: BoardIndex2d { x: 0, y: 0 },
            grid_dx: 0,
            grid_dy: 0,
        }
    }
}

/// convert to the grid coordinates that are used for drawing/collision detection
fn grid_coordinates_of(board_index: &BoardIndex2d) -> Point {
    Point {
        x: 1 + 2 * board_index.x,
        y: 1 + 2 * board_index.y,
    }
}

fn map_shape_points_to_grid_points(
    shape: &Shape,
    moving_tile: &MovingTile,
    board_index: &BoardIndex2d,
) -> Vec<Point> {
    let dx: i32;
    let dy: i32;
    if *board_index == moving_tile.board_index {
        dx = moving_tile.grid_dx;
        dy = moving_tile.grid_dy;
    } else {
        dx = 0;
        dy = 0;
    }

    let grid_coordinates = grid_coordinates_of(board_index);

    shape
        .get_points()
        .iter()
        .map(|point| Point {
            x: point.x + grid_coordinates.x + dx,
            y: point.y + grid_coordinates.y + dy,
        })
        .collect()
}

#[cfg(test)]
mod tests2 {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_map_shape_points_to_grid_points() {
        let mut board = Board::empty_board();
        let top_left = BoardIndex2d { x: 0, y: 0 };
        let shape = Shape::M1;
        board.shapes[top_left.to_index()] = shape.clone();
        let local_points = shape.get_points();

        let moving_top_left_tile = MovingTile {
            board_index: top_left.clone(),
            grid_dx: 0,
            grid_dy: 0,
        };

        assert_eq!(
            map_shape_points_to_grid_points(&shape, &moving_top_left_tile, &top_left),
            local_points
                .iter()
                .map(|point| Point {
                    x: point.x + 1,
                    y: point.y + 1
                })
                .collect_vec()
        );

        let center = BoardIndex2d { x: 1, y: 1 };

        assert_eq!(
            map_shape_points_to_grid_points(&shape, &moving_top_left_tile, &center),
            local_points
                .iter()
                .map(|point| Point {
                    x: point.x + 3,
                    y: point.y + 3
                })
                .collect_vec()
        );

        // now apply some dx

        let dx = 2;

        let moving_center_tile = MovingTile {
            board_index: center.clone(),
            grid_dx: dx,
            grid_dy: 0,
        };

        // global points move accordingly

        assert_eq!(
            map_shape_points_to_grid_points(&shape, &moving_center_tile, &center),
            local_points
                .iter()
                .map(|point| Point {
                    x: point.x + 3 + dx,
                    y: point.y + 3
                })
                .collect_vec()
        );

        // unless the moving tile is a differnt tile, then they stay the same

        assert_eq!(
            map_shape_points_to_grid_points(&shape, &moving_top_left_tile, &center),
            local_points
                .iter()
                .map(|point| Point {
                    x: point.x + 3,
                    y: point.y + 3
                })
                .collect_vec()
        );
    }
}

impl Board {
    fn empty_board() -> Board {
        Board {
            shapes: [Shape::Free; 9],
        }
    }

    fn is_valid(&self) -> bool {
        // has exactly one free space
        let mut free_count = 0;

        for shape in &self.shapes {
            if let Shape::Free = shape {
                free_count += 1;
            }
        }

        // and has no collissions
        free_count == 1 && self.is_collission_free(&MovingTile::no_move())
    }

    fn is_won(&self) -> bool {
        let in_front_of_exit = BoardIndex2d { x: 1, y: 2 };
        if self.shapes[in_front_of_exit.to_index()] != Shape::Ship {
            return false;
        } else {
            let leave_board = MovingTile {
                board_index: in_front_of_exit,
                grid_dx: 0,
                grid_dy: 1,
            };
            self.is_collission_free(&leave_board)
        }
    }

    fn find_free_space(&self) -> BoardIndex2d {
        for (i, shape) in self.shapes.iter().enumerate() {
            if let Shape::Free = shape {
                return BoardIndex2d::from_index(i);
            }
        }

        panic!("No free space found");
    }

    /// Move the free space in the given direction iff it is possible (i.e. a valid move).
    fn move_free_space(&self, space_moves_in_direction: &Direction) -> Option<Board> {
        let free_space_position = self.find_free_space();
        if let Some(neighbor_position) = free_space_position.neighbor(space_moves_in_direction) {
            let neighbor_moves_in_direction = space_moves_in_direction.opposite();

            // check collission for move and final position
            let collission_free = match neighbor_moves_in_direction {
                Direction::Up => [(0, -1), (0, -2)],
                Direction::Down => [(0, 1), (0, 2)],
                Direction::Left => [(-1, 0), (-2, 0)],
                Direction::Right => [(1, 0), (2, 0)],
            }
            .map(|(dx, dy)| MovingTile {
                board_index: neighbor_position,
                grid_dx: dx,
                grid_dy: dy,
            })
            .iter()
            .all(|moving_tile| self.is_collission_free(&moving_tile));
            if !collission_free {
                return None;
            }

            // collission free, hence construct the new situation
            let mut new_shapes = self.shapes.clone();
            let free_space_index = free_space_position.to_index();
            let neighbor_index = neighbor_position.to_index();
            new_shapes.swap(free_space_index as usize, neighbor_index as usize);
            Some(Board { shapes: new_shapes })
        } else {
            None
        }
    }

    /// Check if the constellation on the board is collission free given the
    /// move indicated by `moving_tile`. The `moving_tile` allows to specify a
    /// direction in grid coordinates, hence it is possible to check a
    /// collission for an intermediate state, i.e. _during_ movement.
    /// Note: The moving tile pertains the actual tile, not the free space.
    fn is_collission_free(&self, moving_tile: &MovingTile) -> bool {
        let mut occupied_points: Vec<Point> = vec![];
        for y in 0..3 {
            for x in 0..3 {
                let board_index = BoardIndex2d { x, y };
                let shape = &self.shapes[board_index.to_index()];
                map_shape_points_to_grid_points(shape, &moving_tile, &board_index)
                    .into_iter()
                    .for_each(|point| occupied_points.push(point));
            }
        }
        !has_duplicates(&occupied_points[..])
    }
}

#[cfg(test)]
mod test3 {
    use super::*;

    #[test]
    fn test_is_collission_free_1() {
        let mut board = super::Board::empty_board();
        board.shapes[0] = Shape::Ship;
        board.shapes[1] = Shape::Ship;

        assert!(!board.is_collission_free(&MovingTile::no_move()));
    }

    #[test]
    fn test_is_collission_free_2() {
        let mut board = super::Board::empty_board();
        board.shapes[6] = Shape::Ship;
        board.shapes[7] = Shape::M3;
        println!("{board}");

        assert!(!board.is_collission_free(&MovingTile::no_move()));
    }

    #[test]
    fn test_is_collission_free_3() {
        let mut board = super::Board::empty_board();
        board.shapes[6] = Shape::Ship;
        board.shapes[1] = Shape::Ship;
        println!("{board}");

        assert!(
            board.is_collission_free(&MovingTile {
                board_index: BoardIndex2d::from_index(1),
                grid_dx: 0,
                grid_dy: 0
            }),
            "before move",
        );
        assert!(
            board.is_collission_free(&MovingTile {
                board_index: BoardIndex2d::from_index(1),
                grid_dx: 0,
                grid_dy: 1
            }),
            "move 1"
        );
        assert!(
            board.is_collission_free(&MovingTile {
                board_index: BoardIndex2d::from_index(1),
                grid_dx: 0,
                grid_dy: 2
            }),
            "move 2"
        );
        assert!(
            !board.is_collission_free(&MovingTile {
                board_index: BoardIndex2d::from_index(1),
                grid_dx: 0,
                grid_dy: 3
            }),
            "move 3 -- now we have a collission"
        );
    }
}

// drawing --------------------------------------------------------------------

fn drawing_character_for(shape: &Shape) -> &str {
    match shape {
        Shape::M1 => "x",
        Shape::M2 => "X",
        Shape::M3 => "Y",
        Shape::Free => "o",
        Shape::Ship => "V",
    }
}

fn drawing_points_for(shape: &Shape) -> Vec<Point> {
    match shape {
        Shape::Free => vec![
            Point { x: 0, y: 0 },
            Point { x: 1, y: 0 },
            Point { x: 0, y: 1 },
            Point { x: 1, y: 1 },
        ],
        _ => shape.get_points(),
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut screen_buffer = vec![" "; 8 * 8];
        for y in 0..3 {
            for x in 0..3 {
                let board_index = BoardIndex2d { x, y };
                let shape = &self.shapes[board_index.to_index()];
                drawing_points_for(shape)
                    .iter()
                    .map(|point| Point {
                        x: point.x + 1 + 2 * x,
                        y: point.y + 1 + 2 * y,
                    })
                    .map(|point| point.x + 8 * point.y)
                    .for_each(|index| screen_buffer[index as usize] = drawing_character_for(shape));
            }
        }

        write!(f, "+--------+\n")?;
        for y in 0..8 {
            write!(f, "|")?;
            for x in 0..8 {
                write!(f, "{}", screen_buffer[x + 8 * y])?;
            }
            write!(f, "|\n")?;
        }
        write!(f, "+--------+\n")?;

        Ok(())
    }
}

fn main() -> crossterm::Result<()> {
    let board = Board {
        shapes: [
            Shape::Free,
            Shape::M1,
            Shape::M2,
            Shape::M3,
            Shape::M1,
            Shape::M1,
            Shape::Ship,
            Shape::M1,
            Shape::M1,
        ],
    };

    println!("{}", board);

    println!("Is valid: {}", board.is_valid());

    let mut history = vec![board];

    loop {
        println!(
            "Move {}; use arrow keys to move the 'free' space, or 'q' to quit.",
            history.len() - 1
        );
        enable_raw_mode()?; // raw mode to get individual key strokes
        let keyboard_input = read()?;
        disable_raw_mode()?;
        if let Event::Key(event) = keyboard_input {
            let direction = match event.code {
                KeyCode::Up => Some(Direction::Up),
                KeyCode::Down => Some(Direction::Down),
                KeyCode::Left => Some(Direction::Left),
                KeyCode::Right => Some(Direction::Right),
                KeyCode::Char('q') => break,
                _ => None,
            };

            if let Some(direction) = direction {
                if let Some(new_board) = history.last().unwrap().move_free_space(&direction) {
                    println!("{}", &new_board);
                    if new_board.is_won() {
                        println!("You won!");
                    }
                    history.push(new_board);
                } else {
                    println!("invalid move.")
                }
            } else {
                println!("Use the arrow keys to move the 'free' space.");
            }
        }
    }
    Ok(())
}
