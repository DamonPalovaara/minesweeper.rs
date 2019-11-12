use read_input::prelude::*; // Powerful library to fetch user input.
                            // Library auto parses to desired type and allows you to set up ranges data can be within
                            // Will keep asking user to try again until inputed data passes your checks
use rand::thread_rng;       // Used for generating random locations for the bombs
use rand::seq::SliceRandom; // ^^^

fn main() {
	// Get the size of the board
	let size = Point {
		x: input()                                                    // Ask user for input until it matches desired type
			.inside_err(1..99, "Value must be between 1 & 98")        // Checks if input is in desired range printing the message if not 
			.repeat_msg("Enter a width: ")                            // User command to fetch input
			.err("Value must be a positive number, try again").get(), // Error message if data isn't of right type
		y: input()
			.inside_err(1..99, "Value must be between 1 & 98")
			.repeat_msg("Enter a height: ")
			.err("Value must be a positive number, try again").get(),
	};

	// Get how many mines to plant
	let num_mines = input()
		.inside_err(0..=(size.x * size.y - 10), "Give yourself at least 10 cells without bombs")
		.repeat_msg("How many mines? ")
		.err("Value must be a postive number, try again").get();

	let mut game = Game::new( size, num_mines );
	let mut input = TerminalInput::new( size );
	
	game.draw();

	// The mainloop
	while game.is_running { game.process_command( input.get_command() ); }
}

// Responsible for fetching data from user and sending commands to the game
// Must ensure that commands being sent are fixed for off by 1 errors and are in bounds
struct TerminalInput{
	input: String, // Place to store the inputed data
	size:  Point,  // Size of the board used for making in-bounds checks
} impl TerminalInput {
	fn new(size: Point) -> TerminalInput {
		TerminalInput {
			input: String::from(""),
			size: size,
		}
	}
	// Turns user input into a command enum to be sent to the game
	fn get_command(&mut self) -> Command {
		loop {
			self.input = input() // Make this a struct variable and change this line with self.input.get()
				.repeat_msg("Enter a command: ")
				.default(String::from(""))
				.get().to_uppercase();
			match self.input.as_ref() {
				"SELECT" => return Command::Select(self.get_point()),
				"FLAG"   => return Command::Flag(self.get_point()),
				"DRAW"   => return Command::Draw,
				"HELP"   => self.print_commands(),
				"QUIT"   => return Command::Quit,
				_        => println!("Invalid command, try again (type help to list commands)")
			}
		}
	}
	// Creates an in bounds point corrected for off by 1
	fn get_point(&self) -> Point {
		return Point {
			x: input()
				.inside_err(1..=self.size.x, "x is out of bounds, try again")
				.repeat_msg("Enter a x coordinate: ")
				.err("Input must be a positive number, try again").get() - 1,
			y: input()
				.inside_err(1..=self.size.y, "y is out of bounds, try again")
				.repeat_msg("Enter a y coordinate: ")
				.err("Input must be a positive number, try again").get() - 1
		}
	}
	// Prints out the commands for the user
	fn print_commands(&self) {
		println!("");
		println!("Select: Select a cell");
		println!("Flag: Toggle if cell is flag or not");
		println!("Draw: Redraws the board");
		println!("Help: Prints the commands");
		println!("Quit: Quits the program");
		println!("Commands are not case sensetive");
		println!("");
	}
}

// Commands to be processed by game, should be reusable for when I implement a graphics lib
enum Command {
	Select(Point), // Select cell at point, x & y are human coordinates
	Flag(Point),   // Flag cell at point, x & y are human coordinates
	Draw,          // Redraw the board
	Reset,         // Reset the game
	Quit,          // Quit the game
}

#[derive(Copy, Clone)]
// Basic struct used to index the board
struct Point { x: usize, y: usize } 

