mod backtracking;

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;

#[derive(Clone, Debug, Copy)]
enum Shape {
    Free,
    M1,
    M2,
    M3,
    Ship,
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
            Shape::M1 => vec![Point { x: 0, y: 0 }],
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
    fn to_index(&self) -> usize {
        (self.y * 3 + self.x) as usize
    }
    fn from_index(index: usize) -> BoardIndex2d {
        BoardIndex2d {
            x: (index % 3) as i32,
            y: (index / 3) as i32,
        }
    }
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

    fn map_shape_points_to_grid_points(
        &self,
        shape: &Shape,
        board_index: &BoardIndex2d,
        grid_x: &i32,
        grid_y: &i32,
    ) -> Vec<Point> {
        let dx: i32;
        let dy: i32;
        if *board_index == self.board_index {
            dx = self.grid_dx;
            dy = self.grid_dy;
        } else {
            dx = 0;
            dy = 0;
        }
        shape
            .get_points()
            .iter()
            .map(|point| Point {
                x: point.x + grid_x + dx,
                y: point.y + grid_y + dy,
            })
            .collect()
    }
}

impl Board {
    fn is_valid(&self) -> bool {
        // has exactly one free space
        let mut free_count = 0;

        for shape in &self.shapes {
            if let Shape::Free = shape {
                free_count += 1;
            }
        }

        // and has no collissions
        free_count == 1 && self.is_collission_free(MovingTile::no_move())
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
    fn move_free_space(&self, direction: &Direction) -> Option<Board> {
        let free_space_position = self.find_free_space();
        if let Some(neighbor) = free_space_position.neighbor(direction) {
            let mut new_shapes = self.shapes.clone();
            let free_space_index = free_space_position.to_index();
            let neighbor_index = neighbor.to_index();
            new_shapes.swap(free_space_index as usize, neighbor_index as usize);
            Some(Board { shapes: new_shapes })
        } else {
            None
        }
    }

    fn is_collission_free(&self, moving_tile: MovingTile) -> bool {
        let mut occupied_points: Vec<Point> = vec![];
        for y in 0..3 {
            for x in 0..3 {
                let board_index = BoardIndex2d { x, y };
                let shape = &self.shapes[board_index.to_index()];
                moving_tile
                    .map_shape_points_to_grid_points(shape, &board_index, &x, &y)
                    .into_iter()
                    .for_each(|point| occupied_points.push(point));
            }
        }
        !has_duplicates(&occupied_points[..])
    }
}

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
