use std::{ io::stdout, panic };
use colored::Colorize;
use crossterm::{ cursor, queue };
use machine_info::{ GraphicsUsage, Machine };
use sysinfo::System;

use crate::utils::get_terminal_dimensions;

/// Prints system resource usage to header and returns index of next empty row
pub fn print_resource_header(sys: &mut System, start_row: u16) -> u16 {
    print_memory_usage(start_row, sys);
    print_cpu_usage(start_row + 1, sys);

    let graphs_len = panic::catch_unwind(|| {
        let m = Machine::new();
        let graphics = m.graphics_status();
        print_gpu_usage(start_row + 2, &graphics);
        return graphics.len();
    });

    if graphs_len.is_err() {
        clearscreen::clear().expect("failed to clear");
        return start_row + 2;
    }
    return start_row + 2 + (graphs_len.unwrap() as u16);
}

fn print_cpu_usage(row: u16, sys: &mut System) {
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    print_resource_usage(row, "CPU", sys.global_cpu_usage(), 100.0);
}

fn print_gpu_temp(row: u16, graphics_usage: &GraphicsUsage, gpu_index: usize) {
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    print!("GPU {} temperature: {}°C", gpu_index, graphics_usage.temperature);
}

/// NOTE: machine_info seems to be rather unstable, throwing sometimes on access
fn print_gpu_usage(start_row: u16, graphics: &Vec<GraphicsUsage>) {
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

fn print_memory_usage(row: u16, sys: &mut System) {
    print_resource_usage(row, "Memory", sys.used_memory() as f32, sys.total_memory() as f32);
}

fn print_resource_usage(row: u16, resource: &str, used: f32, total: f32) {
    let resource_usage = used / total;
    let width = get_terminal_dimensions().0;
    let info_text = format!("{} usage: {}%", resource, resource_usage * 100.0);
    let usage_bar_width = ((width as f32) * 0.7).floor() as u16;
    let white_space_count = width - usage_bar_width - (info_text.chars().count() as u16);

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