// All the game components are contained here
// Consider refactoring out the grid into it's own struct
// Make grid responsible for updating/drawing the board
// Game should be responsible for capturing input and sending it to the grid
struct Game {
	size: Point,          // Size of board
	bombs: usize,         // Number of bombs
	grid: Vec<Vec<Cell>>, // 2D grid of cells
	is_generated: bool,   // Flag to keep track of if the board was generated
	is_running: bool      // True while game is running
} impl Game {
	// Initalize the board with a size and number of mines
	fn new(size: Point, bombs: usize) -> Game {
		Game {
			size:  size.clone(),
			bombs: bombs,
			grid: vec![ vec![ Cell::new( false ); size.y ]; size.x ],
			is_generated: false,
			is_running:   true,
		}
	}
	// Implement preset sizes here...

	// Command handler
	fn process_command(&mut self, command: Command) {
		match command {
			Command::Select( index ) => {
				// Generate the bombs after first selection is known
				// that way you never start off by clicking on a bomb
				if !self.is_generated { // This is a bit hackish, maybe find a better place to put this
					self.generate_bombs(index.x as i32, index.y as i32);
					self.count_bombs();
					self.is_generated = true;
				}
				self.select_cell( index );
				self.draw();
			},
			Command::Flag( index ) => { 
				self.grid[index.x][index.y].toggle_flag(); 
				self.draw(); 
			},
			Command::Draw  => self.draw(),
			Command::Reset => {}, // TODO: Implement
			Command::Quit  => { self.is_running = false; }
		}
	}

	fn draw(&self) {
		// Draw the x indexes above each column
		print!("   ");		
		for x in 0..self.size.x { print!("{:02} ", x + 1); }
		println!();
		
		for y in 0..self.size.y {
			// Draw y index before each row
			print!("{:02} ", y + 1);

			for x in 0..self.size.x { self.grid[x][y].draw(); }
			println!();
		}
	}

	// Call this after selecting first cell, selected cell is never a bomb
	fn generate_bombs(&mut self, sel_x: i32, sel_y: i32) {
		// The purpose of this method is to create a vector of all the bombs then
		// shuffle them and take the first n bombs that aren't in the 3x3 safe zone
		let mut cells = Vec::new();
		for y in 0..self.size.y as i32 {
			for x in 0..self.size.x as i32 { cells.push((x, y)); }
		}
		cells.shuffle(&mut thread_rng());
		let mut count = 0;
		cells.iter().for_each( |cell| {
			if count == self.bombs { return; }
			// This checks if cell is within the 3x3 safezone around initial selection
			if !((sel_x - 1 <= cell.0 && cell.0 <= sel_x + 1) && (sel_y - 1 <= cell.1 && cell.1 <= sel_y + 1)) {
				self.grid[ cell.0 as usize ][ cell.1 as usize ].is_bomb = true;
				count += 1;
			}			
		});
	}

	// Counts and stores the number of neighboring bombs for each cell
	fn count_bombs(&mut self) {
		let mut bombs = 0u8;
		for y in 0..self.size.y {
			for x in 0..self.size.x {
				self.get_neighbors( x as i32, y as i32 ).iter().for_each( |neighbor| {
					if self.grid[neighbor.x][neighbor.y].is_bomb { bombs += 1; }
				});
				self.grid[x][y].bombs = bombs;
				bombs = 0;
			}
		}
	}

	// Returns a vector of indexes to neighboring cells
	fn get_neighbors(&self, loc_x: i32, loc_y: i32) -> Vec<Point> {
		let mut neighbors = Vec::with_capacity(8);
		for y in -1..=1 {
			for x in -1..=1 {
				if x == 0 && y == 0 { continue; }
				if loc_x + x < 0 || loc_x + x >= self.size.x as i32 { continue; }
				if loc_y + y < 0 || loc_y + y >= self.size.y as i32 { continue; }

				neighbors.push( Point { x: (loc_x + x) as usize, y: (loc_y + y) as usize } );
			}
		}
		neighbors
	}

	// Handle clicked on cell, index is the 2D index of the cell
	// Method assumes that index is in bounds
	fn select_cell(&mut self, index: Point) {
		if self.grid[index.x][index.y].is_flag {
			println!("Cell is a flag, remove it first");
		}		
		else if self.grid[index.x][index.y].is_bomb {
			println!("GAME OVER!");
		}
		else if self.grid[index.x][index.y].is_visible {
			println!("Already visible");
		}
		else {
			self.make_visible(index);
		}
	}

	// This is a recursive method to make cells visible
	fn make_visible(&mut self, index: Point) {
		// The base case
		if self.grid[index.x][index.y].is_visible { return; }
		
		self.grid[index.x][index.y].is_visible = true;
		// If cell is not touching any bombs call this method on all of it's neighbors
		if self.grid[index.x][index.y].bombs == 0 {
			self.get_neighbors(index.x as i32, index.y as i32).iter().for_each( |neighbor| {
				self.make_visible(*neighbor);
			});
		}
	}
}

#[derive(Debug, Copy, Clone)]
// Individual cell
// Has flags and contains how many bombs it touches
// Has it's own draw method
struct Cell {
	is_bomb:    bool, // Is the cell a bomb?
	is_visible: bool, // Is the cell visible?
	is_flag:    bool, // Is the cell a flag?
	bombs:      u8,   // Number of neighboring bombs
} impl Cell {
	fn new(is_bomb: bool) -> Cell {
		Cell {
			is_bomb:    is_bomb,
			is_visible: false,
			is_flag:    false,
			bombs:      0 // Game struct handles this
		}
	}
	// Draw individual cell
	fn draw(& self) {
		if       self.is_flag    { print!("F  "); }
		else if !self.is_visible { print!(".  "); }
		else if  self.bombs == 0 { print!("   "); }
		else                     { print!("{}  ", self.bombs); }
	}
	// Toggle flag
	fn toggle_flag(&mut self) { self.is_flag = !self.is_flag; }
}