extern crate std;

use std::print;

pub fn draw_point(_x: i64, _y: i64, _c: u32) -> u64 {
    // We don't support GUI for Linux targets as it will only be used for unit testing
    0
}

pub fn write_string(s: &str) -> u64 {
    print!("{s}");
    s.len() as u64
}
