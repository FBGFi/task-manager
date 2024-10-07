use std::io::stdout;

use crossterm::{ cursor, event::{ read, Event, KeyCode, KeyEvent, KeyEventKind }, queue };

use crate::{
    state::{ Mode, MODE, SEARCH_TEXT },
    utils::{
        delete_previous_character,
        empty_row,
        enter_input_mode,
        get_terminal_dimensions,
        navigate_left_input,
        navigate_right_input,
        print_input,
    },
};

pub fn run_search_mode() {
    let height = get_terminal_dimensions().1;
    empty_row(height);
    unsafe {
        let input_prefix = "Search: ";
        let cursor_start_position = enter_input_mode(&input_prefix);
        let mut input = format!("{}", SEARCH_TEXT);
        let mut cursor_position: u16 = cursor_start_position + (input.chars().count() as u16);
        while MODE == Mode::SEARCH {
            match read().unwrap() {
                Event::Key(KeyEvent { code: KeyCode::Esc, kind: KeyEventKind::Press, .. }) => {
                    MODE = Mode::PRINT;
                }
                Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) => {
                    SEARCH_TEXT = format!("{}", input);
                    MODE = Mode::PRINT;
                }
                Event::Key(
                    KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. },
                ) => {
                    cursor_position = delete_previous_character(
                        input_prefix,
                        &mut input,
                        height,
                        cursor_position,
                        cursor_start_position
                    );
                }
                Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                    cursor_position = navigate_left_input(
                        height,
                        cursor_position,
                        cursor_start_position
                    );
                }
                Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => {
                    cursor_position = navigate_right_input(
                        &input,
                        height,
                        cursor_position,
                        cursor_start_position
                    );
                }
                Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => {
                    cursor_position = print_input(
                        &mut input,
                        code,
                        height,
                        cursor_position,
                        cursor_start_position
                    );
                }
                _ => (),
            }
        }
    }
    empty_row(height);
    queue!(stdout(), cursor::Hide).unwrap();
}
