use crossterm_input::{input, AsyncReader, InputEvent, KeyEvent, RawScreen};
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::env;
use std::fs;
use std::path::PathBuf;

/// A collection of all the game's components.
pub struct Game {
    snake: Snake,
    food_pos: (u16, u16),
    speed: f32, // How many times per second the snake moves and the screen is redrawn
    input: AsyncReader,
    ended: bool,
    pub score: u32,
    rng: rand::rngs::ThreadRng,
    paused: bool,
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
    pub fn is_opposite(self: Direction, other: Direction) -> bool {
        (self as i8 + 2) % 4 == other as i8
    }
}

impl Game {
    pub fn new(start_level: u16) -> Game {
        // Get input ready
        let input = input();
        input
            .disable_mouse_mode()
            .expect("Can't disable mouse mode");

        let terminal_width: u16 = get_terminal_size().0;

        let mut initial_snake_parts = HashSet::<(u16, u16)>::new();
        let mut initial_ordered_snake_parts = VecDeque::<(u16, u16)>::new();

        //create snake with size of starting level (+3 because you start with a length of 3)
        //if the snake is longer than the terminal width it loops on the line below so it doesn't crash
        //for example with a terminal width of 4 and a length of 15
        //_____________
        //| 1| 2| 3| 4|
        //| 6| 7| 8| 5|
        //|11|12| 9|10|
        //|  |13|14|15|
        //-------------
        for i in 1..(start_level + 3) {
            initial_snake_parts.insert(((i - (i / terminal_width)) % terminal_width, i / terminal_width));
            initial_ordered_snake_parts.push_back(((i - (i / terminal_width)) % terminal_width, i / terminal_width));
        }

        let mut new_game: Game = Game {
            snake: Snake {
                //if there is less than 5 cells before the snake will crash into itself then set the direction to down
                direction: Direction::Down,
                parts: initial_snake_parts,
                ordered_parts: initial_ordered_snake_parts,
            },
            // placeholder
            food_pos: (0, 0),
            speed: 5.0 + ((start_level as f32) * 0.1),
            input: input.read_async(),
            ended: false,
            score: start_level as u32,
            rng: thread_rng(),
            paused: false,
        };

        new_game.food_pos = new_game.generate_food_pos();

        new_game
    }
    /// Draws the frames.
    pub fn draw(self: &mut Game) {
        // handle the input
        self.handle_input();

        let terminal_size = get_terminal_size();

        // Clear terminal screen
        print!("\x1b[H");

        let real_terminal_size = if let Some((w, h)) = term_size::dimensions() {
            (w as u16, h as u16)
        } else {
            (40, 10)
        };

        let right_side_padding = &" ".repeat((real_terminal_size.0 - terminal_size.0 * 2) as usize);

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
            frame.extend_from_slice(right_side_padding.as_bytes());
        }

        // Add the status line at the bottom
        let status_text = format!("Score: {}", self.score);
        frame.extend_from_slice(b"\x1b[104m\x1b[30m");
        frame.extend_from_slice(
            " ".repeat(
                ((real_terminal_size.0 as usize - status_text.len()) as f64 / 2f64).floor()
                    as usize,
            )
            .as_bytes(),
        );
        frame.extend_from_slice(status_text.as_bytes());
        frame.extend_from_slice(
            " ".repeat(
                ((real_terminal_size.0 as usize - status_text.len()) as f64 / 2f64).ceil() as usize,
            )
            .as_bytes(),
        );
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
                    // Show cursor
                    print!("\x1b[?25h\r\n");
                    RawScreen::disable_raw_mode()
                        .expect("Failed to put terminal into normal mode.");
                    return;
                }
                // A or Left arrow - move left
                InputEvent::Keyboard(KeyEvent::Char('a'))
                | InputEvent::Keyboard(KeyEvent::Char('h'))
                | InputEvent::Keyboard(KeyEvent::Left) => {
                    new_direction = Direction::Left;
                }
                // S or Down arrow - move down
                InputEvent::Keyboard(KeyEvent::Char('s'))
                | InputEvent::Keyboard(KeyEvent::Char('j'))
                | InputEvent::Keyboard(KeyEvent::Down) => {
                    new_direction = Direction::Down;
                }
                // D or Right arrow - move right
                InputEvent::Keyboard(KeyEvent::Char('d'))
                | InputEvent::Keyboard(KeyEvent::Char('l'))
                | InputEvent::Keyboard(KeyEvent::Right) => {
                    new_direction = Direction::Right;
                }
                // W or Up arrow - move up
                InputEvent::Keyboard(KeyEvent::Char('w'))
                | InputEvent::Keyboard(KeyEvent::Char('k'))
                | InputEvent::Keyboard(KeyEvent::Up) => {
                    new_direction = Direction::Up;
                }
                InputEvent::Keyboard(KeyEvent::Esc) => self.paused = !self.paused,
                _ => (),
            }
        }
        if self.snake.direction.is_opposite(new_direction) {
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

        // Make some room for the game frames, without overwriting history text
        // And hide the carriage
        print!("\x1b[2J\x1b[?25l");
        loop {
            self.draw();
            if self.ended {
                RawScreen::disable_raw_mode().expect("Failed to put terminal into normal mode.");
                let result: (bool, Option<u32>) = self.game_finish();
                if result.0 {
                    println!("New high score! You got {}", self.score);
                } else if let Some(highscore) = result.1 {
                    println!("You got {}, the high score is {highscore}. Try again!", self.score);
                } else {
                    println!("You got {}!", self.score);
                }
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
                self.rng.gen_range(0, terminal_size.1) as u16,
            );
            // If the snake is on the food, generate another value

            if self.snake.parts.contains(&food_pos) {
                continue;
            }
            return food_pos;
        }
    }
    fn move_snake(self: &mut Game) -> Move {
        if self.paused {
            return Move::Ok;
        }

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
    //returns true if new high score, and the previous high score
    fn game_finish(self: &mut Game) -> (bool, Option<u32>) {
        //save in $HOME/.snake if possible
        if let Some(mut full_buf) = home::home_dir() {
            full_buf.push(".snake");
            let path = full_buf.as_path();
            let mut current_high_score: u32 = 0;
            if path.exists() {
                let file_content = fs::read_to_string(path).expect("Unable to read high score file");
                current_high_score = file_content.parse().unwrap();
            }
            if self.score > current_high_score {
                fs::write(path, self.score.to_string()).expect("Unable to write high score file");
            }
            return (self.score >= current_high_score, Some(current_high_score))
        }
        (false, None)
    }
}

/// Returns terminal size
pub fn get_terminal_size() -> (u16, u16) {
    if let Some((w, h)) = term_size::dimensions() {
        ((w / 2) as u16, h as u16 - 1)
    } else {
        panic!("Can't get terminal size!");
    }
}

fn main() {
    //get start level from args, default to 0
    let args: Vec<String> = env::args().collect();
    let start_level: u16;
    if args.len() == 1 {
        start_level = 0;
    } else {
        start_level = args[1].parse::<u16>().expect("Not a number!");
    }
    let mut game = Game::new(start_level);
    game.start();
}
