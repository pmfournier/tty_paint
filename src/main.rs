use std::io;
use std::io::prelude::*;
use std::str;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use libc;

mod mouse_event_parser;
use mouse_event_parser::ParseResult;

pub mod console_codes {
    pub const CLEAR: &[u8] = b"\x1b[2J";
    pub const MOUSE_POS_REPORT: &[u8] = b"\x1b[?1002h";
    pub const MOUSE_PIXEL_REPORT: &[u8] = b"\x1b[?1016h";
    pub const HIDE_CURSOR: &[u8] = b"\x1b[?25l";

    pub fn goto(x: usize, y: usize) -> String {
        return format!("\x1b[{};{}H", x, y);
    }
}

fn init_term() {
    {
        let stdin_fd = libc::STDIN_FILENO;
        let termios = Termios::from_fd(stdin_fd).unwrap();
        let mut new_termios = termios.clone();  // make a mutable copy of termios
                                                // that we will modify
        new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
        tcsetattr(stdin_fd, TCSANOW, &mut new_termios).unwrap();
    }

}

const SUBVAL_TO_UTF8: &[&str] = &[
  " ", "▘", "▝", "▀", "▖", "▌", "▞", "▛", "▗", "▚", "▐", "▜", "▄", "▙", "▟", "█",
];


fn main() {
    init_term();
    let mut stdout = io::stdout();
    let mut stdin = io::stdin().lock();

    stdout.write(console_codes::CLEAR).unwrap();
    stdout.write(console_codes::HIDE_CURSOR).unwrap();
    stdout.write(console_codes::MOUSE_POS_REPORT).unwrap();
    stdout.write(console_codes::MOUSE_PIXEL_REPORT).unwrap();
    stdout.flush().unwrap();

    // Arbitrarily use a buffer of 1000 x 1000 pixels
    // Each image_buf[x][y] represents a 2x2 area that will be printed as a single character
    // We only use the lower 4 bits of each u8
    // Each of these 4 bits indicates whether one pixel is active or not.
    // The value is a bit-wise OR of 1 << (x | (y << 1)), for each active pixel, where its coordinates x and y are each 0 or 1
    // This or'ed value is the index in the SUBVAL_TO_UTF8 of the character to display
    let mut image_buf = [[0u8; 1000]; 1000];

    loop {
        let buf = stdin.fill_buf().unwrap();
        let strslice = str::from_utf8(buf).unwrap();

        match mouse_event_parser::try_parse_mouse_event(strslice) {
            ParseResult::Incomplete => (),
            ParseResult::Bad(n) => {
                stdin.consume(n+1);
            }
            ParseResult::Complete(n, mouse_ev) => {
                stdin.consume(n+1);
                if mouse_ev.release == false {
                    // storage is twice the density of a character in each axis
                    let stor_x = (mouse_ev.x / 10) as usize;
                    let stor_y = (mouse_ev.y / 20) as usize;
                    let sub_x = mouse_ev.x & 1;
                    let sub_y = mouse_ev.y & 1;
                    let current_char_pixels = &mut image_buf[stor_x][stor_y];
                    *current_char_pixels |= 1 << (sub_x | (sub_y << 1));
                    stdout.write(format!("{}{}", console_codes::goto(stor_y+1, stor_x+1), SUBVAL_TO_UTF8[*current_char_pixels as usize]).as_bytes()).unwrap();
                    stdout.flush().unwrap();
                }
            }
        }
    }
}
