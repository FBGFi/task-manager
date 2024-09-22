use std::io;
use clearscreen;
use colored::*;
use terminal_size::{ Width, Height, terminal_size };

struct Dimensions {
    pub width: u16,
    pub height: u16,
}

fn get_terminal_dimensions() -> Dimensions {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        return Dimensions {
            width: w,
            height: h,
        };
    }
    panic!("Terminal not supported");
}

fn clear_and_move_input() {
    let dimensions = get_terminal_dimensions();
    let mut i: u16 = 0;
    while i < dimensions.height {
        println!("");
        i += 1;
    }
    print!("{}", "Give input: ".white().on_truecolor(30, 30, 30));
}

fn main() {
    clearscreen::clear().expect("failed to clear");
    clear_and_move_input();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(n) => {
            println!("{} bytes read", n);
            println!("{}", input);
        }
        Err(error) => println!("error: {error}"),
    }
}
