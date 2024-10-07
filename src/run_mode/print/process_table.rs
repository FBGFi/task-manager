use std::io::stdout;
use colored::Colorize;
use crossterm::{ cursor, queue };
use regex::Regex;
use sysinfo::{ Pid, Process, System };

use crate::{
    constants::{ PROCESS_HEADERS, PROCESS_HEADERS_LEN },
    state::{ SEARCH_TEXT, SELECTED_COLUMN, SORT_DIRECTION },
    utils::{ strip_closing_quotes, truncate_text, get_terminal_dimensions },
};

pub fn print_processes(start_row: u16, sys: &mut System) {
    let (width, height) = get_terminal_dimensions();
    let empty_before: u16 = 1;

    let top_border = start_row + empty_before;
    print_row_separator(top_border);

    let header_row = top_border + 1;
    queue!(stdout(), cursor::MoveTo(0, header_row)).unwrap();
    let col_width = ((width as f32) / (PROCESS_HEADERS_LEN as f32)).floor() as u16;

    // TODO this should not be rendered on every cycle, move to only be printed on first cycle
    for i in 0..PROCESS_HEADERS_LEN {
        let header = PROCESS_HEADERS[i];
        print_column(header_row, i, col_width, PROCESS_HEADERS_LEN, header);
    }

    print_row_separator(header_row + 1);

    let max_print_count = height - header_row - 4;
    let processes = get_sorted_processes(sys);
    for i in 0..max_print_count {
        if i >= max_print_count {
            break;
        }

        let mut process_id = String::new();
        let mut process_name = String::new();
        let mut cpu_usage = String::new();
        let mut memory = String::new();
        let mut run_time = String::new();

        if i < (processes.len() as u16) {
            let (pid, process) = processes[i as usize];
            process_id = format!("{}", pid.as_u32());
            process_name = format!("{:?}", process.name());
            cpu_usage = format!("{}", process.cpu_usage());
            memory = format!("{}", (process.memory() as f32) / 1000.0);
            run_time = format!("{}", process.run_time());
        }

        let row = header_row + 2 + i;
        print_column(row, 0, col_width, PROCESS_HEADERS_LEN, process_id.as_str());
        print_column(row, 1, col_width, PROCESS_HEADERS_LEN, process_name.as_str());
        print_column(row, 2, col_width, PROCESS_HEADERS_LEN, cpu_usage.as_str());
        print_column(row, 3, col_width, PROCESS_HEADERS_LEN, memory.as_str());
        print_column(row, 4, col_width, PROCESS_HEADERS_LEN, run_time.as_str());
    }

    print_row_separator(height - 2);
}

fn print_row_separator(row: u16) {
    let width = get_terminal_dimensions().0;
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    for _ in 0..width {
        print!("{}", " ".on_white());
    }
}

fn print_column(row: u16, col_index: usize, col_width: u16, cols_length: usize, text: &str) {
    let col = col_width * (col_index as u16);
    queue!(stdout(), cursor::MoveTo(col, row)).unwrap();
    let is_selected = get_is_selected(col_index);
    let column_separator = " ";
    let mut stripped_text = strip_closing_quotes(text);
    // Checking length of color formatted text does not work, since it is ANSII encoded
    let mut col_print_len = format!("{}{}", column_separator, stripped_text).chars().count() as u16;
    if col_print_len > col_width - 1 {
        stripped_text = truncate_text(stripped_text.as_str(), col_width - 4);
        col_print_len = format!("{}{}", column_separator, stripped_text).chars().count() as u16;
    }
    let col_start = format!(
        "{}{}",
        " ".on_white(),
        format_selected_color(format!(" {}", stripped_text).as_str(), is_selected)
    );
    print!("{}", col_start);

    let mut white_spaces = col_width - col_print_len;
    let is_last = col_index == cols_length - 1;
    let width = get_terminal_dimensions().0;

    if is_last {
        white_spaces = width - col - col_print_len - 1;
    }

    for _ in 0..white_spaces {
        print!("{}", format_selected_color(" ", is_selected));
    }

    if is_last {
        queue!(stdout(), cursor::MoveTo(width, row)).unwrap();
        print!("{}", " ".on_white());
    }
}

fn format_selected_color(text: &str, is_selected: bool) -> String {
    if !is_selected {
        return format!("{}", text);
    }
    return format!("{}", text.on_blue());
}

fn get_is_selected(col_index: usize) -> bool {
    unsafe {
        return col_index == SELECTED_COLUMN;
    }
}

fn get_sorted_processes(sys: &mut System) -> Vec<(&Pid, &Process)> {
    unsafe {
        let mut vec: Vec<_> = sys
            .processes()
            .iter()
            .filter(|p| {
                if SEARCH_TEXT.chars().count() == 0 {
                    return true;
                }
                let re = Regex::new(SEARCH_TEXT.as_str()).unwrap();
                return match SELECTED_COLUMN {
                    1 => re.is_match(p.1.name().to_str().unwrap()),
                    2 => re.is_match(p.1.cpu_usage().to_string().as_str()),
                    3 => re.is_match(p.1.memory().to_string().as_str()),
                    4 => re.is_match(p.1.run_time().to_string().as_str()),
                    _ => re.is_match(p.0.as_u32().to_string().as_str()),
                };
            })
            .collect();
        vec.sort_by(|a, b| {
            let comp = match SELECTED_COLUMN {
                1 => a.1.name().partial_cmp(b.1.name()).unwrap(),
                2 => a.1.cpu_usage().partial_cmp(&b.1.cpu_usage()).unwrap(),
                3 => a.1.memory().partial_cmp(&b.1.memory()).unwrap(),
                4 => a.1.run_time().partial_cmp(&b.1.run_time()).unwrap(),
                // TODO this does not sort for PID?
                _ => a.0.as_u32().partial_cmp(&a.0.as_u32()).unwrap(),
            };
            if SORT_DIRECTION == "ASC" {
                return comp.reverse();
            }
            return comp;
        });
        return vec;
    }
}
