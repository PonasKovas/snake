use crossterm_input::{input, AsyncReader, InputEvent, KeyEvent, RawScreen};
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::thread::sleep;
use std::time::Duration;
use std::io::prelude::*;

/// A collection of all the game's components.
pub struct Game {
    snake: Snake,
    food_pos: (u16, u16),
    speed: f32, // How many times per second the snake moves and the screen is redrew
    input: AsyncReader,
    ended: bool,
    pub score: u32,
    rng: rand::rngs::ThreadRng,
}

/// This struct defines the player: position, direction and stuff.
pub struct Snake {
    direction: Direction,
    parts: HashSet<(u16, u16)>,
    ordered_parts: VecDeque<(u16, u16)>,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

pub enum Move {
    Ok,
    Crash,
}

impl Direction {
    pub fn is_opposite(self: &Direction, other: &Direction) -> bool {
        (*self as i8 + 2) % 4 == *other as i8
    }
}

impl Game {
    pub fn new() -> Game {
        // Get input ready
        let input = input();
        input
            .disable_mouse_mode()
            .expect("Can't disable mouse mode");

        let mut initial_snake_parts = HashSet::<(u16, u16)>::new();
        initial_snake_parts.insert((0, 0));
        initial_snake_parts.insert((1, 0));
        initial_snake_parts.insert((2, 0));

        let mut initial_ordered_snake_parts = VecDeque::<(u16, u16)>::new();
        initial_ordered_snake_parts.push_back((0, 0));
        initial_ordered_snake_parts.push_back((1, 0));
        initial_ordered_snake_parts.push_back((2, 0));

        Game {
            snake: Snake {
                direction: Direction::Right,
                parts: initial_snake_parts,
                ordered_parts: initial_ordered_snake_parts,
            },
            food_pos: (5, 0),
            speed: 5.0,
            input: input.read_async(),
            ended: false,
            score: 0,
            rng: thread_rng(),
        }
    }
    /// Draws the frames.
    pub fn draw(self: &mut Game) {
        // handle the input
        self.handle_input();

        let terminal_size = get_terminal_size();

        clear_terminal();

        // Draw the frame
        let mut frame = Vec::<u8>::new();

        for y in 0..terminal_size.1 {
            for x in 0..terminal_size.0 {
                // See if there's snake on this position
                if self.snake.parts.contains(&(x, y)) {
                    frame.extend_from_slice(b"\x1b[97m\x1b[107m  \x1b[0m"); // A white square
                    continue;
                }

                // If there's food in this position
                if (x, y) == self.food_pos {
                    frame.extend_from_slice(b"\x1b[92m\x1b[102m  \x1b[0m"); // A light-green square
                } else {
                    frame.extend_from_slice(b"  ");
                }
            }
            frame.extend_from_slice(b"\r\n");
        }

        // Add the status line at the bottom
        let status_text = format!("Score: {}", self.score);
        frame.extend_from_slice(b"\x1b[104m\x1b[30m");
        frame.extend_from_slice(" ".repeat( (((terminal_size.0 * 2) as usize - status_text.len()) as f64 / 2f64).floor() as usize).as_bytes());
        frame.extend_from_slice(status_text.as_bytes());
        frame.extend_from_slice(" ".repeat( (((terminal_size.0 * 2) as usize - status_text.len()) as f64 / 2f64).ceil() as usize - 1).as_bytes());
        frame.extend_from_slice(b"\x1b[0m");

        // Print it to the terminal
        print!("{}", String::from_utf8(frame).unwrap());
        std::io::stdout().flush().unwrap();
    }
    /// Handles the user input and moves the snake accordingly
    fn handle_input(self: &mut Game) {
        let mut new_direction = self.snake.direction;

        for event in &mut self.input {
            match event {
                // ctrl-c or Q to quit the game
                InputEvent::Keyboard(KeyEvent::Ctrl('c'))
                | InputEvent::Keyboard(KeyEvent::Char('q')) => {
                    self.ended = true;
                    RawScreen::disable_raw_mode()
                        .expect("Failed to put terminal into normal mode.");
                    return;
                }
                // A or Left arrow - move left
                InputEvent::Keyboard(KeyEvent::Char('a')) | InputEvent::Keyboard(KeyEvent::Left) => {
                    new_direction = Direction::Left;
                }
                // S or Down arrow - move down
                InputEvent::Keyboard(KeyEvent::Char('s')) | InputEvent::Keyboard(KeyEvent::Down) => {
                    new_direction = Direction::Down;
                }
                // D or Right arrow - move right
                InputEvent::Keyboard(KeyEvent::Char('d')) | InputEvent::Keyboard(KeyEvent::Right) => {
                    new_direction = Direction::Right;
                }
                // W or Up arrow - move up
                InputEvent::Keyboard(KeyEvent::Char('w')) | InputEvent::Keyboard(KeyEvent::Up) => {
                    new_direction = Direction::Up;
                }
                _ => (),
            }
        }
        if self.snake.direction.is_opposite(&new_direction) {
            new_direction = self.snake.direction;
        }
        self.snake.direction = new_direction;
        // Move the snake
        if let Move::Crash = self.move_snake() {
            self.ended = true;
        }
    }

