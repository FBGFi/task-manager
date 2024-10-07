use std::io::{ stdout, Write };
use crossterm::{ cursor, event::KeyCode, queue };
use regex::Regex;

use terminal_size::{ terminal_size, Height, Width };

use crate::state::PREVIOUS_DIMENSIONS;

pub fn get_terminal_dimensions() -> (u16, u16) {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        return (w, h);
    }
    panic!("Terminal not supported");
}

pub fn set_current_terminal_dimensions() {
    let (width, height) = get_terminal_dimensions();
    unsafe {
        PREVIOUS_DIMENSIONS.height = height;
        PREVIOUS_DIMENSIONS.width = width;
    }
}

pub fn clear_screen_on_dimension_changed() {
    let (width, height) = get_terminal_dimensions();
    unsafe {
        if width != PREVIOUS_DIMENSIONS.width || height != PREVIOUS_DIMENSIONS.height {
            clearscreen::clear().expect("failed to clear");
            set_current_terminal_dimensions();
        }
    }
}

/// Moves cursor to the last row in terminal, clears it and returns initial cursor position
pub fn enter_input_mode(input_prefix: &str) -> u16 {
    let height = get_terminal_dimensions().1;
    empty_row(height);
    print_on_last_row(input_prefix);
    let cursor_start_position: u16 = input_prefix.chars().count() as u16;
    queue!(stdout(), cursor::MoveTo(cursor_start_position, height)).unwrap();
    queue!(stdout(), cursor::Show).unwrap();
    stdout().flush().ok().expect("failed to flush");
    return cursor_start_position;
}

pub fn print_on_last_row(text: &str) {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    print!("{}", text);
    stdout().flush().ok().expect("failed to flush");
}

pub fn move_cursor(row: u16, column: u16) {
    queue!(stdout(), cursor::MoveTo(column, row)).unwrap();
    stdout().flush().ok().expect("failed to flush");
}

pub fn print_on_position(text: &str, row: u16, column: u16) {
    move_cursor(row, column);
    print!("{}", text);
    stdout().flush().ok().expect("failed to flush");
}

pub fn empty_row(row: u16) {
    let width = get_terminal_dimensions().0;
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    for _ in 0..width {
        print!(" ");
    }
}

/// Removes character previous to the cursor position and returns the new cursor position
pub fn delete_previous_character(
    input_prefix: &str,
    input: &mut String,
    row: u16,
    cursor_position: u16,
    cursor_start_position: u16
) -> u16 {
    if cursor_position > cursor_start_position {
        let new_position = cursor_position - 1;
        input.remove((new_position - cursor_start_position) as usize);
        empty_row(row);
        print_on_position(format!("{input_prefix}{input}").as_str(), row, 0);
        move_cursor(row, new_position);
        return new_position;
    }
    return cursor_position;
}

/// Returns new cursor position when moving left while inputting
pub fn navigate_left_input(row: u16, cursor_position: u16, cursor_start_position: u16) -> u16 {
    if cursor_position > cursor_start_position {
        let new_position = cursor_position - 1;
        move_cursor(row, new_position);
        return new_position;
    }
    return cursor_position;
}

/// Returns new cursor position when moving right while inputting
pub fn navigate_right_input(
    input: &String,
    row: u16,
    cursor_position: u16,
    cursor_start_position: u16
) -> u16 {
    if cursor_position <= ((input.chars().count() - 1) as u16) + cursor_start_position {
        let new_position = cursor_position + 1;
        move_cursor(row, new_position);
        return new_position;
    }
    return cursor_position;
}

fn get_keycode_char(keycode: KeyCode) -> Option<char> {
    return match keycode.to_string().as_str() {
        "Space" => Some(' '),
        str => {
            if str.chars().count() == 1 {
                return Some(str.chars().nth(0).unwrap());
            }
            return None;
        }
    };
}

/// Prints to cursor position given character, inserts it into the input and returns new cursor position
pub fn print_input(
    input: &mut String,
    keycode: KeyCode,
    row: u16,
    cursor_position: u16,
    cursor_start_position: u16
) -> u16 {
    let char = get_keycode_char(keycode);

    if char.is_some() {
        let new_position = cursor_position + 1;
        input.insert((cursor_position - cursor_start_position) as usize, char.unwrap());
        let text = input.split_at((cursor_position - cursor_start_position) as usize).1;
        print_on_position(text, row, cursor_position);
        move_cursor(row, new_position);
        return new_position;
    }
    return cursor_position;
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

pub fn truncate_text(text: String, max_length: u16) -> String {
    if (text.chars().count() as u16) < max_length {
        return text;
    }
    let trailing_characters = "...";
    let truncate_length = (
        (max_length as i32) -
        (text.chars().count() as i32) -
        (trailing_characters.chars().count() as i32)
    ).abs() as u16;
    let re = Regex::new(format!(".{}{}{}$", "{", truncate_length, "}").as_str()).unwrap();
    let result = re.replace_all(text.as_str(), trailing_characters);
    return format!("{}", result);
}
