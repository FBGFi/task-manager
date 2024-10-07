mod constants;
mod state;
mod run_mode;
mod utils;

use std::io::stdout;

use crossterm::{ cursor, queue };
use run_mode::run;
use state::{ Mode, MODE };
use clearscreen;
use utils::set_current_terminal_dimensions;

fn main() {
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), cursor::Hide).unwrap();
    set_current_terminal_dimensions();
    unsafe {
        while MODE != Mode::EXIT {
            run();
        }
    }
    clearscreen::clear().expect("failed to clear");
}