    /// Starts the game
    pub fn start(self: &mut Game) {
        // Put the terminal into raw mode
        RawScreen::into_raw_mode()
            .expect("Failed to put terminal into raw mode.")
            .disable_drop();

        loop {
            self.draw();
            if self.ended {
                RawScreen::disable_raw_mode().expect("Failed to put terminal into normal mode.");
                return;
            }
            sleep(Duration::from_millis((1000f64 / self.speed as f64) as u64));
        }
    }
    fn generate_food_pos(self: &mut Game) -> (u16, u16) {
        // Get terminal size
        let terminal_size = get_terminal_size();
        loop {
            let food_pos: (u16, u16) = (
                self.rng.gen_range(0, terminal_size.0) as u16,
                self.rng.gen_range(0, terminal_size.1 - 3) as u16,
            );
            // If the snake is on the food, generate another value

            if self.snake.parts.contains(&food_pos) {
                continue;
            }
            return food_pos;
        }
    }
    fn move_snake(self: &mut Game) -> Move {
        // Remove the last part
        let terminal_size = get_terminal_size();

        let mut new_head_pos = *self.snake.ordered_parts.back().unwrap();

        let (dx, dy) = match self.snake.direction {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Up => (0, -1),
        };
        let width = terminal_size.0 as i16;
        let height = terminal_size.1 as i16;
        new_head_pos.0 = (((new_head_pos.0 as i16 + dx) + width) % width) as u16;
        new_head_pos.1 = (((new_head_pos.1 as i16 + dy) + height) % height) as u16;

        // If the head is on food, eat it
        if *self.snake.ordered_parts.back().unwrap() == self.food_pos {
            self.score += 1;
            self.speed += 0.1;
            self.food_pos = self.generate_food_pos();
        } else {
            // Only remove the last part if no food was eaten
            let last_part_pos = self.snake.ordered_parts.pop_front().unwrap();
            self.snake.parts.remove(&last_part_pos);
        }

        // See if the snake crashed
        if self.snake.parts.contains(&new_head_pos) {
            print!("{}", 7 as char);
            return Move::Crash;
        }

        self.snake.ordered_parts.push_back(new_head_pos);
        self.snake.parts.insert(new_head_pos);
        Move::Ok
    }
}

/// Returns terminal size
pub fn get_terminal_size() -> (u16, u16) {
    if let Some((mut w, h)) = term_size::dimensions() {
        // Width must be even.
        if w % 2 == 1 {
            w -= 1;
        }
        ((w / 2) as u16, h as u16 - 1)
    } else {
        panic!("Can't get terminal size!");
    }
}

/// Clears the terminal screen, making it ready for drawing the next frame
pub fn clear_terminal() {
    print!("\x1b[2J\x1b[H");
}

fn main() {
    let mut game = Game::new();
    game.start();
}
