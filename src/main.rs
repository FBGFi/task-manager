use std::io::{ self, stdout };
use clearscreen;
use colored::*;
use terminal_size::{ Width, Height, terminal_size };
use sysinfo::System;
use crossterm::{ queue };

fn get_terminal_dimensions() -> (u16, u16) {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        return (w, h);
    }
    panic!("Terminal not supported");
}

fn clear_and_move_input() {
    let (width, height) = get_terminal_dimensions();
    let mut i: u16 = 0;
    while i < height {
        println!("");
        i += 1;
    }
    print!("{}", "Give input: ".white().on_truecolor(30, 30, 30));
}

fn print_memory_usage() {
    let sys = System::new_all();
    let total_memory = sys.total_memory() as f32;
    let used_memory = sys.used_memory() as f32;
    let memory_usage = used_memory / total_memory;
    let (width, height) = get_terminal_dimensions();
    let info_text = format!("Memory usage: {}%", memory_usage * 100.0);
    let usage_bar_width = ((width as f32) * 0.7).floor() as u16;
    let white_space_count = width - usage_bar_width - (info_text.len() as u16);

    print!("\r");

    print!("{}", info_text);

    for _ in 0..white_space_count {
        print!(" ");
    }

    print!("[");
    for i in 0..usage_bar_width - 2 {
        if (i as f32) / (usage_bar_width as f32) < memory_usage {
            print!("{}", "-".green());
        } else {
            print!("{}", "-".red());
        }
    }
    print!("]");
}

fn main() {
    // clear_and_move_input();
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), crossterm::cursor::Hide).unwrap();
    while true {
        print_memory_usage();
        std::thread::sleep(std::time::Duration::from_millis(500));
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
