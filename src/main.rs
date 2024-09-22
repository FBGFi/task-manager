use std::{ any::Any, collections::HashMap, io::{ self, stdout, Read }, panic, time::Duration };
use clearscreen;
use machine_info::{ GraphicsUsage, Machine };
use terminal_size::{ Width, Height, terminal_size };
use sysinfo::{ Pid, Process, System };
use crossterm::{
    cursor,
    event::{ poll, read, Event, KeyCode, KeyEvent, KeyEventKind },
    queue,
    style::Stylize,
};

const HEADERS: [&str; 5] = ["PID", "Name", "CPU (%)", "Memory (KB)", "Run time (s)"];
const HEADERS_LEN: usize = HEADERS.len();
const WAIT_TIME_MS: u64 = 200;

static mut SELECTED_COLUMN: usize = 0;
static mut SORT_DIRECTION: &str = "ASC";

fn get_terminal_dimensions() -> (u16, u16) {
    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        return (w, h);
    }
    panic!("Terminal not supported");
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

fn print_column(row: u16, col_index: usize, col_width: u16, cols_length: usize, text: &str) {
    let col = col_width * (col_index as u16);
    queue!(stdout(), cursor::MoveTo(col, row)).unwrap();
    let is_selected = get_is_selected(col_index);
    // TODO this will throw if column width is shorter than given text
    let col_start = format!(
        "{}{}",
        " ".on_white(),
        format_selected_color(format!(" {}", text).as_str(), is_selected)
    );
    // Checking length of color formatted text does not work, since it is ANSII encoded
    let col_print_len = format!("{}{}", " ", format!(" {}", text)).chars().count();
    print!("{}", col_start);

    let mut white_spaces = col_width - (col_print_len as u16);
    let is_last = col_index == cols_length - 1;
    let width = get_terminal_dimensions().0;

    if is_last {
        white_spaces = width - col - (col_print_len as u16) - 1;
    }

    for _ in 0..white_spaces {
        print!("{}", format_selected_color(" ", is_selected));
    }

    if is_last {
        queue!(stdout(), cursor::MoveTo(width, row)).unwrap();
        print!("{}", " ".on_white());
    }
}

fn print_row_separator(row: u16) {
    let width = get_terminal_dimensions().0;
    queue!(stdout(), cursor::MoveTo(0, row)).unwrap();
    for _ in 0..width {
        print!("{}", " ".on_white());
    }
}

fn print_on_last_row(text: &str) {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    print!("{}", text);
}

fn get_sorted_processes(sys: &mut System) -> Vec<(&Pid, &Process)> {
    unsafe {
        let mut vec: Vec<_> = sys.processes().iter().collect();
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

fn print_processes(start_row: u16, sys: &mut System) {
    let (width, height) = get_terminal_dimensions();
    let empty_before: u16 = 1;

    let top_border = start_row + empty_before;
    print_row_separator(top_border);

    let header_row = top_border + 1;
    queue!(stdout(), cursor::MoveTo(0, header_row)).unwrap();
    let col_width = ((width as f32) / (HEADERS_LEN as f32)).floor() as u16;

    // TODO this should not be rendered on every cycle, move to only be printed on first cycle
    for i in 0..HEADERS_LEN {
        let header = HEADERS[i];
        print_column(header_row, i, col_width, HEADERS_LEN, header);
    }

    print_row_separator(header_row + 1);

    let max_print_count = height - header_row - 4;
    let mut i: u16 = 0;
    let processes = get_sorted_processes(sys);
    for (pid, process) in processes {
        if i >= max_print_count {
            break;
        }
        let row = header_row + 2 + i;
        print_column(row, 0, col_width, HEADERS_LEN, format!("{}", pid.as_u32()).as_str());
        print_column(row, 1, col_width, HEADERS_LEN, format!("{:?}", process.name()).as_str());
        print_column(row, 2, col_width, HEADERS_LEN, format!("{}", process.cpu_usage()).as_str());
        print_column(
            row,
            3,
            col_width,
            HEADERS_LEN,
            format!("{}", (process.memory() as f32) / 1000.0).as_str()
        );
        print_column(row, 4, col_width, HEADERS_LEN, format!("{}", process.run_time()).as_str());
        i += 1;
    }

    print_row_separator(height - 2);
}

fn print_user_input() {
    let height = get_terminal_dimensions().1;
    queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
    let input = read().unwrap();
    print!("{:?}, {:?}", input.type_id(), input);
}

fn read_user_input() {
    if poll(Duration::from_millis(WAIT_TIME_MS)).is_ok_and(|e| { e }) {
        let input = read().unwrap();

        match input {
            Event::Key(KeyEvent { code: KeyCode::Left, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    if SELECTED_COLUMN > 0 {
                        SELECTED_COLUMN -= 1;
                    }
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    if SELECTED_COLUMN < HEADERS_LEN - 1 {
                        SELECTED_COLUMN += 1;
                    }
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Up, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    SORT_DIRECTION = "ASC";
                }
            }
            Event::Key(KeyEvent { code: KeyCode::Down, kind: KeyEventKind::Press, .. }) => {
                unsafe {
                    SORT_DIRECTION = "DESC";
                }
            }
            _ => (),
        }
        // print_user_input();
    }
}

fn main() {
    clearscreen::clear().expect("failed to clear");
    queue!(stdout(), cursor::Hide).unwrap();
    let mut sys = System::new_all();
    let mut panics: u16 = 0;

    loop {
        read_user_input();
        refresh_system_usage(&mut sys);
        print_memory_usage(0, &mut sys);
        print_cpu_usage(1, &mut sys);

        let graphs_len: Result<usize, usize> = panic
            ::catch_unwind(|| {
                let m = Machine::new();
                let graphics = m.graphics_status();
                print_gpu_usage(2, &graphics);
                Ok(graphics.len())
            })
            .unwrap();

        if graphs_len.is_err() {
            clearscreen::clear().expect("failed to clear");
            panics += 1;
            let height = get_terminal_dimensions().1;
            queue!(stdout(), cursor::MoveTo(0, height)).unwrap();
            print!("Machine panicked {} times", panics);
        } else {
            let len = graphs_len.unwrap() as u16;
            print_processes(3 + len, &mut sys);
        }

        std::thread::sleep(std::time::Duration::from_millis(WAIT_TIME_MS));
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
