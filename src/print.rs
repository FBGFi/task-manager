use std::io::stdout;
use crossterm::{ cursor, queue };

use crate::utils::get_terminal_dimensions;

pub fn print_on_last_row(text: &str) {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    print!("{}", text);
}
