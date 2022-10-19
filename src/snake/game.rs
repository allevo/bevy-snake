use std::{ops::Deref, str::FromStr};

use super::common::{CellField, Direction, Position};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnakeError {
    #[error("Snake is on the wall at {0:?}")]
    OnWall(Position),
    #[error("Snake is eating itself as {0:?}")]
    OnSnake(Position),
}

#[derive(Debug)]
struct Row {
    cells: Vec<CellField>,
}
impl Deref for Row {
    type Target = Vec<CellField>;

    fn deref(&self) -> &Self::Target {
        &self.cells
    }
}

#[derive(Debug)]
struct Map {
    rows: Vec<Row>,
    dimension: (usize, usize),
}
impl Deref for Map {
    type Target = Vec<Row>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

#[derive(Debug)]
pub struct SnakeGame {
    map: Map,
    snake_head: Position,
    snake_body: Vec<Position>,
    food: Position,
    direction: Direction,
    increment_size: usize,
}

impl SnakeGame {
    pub fn play(&mut self, mut direction: Direction) -> Result<SnakeGameSnapshot, SnakeError> {
        info!("play with {:?}", direction);

        // if the given direction is not allowed we ignore it
        if !self.direction.allows(&direction) {
            direction = self.direction;
        }

        self.move_body();
        self.move_head(&direction);

        if self.on_walls(&self.snake_head) {
            return Err(SnakeError::OnWall(self.snake_head.clone()));
        }

        if self.on_snake_body(&self.snake_head) {
            return Err(SnakeError::OnSnake(self.snake_head.clone()));
        }

        let food_ate = self.on_food(&self.snake_head);
        if food_ate {
            self.increment_size = 1;
            self.food = self.create_new_food()
        }

        self.direction = direction;

        Ok(self.snapshot_with_food_ate(food_ate))
    }

    pub fn snapshot(&self) -> SnakeGameSnapshot {
        self.snapshot_with_food_ate(false)
    }

    pub fn dimension(&self) -> (usize, usize) {
        self.map.dimension
    }

    pub fn on_walls(&self, position: &Position) -> bool {
        position.x >= self.map.dimension.0
            || position.y >= self.map.dimension.1
            || matches!(self.map[position.y][position.x], CellField::Wall)
    }

    fn snapshot_with_food_ate(&self, food_ate: bool) -> SnakeGameSnapshot {
        let mut snake_snapshot = self.snake_body.clone();
        snake_snapshot.insert(0, self.snake_head.clone());

        SnakeGameSnapshot {
            food: self.food.clone(),
            snake: snake_snapshot,
            food_ate,
        }
    }

    fn on_food(&self, position: &Position) -> bool {
        position == &self.food
    }

    fn on_snake_body(&self, position: &Position) -> bool {
        self.snake_body.contains(position)
    }

    fn move_head(&mut self, direction: &Direction) {
        let (dx, dy): (isize, isize) = match direction {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        self.snake_head.x = (self.snake_head.x as isize + dx) as usize;
        self.snake_head.y = (self.snake_head.y as isize + dy) as usize;
    }

    fn move_body(&mut self) {
        if self.increment_size > 0 {
            let new_piece = self.snake_head.clone();
            self.snake_body.insert(0, new_piece);
            self.increment_size -= 1;
        } else {
            let mut tail = match self.snake_body.pop() {
                // Means the snake is just its head, so nothing to do
                None => return,
                Some(p) => p,
            };

            tail.x = self.snake_head.x;
            tail.y = self.snake_head.y;
            self.snake_body.insert(0, tail);
        }
    }

    fn create_new_food(&self) -> Position {
        loop {
            let y = fastrand::usize(0..self.map.dimension.0);
            let x = fastrand::usize(0..self.map.dimension.1);
            let position = Position::new(x, y);

            debug!("position generated {:?}", position);

            if self.on_snake_body(&position) {
                continue;
            }

            if self.on_walls(&position) {
                continue;
            }

            if self.snake_head == position {
                continue;
            }

            break position;
        }
    }
}

impl FromStr for SnakeGame {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().filter(|l| !l.is_empty());
        let dimension = lines.next().unwrap();
        let dimension = dimension.split_once(',').unwrap();
        let w: usize = dimension.0.parse().unwrap();
        let h: usize = dimension.1.parse().unwrap();

