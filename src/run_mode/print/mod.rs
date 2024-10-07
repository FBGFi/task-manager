mod resource_header;
mod process_table;

use std::time::Duration;

use crossterm::event::{ poll, read, Event, KeyCode, KeyEvent, KeyEventKind };
use process_table::print_processes;
use resource_header::print_resource_header;
use sysinfo::{ CpuRefreshKind, RefreshKind, System };

use crate::{
    constants::{ CYCLE_WAIT_TIME_MS, PROCESS_HEADERS_LEN },
    state::{ Mode, MODE, SELECTED_COLUMN, SORT_DIRECTION },
    utils::clear_screen_on_dimension_changed,
};

fn refresh_system_usage(sys: &mut System) {
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();
    sys.refresh_cpu_usage();
}

fn read_user_input() {
    if poll(Duration::from_millis(CYCLE_WAIT_TIME_MS)).is_ok_and(|e| { e }) {
        let input = read().unwrap();
        unsafe {
            match input {
                Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                    if SELECTED_COLUMN > 0 {
                        SELECTED_COLUMN -= 1;
                    }
                }
                Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => {
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

pub fn run_print_mode() {
    // TODO declaring this only once might give incorrect cpu information
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything())
    );
    unsafe {
        while MODE == Mode::PRINT {
            clear_screen_on_dimension_changed();
            refresh_system_usage(&mut sys);
            let next_row = print_resource_header(&mut sys, 0);
            print_processes(next_row + 1, &mut sys);
            read_user_input();
            std::thread::sleep(std::time::Duration::from_millis(CYCLE_WAIT_TIME_MS));
        }
    }
}
