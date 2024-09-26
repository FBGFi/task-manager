use std::io::stdout;
use crossterm::{ cursor, queue };
use regex::Regex;

use crate::utils::get_terminal_dimensions;

pub fn print_on_last_row(text: &str) {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    print!("{}", text);
}

pub fn empty_row(row: u16) {
    let width = get_terminal_dimensions().0;
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    for _ in 0..width {
        print!(" ");
    }
}

pub fn print_at_end_of_row(text: &str, row: u16) {
    let width = get_terminal_dimensions().0;
    queue!(stdout(), cursor::MoveTo(width - (text.chars().count() as u16), row)).unwrap();
    print!("{}", text);
}

pub fn strip_closing_quotes(text: &str) -> String {
    let re = Regex::new(r####"^\"|\"$"####).unwrap();
    let result = re.replace_all(text, "");
    return format!("{}", result);
}

pub fn truncate_text(text: &str, max_length: u16) -> String {
    if (text.chars().count() as u16) < max_length {
        return format!("{}", text);
    }
    let trailing_characters = "...";
    let truncate_length = (
        (max_length as i32) -
        (text.chars().count() as i32) -
        (trailing_characters.chars().count() as i32)
    ).abs() as u16;
    let re = Regex::new(format!(".{}{}{}$", "{", truncate_length, "}").as_str()).unwrap();
    let result = re.replace_all(text, trailing_characters);
    return format!("{}", result);
}
