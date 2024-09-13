use std::io;

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, ContentStyle, PrintStyledContent, StyledContent, Stylize},
    terminal::{Clear, ClearType},
};
use rand::{thread_rng, Rng};

struct Board {
    width: u32,
    height: u32,
    cells: Vec<bool>,
}

impl Board {
    fn new(width: u32, height: u32) -> Board {
        Board {
            width,
            height,
            cells: vec![false; (width * height) as usize],
        }
    }
    fn randomize_cells(&mut self) {
        let mut rng = thread_rng();
        for x in 0..self.width {
            for y in 0..self.height {
                self.set_cell(x, y, rng.gen_bool(0.15));
            }
        }
    }
    fn tick(&mut self) -> u32 {
        let mut cells: Vec<bool> = vec![false; (self.width * self.height) as usize];
        let mut updated_cells = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                let index = get_index(self.width, x, y);
                let cell_alive = self.cells[index];
                let neighbours = self.get_neighbours(x, y);

                if cell_alive && neighbours < 2 {
                    cells[index] = false;
                    updated_cells += 1;
                    continue;
                }
                if cell_alive && neighbours <= 3 {
                    cells[index] = true;
                    continue;
                }
                if cell_alive && neighbours > 3 {
                    cells[index] = false;
                    updated_cells += 1;
                    continue;
                }
                if !cell_alive && neighbours == 3 {
                    cells[index] = true;
                    updated_cells += 1;
                    continue;
                }
            }
        }
        self.cells = cells;
        updated_cells
    }
    fn get_cell(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.cells[get_index(self.width, x, y)]
    }
    fn set_cell(&mut self, x: u32, y: u32, alive: bool) {
        self.cells[get_index(self.width, x, y)] = alive;
    }
    fn get_neighbours(&self, x: u32, y: u32) -> u32 {
        let mut neighbours = 0;
        for i in x.saturating_sub(1)..=x.saturating_add(1) {
            for j in y.saturating_sub(1)..=y.saturating_add(1) {
                if i == x && j == y {
                    continue;
                }
                if self.get_cell(i, j) {
                    neighbours += 1;
                }
            }
        }
        neighbours
    }

    fn set_size(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.cells = vec![false; (w * h) as usize];
        self.randomize_cells();
    }
}

fn get_index(w: u32, x: u32, y: u32) -> usize {
    (y * w + x) as usize
}

fn terminal_size() -> io::Result<(u32, u32)> {
    let (w, h) = crossterm::terminal::size()?;
    Ok((w as u32, h as u32))
}

fn main() -> io::Result<()> {
    let dead_char = StyledContent::new(ContentStyle::new(), ' ');
    let alive_char = '*'.on(Color::White);

    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All))?;
    let (w, h) = terminal_size()?;
    let mut board = Board::new(w, h);
    board.randomize_cells();
    // Output from last "render"
    let mut front_buffer = vec![dead_char; (w * h) as usize];
    loop {
        let (w, h) = terminal_size()?;
        if w != board.width || h != board.height {
            // Terminal was resized, reset cells
            board.set_size(w, h);
            front_buffer = vec![dead_char; (w * h) as usize];
            execute!(stdout, Clear(ClearType::All))?;
        }
        let updates = board.tick();
        if board.width * board.height / 100 > updates {
            board.randomize_cells();
        }
        let mut back_buffer = vec![dead_char; (w * h) as usize];
        for x in 0..w {
            for y in 0..h {
                if board.get_cell(x, y) {
                    back_buffer[get_index(board.width, x, y)] = alive_char;
                }
            }
        }
        for x in 0..w {
            for y in 0..h {
                let i = get_index(board.width, x, y);
                // Only print character if it changed from last render
                if front_buffer[i] != back_buffer[i] {
                    front_buffer[i] = back_buffer[i];
                    execute!(stdout, MoveTo(x as u16, y as u16))?;
                    execute!(stdout, PrintStyledContent(back_buffer[i]))?;
                }
            }
        }
        execute!(stdout, MoveTo(board.width as u16, board.height as u16))?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