        let mut map: Vec<Vec<_>> = vec![];
        for _ in 0..h {
            let line = lines.next().unwrap();
            map.push(
                line.chars()
                    .take(w)
                    .map(|c| match c {
                        ' ' => CellField::Empty,
                        'w' => CellField::Wall,
                        _ => panic!("Unexpected char: {}", c),
                    })
                    .collect(),
            );
        }

        let food = lines.next().unwrap();
        let food = food.split_once(',').unwrap();
        let food = Position::new(food.0.parse().unwrap(), food.1.parse().unwrap());

        let snake = lines.next().unwrap();
        let mut snake: Vec<Position> = snake
            .split(';')
            .map(|t| {
                let t = t.split_once(',').unwrap();
                Position::new(t.0.parse().unwrap(), t.1.parse().unwrap())
            })
            .collect();

        let snake_head = snake.remove(0);
        let snake_body = snake;

        Ok(Self {
            map: Map {
                dimension: (map[0].len(), map.len()),
                rows: map.into_iter().map(|cells| Row { cells }).collect(),
            },
            snake_head,
            snake_body,
            food,
            direction: Direction::Up,
            increment_size: 0,
        })
    }
}

pub struct SnakeGameSnapshot {
    pub snake: Vec<Position>,
    pub food: Position,
    pub food_ate: bool,
}

#[cfg(test)]
mod tests {
    use crate::snake::{
        common::{Direction, Position},
        game::SnakeError,
    };

    use super::SnakeGame;

    #[test]
    fn test_snake_should_move_till_wall() {
        let mut game = create_game();

        let result = game.play(Direction::Up);
        assert!(result.is_ok());
        let result = game.play(Direction::Up);
        assert!(result.is_ok());
        let result = game.play(Direction::Up);
        assert!(result.is_ok());
        let result = game.play(Direction::Up);
        assert!(result.is_ok());
        let result = game.play(Direction::Up);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            SnakeError::OnWall(Position::new(2, 7))
        );
    }

    #[test]
    fn test_snake_should_move_changing_direction() {
        let mut game = create_game();

        let snapshot = game.play(Direction::Up).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(2, 3));
        assert_eq!(snapshot.snake[1], Position::new(2, 2));
        assert_eq!(snapshot.snake.len(), 2);
        let snapshot = game.play(Direction::Left).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(1, 3));
        assert_eq!(snapshot.snake[1], Position::new(2, 3));
        assert_eq!(snapshot.snake.len(), 2);
        let snapshot = game.play(Direction::Down).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(1, 2));
        assert_eq!(snapshot.snake[1], Position::new(1, 3));
        assert_eq!(snapshot.snake.len(), 2);
    }

    #[test]
    fn test_snake_eats_increasing_length() {
        let mut game = create_game();

        _ = game.play(Direction::Up).unwrap();
        let snapshot = game.play(Direction::Up).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(2, 4));
        assert_eq!(snapshot.snake.len(), 2);

        _ = game.play(Direction::Right).unwrap();
        let snapshot = game.play(Direction::Right).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(4, 4));
        assert_eq!(snapshot.snake.len(), 2);

        let snapshot = game.play(Direction::Right).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(5, 4));
        assert_eq!(snapshot.snake[1], Position::new(4, 4));
        assert_eq!(snapshot.snake[2], Position::new(3, 4));
        assert_eq!(snapshot.snake.len(), 3);

        let snapshot = game.play(Direction::Right).unwrap();
        assert_eq!(snapshot.snake[0], Position::new(6, 4));
        assert_eq!(snapshot.snake[1], Position::new(5, 4));
        assert_eq!(snapshot.snake[2], Position::new(4, 4));
        assert_eq!(snapshot.snake.len(), 3);
    }

    fn create_game() -> SnakeGame {
        let s = r#"
9,8
wwwwwwwww
w       w
w       w
w       w
w       w
w       w
w       w
wwwwwwwww
4,4
2,2;2,1"#;
        s.parse().unwrap()
    }
}
