mod print;
mod input;
mod search;

use crate::state::{ Mode, MODE };

pub fn run() {
    unsafe {
        match MODE {
            Mode::PRINT => {
                print::run_print_mode();
            }
            Mode::INPUT => {
                input::run_input_mode();
            }
            Mode::SEARCH => {
                search::run_search_mode();
            }
            Mode::EXIT => (),
        }
    }
}
