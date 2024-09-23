mod constants;
mod state;
mod process_table;
mod print;
mod utils;
mod resource_header;

use constants::{ PROCESS_HEADERS_LEN, CYCLE_WAIT_TIME_MS };
use process_table::print_processes;
use resource_header::print_resource_header;
use state::{ SELECTED_COLUMN, SORT_DIRECTION };
use utils::{
    clear_screen_on_dimension_changed,
    get_terminal_dimensions,
    set_current_terminal_dimensions,
};
use std::{ any::Any, io::stdout, time::Duration };
use clearscreen;
use sysinfo::System;
use crossterm::{ cursor, event::{ poll, read, Event, KeyCode, KeyEvent, KeyEventKind }, queue };

fn refresh_system_usage(sys: &mut System) {
    sys.refresh_all();
    sys.refresh_cpu_usage();
}

// TODO only used for debugging, remove
fn print_user_input() {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    let input = read().unwrap();
    print!("{:?}, {:?}", input.type_id(), input);
}

fn read_user_input() {
    if poll(Duration::from_millis(CYCLE_WAIT_TIME_MS)).is_ok_and(|e| { e }) {
        let input = read().unwrap();

        match input {
            Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    if SELECTED_COLUMN > 0 {
                        SELECTED_COLUMN -= 1;
                    }
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    if SELECTED_COLUMN < PROCESS_HEADERS_LEN - 1 {
                        SELECTED_COLUMN += 1;
                    }
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Up, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    SORT_DIRECTION = "ASC";
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    SORT_DIRECTION = "DESC";
                }
            }
            _ => (),
        }
        // print_user_input();
    }
}

fn main() {
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), cursor::Hide).unwrap();
    let mut sys = System::new_all();
    set_current_terminal_dimensions();

    loop {
        read_user_input();
        clear_screen_on_dimension_changed();
        refresh_system_usage(&mut sys);
        let next_row = print_resource_header(&mut sys, 0);
        print_processes(next_row, &mut sys);

        std::thread::sleep(std::time::Duration::from_millis(CYCLE_WAIT_TIME_MS));
        // let mut input = String::new();
        // match io::stdin().read_line(&mut input) {
        //     Ok(n) => {
        //         println!("{} bytes read", n);
        //         println!("{}", input);
        //     }
        //     Err(error) => println!("error: {error}"),
        // }
    }
}
