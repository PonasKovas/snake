use rand::prelude::*;
use crossterm_input::{input, AsyncReader, RawScreen, InputEvent, KeyEvent};
use std::thread::sleep;
use std::time::Duration;


/// A collection of all the game's components.
pub struct Game {
	snake: Snake,
	food_pos: (u16, u16),
	speed: f32,
	input: AsyncReader,
	ended: bool,
	pub score: u32
}

/// This struct defines the player: position, direction and stuff.
pub struct Snake {
	direction: Direction,
	previous_direction: Direction,
	parts: Vec<(u16, u16)>
}

#[derive(PartialEq, Copy, Clone)]
pub enum Direction {
	Left,
	Right,
	Up,
	Down
}

pub enum Move {
	Ok,
	Crash
}

impl Game {
	pub fn new() -> Game {
		// Initialize game

		// Put the terminal into raw mode and start reading input
		RawScreen::into_raw_mode().expect("Failed to put terminal into raw mode.").disable_drop();

		// Get input ready
		let input = input();
		input.disable_mouse_mode().expect("Can't disable mouse mode");

		Game {
			snake: Snake {
				direction: Direction::Right,
				previous_direction: Direction::Right,
				parts: vec![(0,0), (1,0), (2,0)]
			},
			food_pos: (5, 0),
			speed: 5.0,
			input: input.read_async(),
			ended: false,
			score: 0
			}
	}
	/// Handles the user input and draws frames.
	pub fn draw(self: &mut Game) {
		let terminal_size = get_terminal_size();

		// Handle the input
		self.snake.previous_direction = self.snake.direction;
		let mut new_direction = self.snake.direction;
		for event in &mut self.input {
			match event {
				// ctrl-c or Q to quit the game
				InputEvent::Keyboard(KeyEvent::Ctrl('c')) | InputEvent::Keyboard(KeyEvent::Char('q')) => {
					self.ended = true;
					RawScreen::disable_raw_mode().expect("Failed to put terminal into normal mode.");
					return;
				},
				// A or Left arrow - move left
				InputEvent::Keyboard(KeyEvent::Char('a')) | InputEvent::Keyboard(KeyEvent::Left) => {
					if self.snake.previous_direction != Direction::Right {
						new_direction = Direction::Left;
					}
				},
				// S or Down arrow - move down
				InputEvent::Keyboard(KeyEvent::Char('s')) | InputEvent::Keyboard(KeyEvent::Down) => {
					if self.snake.previous_direction != Direction::Up {
						new_direction = Direction::Down;
					}
				},
				// D or Right arrow - move right
				InputEvent::Keyboard(KeyEvent::Char('d')) | InputEvent::Keyboard(KeyEvent::Right) => {
					if self.snake.previous_direction != Direction::Left {
						new_direction = Direction::Right;
					}
				},
				// W or Up arrow - move up
				InputEvent::Keyboard(KeyEvent::Char('w')) | InputEvent::Keyboard(KeyEvent::Up) => {
					if self.snake.previous_direction != Direction::Down {
						new_direction = Direction::Up;
					}
				},
				_ => ()
			}
		}
		self.snake.direction = new_direction;
		// Move the snake
		if let Move::Crash = self.move_snake() {
			self.ended = true;
		}
		
		clear_terminal();
		// Draw the frame
		let mut frame = String::from("");
		
		let status_text = format!("Score: {}", self.score);
		frame += &(
			"\x1b[104m\x1b[30m".to_owned() + 
			&" ".repeat((((terminal_size.0*2) as usize - status_text.len()) as f64/2f64).floor() as usize) + 
			&status_text + 
			&" ".repeat((((terminal_size.0*2) as usize - status_text.len()) as f64/2f64).ceil() as usize) + 
			"\x1b[0m\r\n"
			);
		
		for y in 0..terminal_size.1-1 {
			'column: for x in 0..terminal_size.0 { // -1 because the last line is for displaying stats
				// Iterate throught all snake's parts to find out if there's a part on this position
				for part in &self.snake.parts {
					if *part == (x, y) {
						frame += "\x1b[97m\x1b[107m  \x1b[0m"; // A white square
						continue 'column;
					}
				}

				// If there's food in this position
				if (x, y) == self.food_pos {
					frame += "\x1b[92m\x1b[102m  \x1b[0m"; // A light-green square
				}
				else {
					frame += "  ";
				}
			}
			frame += "\r\n";
		}
		// Remove last two characters: \r\n
		frame.pop();
		frame.pop();
		print!("{}", frame);
	}
	/// Starts the game
	pub fn start(self: &mut Game) {
		loop {
			self.draw();
			if self.ended {
				RawScreen::disable_raw_mode().expect("Failed to put terminal into normal mode.");
				break;
			}
			sleep( Duration::from_millis( (1000f64/self.speed as f64) as u64 ) );
		}
	}
	fn generate_food_pos(self: &Game) -> (u16, u16) {
		// Get terminal size
		let terminal_size = get_terminal_size();
		'outer: loop {
			let mut rng = thread_rng();
			let food_pos: (u16, u16) = ( rng.gen_range(0, terminal_size.0) as u16, rng.gen_range(0, terminal_size.1-3) as u16 );
			// Check if the snake is not on the food pos
			for part in &self.snake.parts {
				if food_pos == *part {
					continue 'outer;
				}
			}
			return food_pos;
		}
	}
	pub fn move_snake (self: &mut Game) -> Move {
		// Remove the last part
		let terminal_size = get_terminal_size();
		let mut new_head_pos = *self.snake.parts.last().unwrap();
		match self.snake.direction {
			Direction::Left => {
				if new_head_pos.0 == 0 {
					new_head_pos.0 = terminal_size.0-1;
				} else {
					new_head_pos.0 -= 1;
				}
			},
			Direction::Right => {
				if new_head_pos.0 == terminal_size.0-1 {
					new_head_pos.0 = 0;
				} else {
					new_head_pos.0 += 1;
				}
			},
			Direction::Down => {
				if new_head_pos.1 == terminal_size.1-3 {
					new_head_pos.1 = 0;
				} else {
					new_head_pos.1 += 1;
				}
			},
			Direction::Up => {
				if new_head_pos.1 == 0 {
					new_head_pos.1 = terminal_size.1-3;
				} else {
					new_head_pos.1 -= 1;
				}
			}
		}
		// Iterate through all other parts to see if the snake crashed
		for part in &self.snake.parts[1..] {
			if new_head_pos == *part {
				return Move::Crash
			}
		}

		// If the head is on food, eat it
		if *self.snake.parts.last().unwrap() == self.food_pos {
			self.score += 1;
			self.speed += 0.5;
			self.food_pos = self.generate_food_pos();
		} else {
			// Only remove the last part if no food was eaten
			self.snake.parts.remove(0);
		}
		
		self.snake.parts.push(new_head_pos);
		Move::Ok
	}
}

/// Returns terminal size
pub fn get_terminal_size() -> (u16, u16) {
	if let Some((mut w, h)) = term_size::dimensions() {
		// Width must be even.
		if w%2==1 {
			w -= 1;
		}
		return ((w/2) as u16, h as u16);
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
    println!("\x1b[5m\x1b[34m - - Congrats, your score was {}! - -\x1b[0m\x1b[25m", game.score);
}