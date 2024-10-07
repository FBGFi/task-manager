use std::io::{ stdout, Write };

use crossterm::{ cursor, event::{ read, Event, KeyCode, KeyEvent, KeyEventKind }, queue };

use crate::{
    state::{ Mode, MODE },
    utils::{
        delete_previous_character,
        empty_row,
        navigate_left_input,
        navigate_right_input,
        print_at_end_of_row,
        print_input,
        print_on_last_row,
        enter_input_mode,
        get_terminal_dimensions,
    },
};

pub fn run_input_mode() {
    let height = get_terminal_dimensions().1;
    empty_row(height);
    unsafe {
        let input_prefix = ":";
        let cursor_start_position = enter_input_mode(&input_prefix);
        let mut input = String::new();
        let mut cursor_position: u16 = cursor_start_position;
        let mut cleanup_needed = false;
        while MODE == Mode::INPUT {
            queue!(stdout(), cursor::MoveTo(cursor_position, height)).unwrap();
            stdout().flush().ok().expect("failed to flush");
            match read().unwrap() {
                Event::Key(KeyEvent { code: KeyCode::Esc, kind: KeyEventKind::Press, .. }) => {
                    MODE = Mode::PRINT;
                }
                Event::Key(KeyEvent { code: KeyCode::Enter, kind: KeyEventKind::Press, .. }) => {
                    if input.chars().count() > 0 {
                        match input.as_str() {
                            "p" => {
                                MODE = Mode::PRINT;
                            }
                            "q" => {
                                MODE = Mode::EXIT;
                            }
                            "s" => {
                                MODE = Mode::SEARCH;
                            }
                            "h" => {
                                clearscreen::clear().expect("failed to clear");
                                queue!(stdout(), cursor::MoveTo(0, 0)).unwrap();
                                println!("Accepted commands are:\n");
                                println!("p - Print running process information");
                                println!("h - Help");
                                println!("n - Navigate between columns");
                                println!(
                                    "s - Enter search mode for filtering processes via selected column"
                                );
                                println!("q - Exit program");
                                cleanup_needed = true;
                                input = String::new();
                                cursor_position = cursor_start_position;
                            }
                            _ => {
                                print_at_end_of_row(
                                    "Error: Incorrect input, type 'h' for help",
                                    height
                                );
                                cleanup_needed = true;
                            }
                        }
                    }
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
                    if cleanup_needed {
                        empty_row(height);
                        print_on_last_row(format!(":{input}").as_str());
                        cleanup_needed = false;
                    }
                }
                _ => (),
            }
        }
    }
    empty_row(height);
    queue!(stdout(), cursor::Hide).unwrap();
}
