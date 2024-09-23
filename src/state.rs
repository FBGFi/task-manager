pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

pub static mut SELECTED_COLUMN: usize = 0;
pub static mut SORT_DIRECTION: &str = "ASC";
pub static mut PREVIOUS_DIMENSIONS: Dimensions = Dimensions {
    width: 0,
    height: 0,
};
