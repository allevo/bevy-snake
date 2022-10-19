#[derive(Debug, Clone)]
pub enum CellField {
    Empty,
    Wall,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    pub fn allows(&self, direction: &Self) -> bool {
        !matches!(
            (self, direction),
            (Direction::Up, Direction::Down)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
                | (Direction::Down, Direction::Up)
        )
    }
}
