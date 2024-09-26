pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

#[derive(PartialEq)]
pub enum Mode {
    PRINT,
    INPUT,
    EXIT,
    NAVIGATE,
    SEARCH,
}

pub static mut SELECTED_COLUMN: usize = 0;
pub static mut SORT_DIRECTION: &str = "ASC";
pub static mut PREVIOUS_DIMENSIONS: Dimensions = Dimensions {
    width: 0,
    height: 0,
};
pub static mut MODE: Mode = Mode::PRINT;
pub static mut SEARCH_TEXT: String = String::new();
