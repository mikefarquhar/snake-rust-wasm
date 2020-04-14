// TODO: Update the JS rendering to draw the snake smaller than the cell size. This should make it
// clearer when playing the game where the head is and which direction you're travelling in.

extern crate wasm_bindgen;

// Initiate bindgen to allow Rust <-> JavaScript communication.
use wasm_bindgen::prelude::*;

// Import methods from JavaScript
#[wasm_bindgen]
extern {
    // Math.random
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

// Define the board dimensions as constants rather than as runtime variables. This allows the use of
// stack allocated arrays rather than heap allocated vectors.
const BOARD_WIDTH: usize = 16;
const BOARD_HEIGHT: usize = 13;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_HEIGHT;

// The game of snake is small enough in scope to be simpler to deal with the state and logic without
// splitting things into smaller chunks.

/// The full game state for snake.
#[wasm_bindgen]
pub struct Game {
    score: u16,
    // board: [BoardItem; BOARD_SIZE],
    apple: Option<u8>,
    snake_buffer: [SnakePart; BOARD_SIZE],
    snake_head: u8,
    snake_len: u8,
}

#[wasm_bindgen]
impl Game {
    /// Create a new game instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut game = Game {
            score: 0,
            // board: [BoardItem::Empty; BOARD_SIZE],
            apple: Option::None,
            snake_buffer: [SnakePart::default(); BOARD_SIZE],
            snake_head: 3,
            snake_len: 4,
        };

        game.snake_buffer[0].board_index = 102;
        game.snake_buffer[1].board_index = 103;
        game.snake_buffer[2].board_index = 104;
        game.snake_buffer[3].board_index = 105;

        game.place_apple();

        game
    }

    /// Reset the game.
    pub fn reset (&mut self) {
        self.score = 0;

        self.snake_head = 3;
        self.snake_len = 4;
        
        self.snake_buffer[0].board_index = 102;
        self.snake_buffer[0].direction = Direction::Right;
        self.snake_buffer[1].board_index = 103;
        self.snake_buffer[1].direction = Direction::Right;
        self.snake_buffer[2].board_index = 104;
        self.snake_buffer[2].direction = Direction::Right;
        self.snake_buffer[3].board_index = 105;
        self.snake_buffer[3].direction = Direction::Right;

        self.place_apple();
    }

    /// Get the width of the game board.
    pub fn get_width(&self) -> usize {
        // If the implementation changes then this can be updated to return the correct value.
        BOARD_WIDTH
    }

    /// Get the height of the game board.
    pub fn get_height(&self) -> usize {
        // If the implementation changes then this can be updated to return the correct value.
        BOARD_HEIGHT
    }

    pub fn get_snake_len(&self) -> u8 {
        self.snake_len
    }

    pub fn get_snake_part(&self, n: u8) -> *const SnakePart {
        self.get_snake_point(n)
    }

    pub fn get_apple(&self) -> Option<u8> {
        self.apple
    }

    /// Get the current score.
    pub fn get_score(&self) -> u16 {
        self.score
    }

    /// Update the game state to the next state.
    pub fn update(&mut self, input: Option<Direction>) -> UpdateResult {
        let board_size = BOARD_SIZE as u8;
        let board_width = BOARD_WIDTH as u8;

        // Figure out which way snake is moving and fix direction if the user input is not valid.
        let prev = self.get_snake_point(0);
        let head = prev.board_index;
        let prev_dir = prev.direction;

        let dir = match (input, prev_dir) {
            (None, prev_dir) => prev_dir,
            (Some(Direction::Up), Direction::Down) => prev_dir,
            (Some(Direction::Left), Direction::Right) => prev_dir,
            (Some(Direction::Down), Direction::Up) => prev_dir,
            (Some(Direction::Right), Direction::Left) => prev_dir,
            (Some(dir), _) => dir,
        };

        // Get next head position. Return a GameOver if the snake has collided with the edge.
        let next_point = match dir {
            Direction::Up if head < board_width => return UpdateResult::GameOver,
            Direction::Down if head >= board_size - board_width => return UpdateResult::GameOver,
            Direction::Left if head % board_width == 0 => return UpdateResult::GameOver,
            Direction::Right if head % board_width == board_width - 1 => return UpdateResult::GameOver,
            Direction::Up => head - board_width,
            Direction::Down => head + board_width,
            Direction::Left => head - 1,
            Direction::Right => head + 1,
        };

        // Return a GameOver if the snake has hit its own tail.
        // The snake can only collide with itself if it has a length of 5 or more. Don't check the
        // last tail position as it will be moving out of it's cell.
        if self.snake_len > 4 {
            for i in 3..self.snake_len - 1 {
                let snake_point = self.get_snake_point(i).board_index;
                if next_point == snake_point {
                    return UpdateResult::GameOver;
                }
            }
        }

        // Move the snake
        self.snake_head = (self.snake_head + 1) % board_size;
        let snake_point = &mut self.snake_buffer[self.snake_head as usize];
        snake_point.board_index = next_point;
        snake_point.direction = dir;

        // Update the apple
        match self.apple {
            Option::Some(apple) => {
                if next_point == apple {
                    // If the snake has collided with the apple we add a copy of the head to the 
                    // snake. This will cause the snake to grow as expected.
                    self.snake_len += 1;
                    self.snake_head = (self.snake_head + 1) % BOARD_SIZE as u8;
                    let snake_point = &mut self.snake_buffer[self.snake_head as usize];
                    snake_point.board_index = next_point;
                    snake_point.direction = dir;
                    self.score += 1;
                    
                    // Attempt to replace the apple.
                    self.place_apple();

                }
            },
            // There is no apple to collide with or draw. Technically signals the end of the game,
            // but following the logic of the 3310 game the user has to end the game even on a
            // perfect run.
            _ => ()
        };

        // Let the caller know to keep continuing the game.
        UpdateResult::Running
    }

    /// Get the nth point in the snake from the head to the tail.
    fn get_snake_point(&self, n: u8) -> &SnakePart {
        &self.snake_buffer[(self.snake_head as usize + BOARD_SIZE - n as usize) % BOARD_SIZE]
    }

    /// Attempts to find a free location to place an apple.
    fn place_apple (&mut self) {
        let num_free_spaces = BOARD_SIZE as u8 - self.snake_len;
        if num_free_spaces > 0 {
            // Choose the nth random free slot.
            let space = (random() * num_free_spaces as f64).floor() as u8;

            // Mark which cells of the board are free/filled.
            let mut free_cells = [false; BOARD_SIZE];
            for i in 0.. self.snake_len {
                let index = self.get_snake_point(i).board_index;
                free_cells[index as usize] = true;
            }

            // Find the board index of the random slot.
            let mut next_apple: usize = 0;
            let mut counter = 0;
            for i in 0..BOARD_SIZE {
                match free_cells[i] {
                    false => {
                        if counter == space {
                            next_apple = i;
                            break;
                        } else {
                            counter += 1;
                        }
                    },
                    _ => () // Skip non-empty cells.
                }
            }

            // Store the new location of the apple.
            self.apple = Option::Some(next_apple as u8);
        } else {
            // There's no free places to put the apple.
            self.apple = Option::None;
        }
    }
}

/// The possible directions the inputs can map to.
#[wasm_bindgen]
#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

/// The result of running the game. Lets the caller know how to proceed.
#[wasm_bindgen]
#[repr(u8)]
pub enum UpdateResult {
    Running,
    GameOver
}

/// A single part of the snake (a single cell on the game board).
#[derive(Copy, Clone)]
pub struct SnakePart {
    board_index: u8,
    direction: Direction,
}

impl Default for SnakePart {
    fn default() -> Self {
        SnakePart {
            board_index: 0,
            direction: Direction::Right,
        }
    }
}
