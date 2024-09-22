use std::{ io::{ self, stdout }, panic };
use clearscreen;
use colored::*;
use machine_info::{ GraphicsUsage, Machine };
use terminal_size::{ Width, Height, terminal_size };
use sysinfo::{ System };
use crossterm::{ queue, cursor };

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

fn print_resource_usage(row: u16, resource: &str, used: f32, total: f32) {
    let resource_usage = used / total;
    let width = get_terminal_dimensions().0;
    let info_text = format!("{} usage: {}%", resource, resource_usage * 100.0);
    let usage_bar_width = ((width as f32) * 0.7).floor() as u16;
    let white_space_count = width - usage_bar_width - (info_text.len() as u16);

    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();

    print!("{}", info_text);

    for _ in 0..white_space_count {
        print!(" ");
    }

    print!("[");
    for i in 0..usage_bar_width - 2 {
        if (i as f32) / (usage_bar_width as f32) < resource_usage {
            print!("{}", "|".green());
        } else {
            print!("{}", "|".red());
        }
    }
    print!("]");
}

fn refresh_system_usage(sys: &mut System) {
    sys.refresh_all();
    sys.refresh_cpu_usage();
}

fn print_memory_usage(row: u16, sys: &mut System) {
    print_resource_usage(row, "Memory", sys.used_memory() as f32, sys.total_memory() as f32);
}

/// NOTE: atleast on my system Windows task manager returns drastically different values for cpu usage
fn print_cpu_usage(row: u16, sys: &mut System) {
    let cpus = sys.cpus();

    // Print each cpu separately
    // for i in 0..cpus.len() {
    //     let cpu = &cpus[i];
    //     print_resource_usage(
    //         (i as u16) + 1,
    //         format!("CPU{}", i + 1).as_str(),
    //         cpu.cpu_usage(),
    //         100.0
    //     );
    // }

    // Print total cpu usage
    let mut total_used: f32 = 0.0;
    for cpu in cpus {
        total_used += cpu.cpu_usage();
    }
    print_resource_usage(row, "CPU", total_used, 100.0 * (cpus.len() as f32));
}

fn print_gpu_temp(row: u16, graphics_usage: &GraphicsUsage, gpu_index: usize) {
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    print!("GPU {} temperature: {}°C", gpu_index, graphics_usage.temperature);
}

/// NOTE: machine_info seems to be rather unstable, throwing sometimes on access
fn print_gpu_usage(start_row: u16, graphics: Vec<GraphicsUsage>) {
    for i in 0..graphics.len() {
        let graphics_usage = &graphics[i];
        print_resource_usage(
            start_row + (i as u16),
            format!("GPU {}", i).as_str(),
            graphics_usage.gpu as f32,
            100.0
        );
        print_gpu_temp(start_row + (i as u16) + 1, graphics_usage, i);
    }
}

fn main() {
    // clear_and_move_input();
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), cursor::Hide).unwrap();
    let mut sys = System::new_all();
    let mut panics: u16 = 0;
    loop {
        refresh_system_usage(&mut sys);
        print_memory_usage(0, &mut sys);
        print_cpu_usage(1, &mut sys);

        let result = panic::catch_unwind(|| {
            let m = Machine::new();
            print_gpu_usage(2, m.graphics_status());
        });

        if result.is_err() {
            clearscreen::clear().expect("failed to clear");
            panics += 1;
            let height = get_terminal_dimensions().1;
            queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
            print!("Machine panicked {} times", panics);
        }

        std::thread::sleep(std::time::Duration::from_millis(200));
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
