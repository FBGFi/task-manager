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
