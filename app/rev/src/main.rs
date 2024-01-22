#![no_std]
#![cfg_attr(not(target_os = "linux"), no_main)]

extern crate alloc;
use alloc::string::String;
use noli::entry_point;
use noli::error::Result;
use noli::print;
use noli::println;

/// Reverses the input string on a row.
/// Input q and enter to exit
fn main() -> Result<()> {
    println!("rev~");
    let mut line = String::new();
    loop {
        if let Some(c) = char::from_u32(noli::sys::read_key() as u32) {
            print!("{c}");
            if c == 'q' {
                break;
            } else if c == '\n' {
                println!("rev!!");
                let reversed = line.chars().rev().collect::<String>();
                println!(">>> '{line}'");
                println!(">>> '{reversed}'");
                line.clear();
            } else {
                println!("\npush!");
                line.push('h');
            }
        }
    }
    Ok(())
}

entry_point!(main);
