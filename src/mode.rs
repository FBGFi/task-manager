use crate::state::{ Mode, MODE };

pub fn switch_mode(mode: Mode) {
    unsafe {
        MODE = mode;
    }
}
