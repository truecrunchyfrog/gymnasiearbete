//! Game logic simulator to express the result of a program solution submission.
//! This library contains the logic to handle the different events and return the result.

const BOARD_WIDTH:  u8 = 8;
const BOARD_HEIGHT: u8 = 5;

struct Game {
    player: Player,
    board: Board
}

struct Player {
    orientation: Orientation
}

struct Orientation {
    x: u8,
    y: u8,

    direction: Direction
}

#[derive(PartialEq, Debug)]
enum Direction {
            Up,

    Left,           Right,

            Down
}

struct Board {
    entities: Vec<Figure>
}

struct Figure {
    kind: FigureKind,

    orientation: Orientation
}

enum FigureKind {
    Swordsman,
    Priest,
}

trait Character {
    /// Rotates the character clockwise.
    fn rotate(&mut self) {
        let orientation = self.get_mut_orientation();

        orientation.direction = match orientation.direction {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up
        };
    }

    fn sprint(&mut self) -> Result<(), String> {
        let orientation = self.get_mut_orientation();
        
        let (delta_x, delta_y): (i8, i8) = match orientation.direction {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0)
        };

        let new_x = orientation.x as i8 + delta_x;
        let new_y = orientation.y as i8 + delta_y;

        if !within_bounds(new_x, new_y) {
            return Err(String::from("Cannot move out of board."))
        }

        orientation.x = new_x as u8;
        orientation.y = new_y as u8;

        Ok(())
    }

    fn get_orientation(&self) -> &Orientation;
    fn get_mut_orientation(&mut self) -> &mut Orientation;
}

impl Character for Player {
    fn get_mut_orientation(&mut self) -> &mut Orientation {
        &mut self.orientation
    }

    fn get_orientation(&self) -> &Orientation {
        &self.orientation
    }
}

impl Character for Figure {
    fn get_mut_orientation(&mut self) -> &mut Orientation {
        &mut self.orientation
    }

    fn get_orientation(&self) -> &Orientation {
        &self.orientation
    }
}


/// Check whether a location (x, y) is within bounds of the game board's area.
fn within_bounds(x: i8, y: i8) -> bool {
    x >= 0 && (x as u8) < BOARD_WIDTH && // 0 <= X < width
    y >= 0 && (y as u8) < BOARD_HEIGHT   // 0 <= Y < height
}



#[cfg(test)]
mod test {
    use crate::{Orientation, Direction, Figure, FigureKind, Character, within_bounds};

    #[test]
    fn test_within_bounds() {
        assert!(within_bounds(2, 4));
        assert!(!within_bounds(30, 1));

        assert!(within_bounds(5, 0));
        assert!(!within_bounds(5, -1));
    }

    #[test]
    fn character_leave_board() {
        Figure {
            kind: FigureKind::Priest,
            orientation: Orientation { x: 0, y: 0, direction: Direction::Up }
        }.sprint().expect_err("should not be able to leave board area");
    }

    #[test]
    fn character_rotate() {
        let mut figure = Figure {
            kind: FigureKind::Priest,
            orientation: Orientation { x: 0, y: 0, direction: Direction::Up }
        };

        figure.rotate(); // Rotate once clockwise.

        assert_eq!(figure.get_orientation().direction, Direction::Right);

        figure.rotate(); // Rotate again.

        assert_eq!(figure.get_orientation().direction, Direction::Down);
    }

    #[test]
    fn character_sprint() {
        let mut figure = Figure {
            kind: FigureKind::Priest,
            orientation: Orientation { x: 0, y: 0, direction: Direction::Right }
        };

        figure.sprint().expect("should be able to move within board area");

        assert_eq!(figure.get_orientation().x, 1);
    }

    #[test]
    fn character_sprint_and_rotate() {
        let mut figure = Figure {
            kind: FigureKind::Priest,
            orientation: Orientation { x: 0, y: 0, direction: Direction::Right }
        };

        figure.sprint().expect("should be able to move right from (0, 0) to (1, 0)");

        figure.rotate(); // Now facing down.

        figure.sprint().expect("should be able to run down from (1, 0) to (1, 1)");

        figure.rotate(); // Now facing left.

        figure.sprint().expect("should be able to run left from (1, 1) to (0, 1)");

        figure.sprint().expect_err("should NOT be able to run left from (0, 1) to (-1, 1) which is outside of the board");
    }
}