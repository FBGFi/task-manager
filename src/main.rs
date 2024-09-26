mod constants;
mod state;
mod process_table;
mod print;
mod utils;
mod resource_header;
mod mode;

use constants::{ PROCESS_HEADERS_LEN, CYCLE_WAIT_TIME_MS };
use print::{ empty_row, print_at_end_of_row, print_on_last_row };
use process_table::print_processes;
use resource_header::print_resource_header;
use state::{ Mode, MODE, SELECTED_COLUMN, SORT_DIRECTION };
use utils::{
    clear_screen_on_dimension_changed,
    get_terminal_dimensions,
    set_current_terminal_dimensions,
};
use std::{ any::Any, io::{ stdin, stdout, Write }, time::Duration };
use clearscreen;
use sysinfo::System;
use crossterm::{ cursor, event::{ poll, read, Event, KeyCode, KeyEvent, KeyEventKind }, queue };

fn refresh_system_usage(sys: &mut System) {
    sys.refresh_all();
    sys.refresh_cpu_usage();
}

fn read_user_input() {
    if poll(Duration::from_millis(CYCLE_WAIT_TIME_MS)).is_ok_and(|e| { e }) {
        let input = read().unwrap();
        unsafe {
            if MODE == Mode::NAVIGATE {
                match input {
                    Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                        if SELECTED_COLUMN > 0 {
                            SELECTED_COLUMN -= 1;
                        }
                    }
                    Event::Key(
                        KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. },
                    ) => {
                        if SELECTED_COLUMN < PROCESS_HEADERS_LEN - 1 {
                            SELECTED_COLUMN += 1;
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Up, kind: KeyEventKind::Press, .. }) => {
                        SORT_DIRECTION = "ASC";
                    }
                    Event::Key(KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, .. }) => {
                        SORT_DIRECTION = "DESC";
                    }
                    _ => (),
                }
            }
            match input {
                Event::Key(
                    KeyEvent { code: KeyCode::Char(':'), kind: KeyEventKind::Press, .. },
                ) => {
                    MODE = Mode::INPUT;
                }
                _ => (),
            }
        }
    }
}

fn enter_input_mode() {
    unsafe {
        let height = get_terminal_dimensions().1;
        empty_row(height);
        print_on_last_row(":");
        queue!(stdout(), cursor::MoveTo(1, height)).unwrap();
        queue!(stdout(), cursor::Show).unwrap();
        stdout().flush().ok().expect("failed to flush");
        let mut input = String::new();
        let mut cursor_position: u16 = 1;
        let mut cleanup_needed = false;
        while MODE == Mode::INPUT {
            queue!(stdout(), cursor::MoveTo(cursor_position, height)).unwrap();
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
                            "n" => {
                                MODE = Mode::NAVIGATE;
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
                                cursor_position = 1;
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
                    if cursor_position > 1 {
                        input.remove((cursor_position - 2) as usize);
                        empty_row(height);
                        cursor_position -= 1;
                        print_on_last_row(format!(":{input}").as_str());
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                    if cursor_position > 1 {
                        cursor_position -= 1;
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => {
                    if cursor_position <= (input.chars().count() as u16) {
                        cursor_position += 1;
                    }
                }
                Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => {
                    if code.to_string().chars().count() == 1 {
                        input.push(code.to_string().chars().nth(0).unwrap());
                        if cleanup_needed {
                            empty_row(height);
                            print_on_last_row(format!(":{input}").as_str());
                        } else {
                            print!("{}", code.to_string());
                        }
                        cursor_position += 1;
                    }
                }
                _ => (),
            }
            stdout().flush().ok().expect("failed to flush");
        }
        empty_row(height);
        queue!(stdout(), cursor::Hide).unwrap();
    }
}

fn main() {
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), cursor::Hide).unwrap();
    set_current_terminal_dimensions();
    unsafe {
        while MODE != Mode::EXIT {
            let mut sys = System::new_all();
            match MODE {
                Mode::PRINT | Mode::NAVIGATE => {
                    clear_screen_on_dimension_changed();
                    refresh_system_usage(&mut sys);
                    let next_row = print_resource_header(&mut sys, 0);
                    print_processes(next_row + 1, &mut sys);
                }
                Mode::INPUT => {
                    enter_input_mode();
                }
                Mode::SEARCH => {
                    panic!("Not implemented yet");
                }
                Mode::EXIT => (),
            }
            read_user_input();
            std::thread::sleep(std::time::Duration::from_millis(CYCLE_WAIT_TIME_MS));
        }
    }
    clearscreen::clear().expect("failed to clear");
}
