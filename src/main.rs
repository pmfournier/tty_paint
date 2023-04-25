use std::io;
use std::io::prelude::*;
use std::str;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use libc;

pub mod console_codes {
    pub const CLEAR: &str = "\x1b[2J";
    pub const MOUSE_POS_REPORT: &str = "\x1b[?1002h";
    pub const MOUSE_PIXEL_REPORT: &str = "\x1b[?1016h";
    pub const HIDE_CURSOR: &str = "\x1b[?25l";

    pub fn goto(x: usize, y: usize) -> String {
        return format!("\x1b[{};{}H", x, y);
    }
}

#[derive(PartialEq, Debug)]
enum ParseResult {
    Incomplete,
    Bad(usize),
    Complete(usize, MouseEvent),
}

fn try_parse_mouse_event(buf: &str) -> ParseResult {
    let mut it = buf.char_indices().peekable();

    match it.next() {
        None => return ParseResult::Incomplete,
        Some((_, '\x1b')) => {},
        Some((i, _)) => return ParseResult::Bad(i)
    };

    match it.next() {
        None => return ParseResult::Incomplete,
        Some((_, '[')) => {},
        Some((i, _)) => return ParseResult::Bad(i)
    };

    let but_begin;
    match it.next() {
        None => return ParseResult::Incomplete,
        Some((i, '<')) => { but_begin = i+1; },
        Some((i, _)) => return ParseResult::Bad(i)
    };

    let but: u16;
    loop {
        match it.next() {
            None => return ParseResult::Incomplete,
            Some((i, '0'..='9')) => {
                if let Some((_, '0'..='9')) = it.peek() {
                } else {
                    but = buf[but_begin..i+1].parse().unwrap();// FIXME
                    break;
                }
            }
            Some((i, _)) => return ParseResult::Bad(i)
        }
    };

    let x_begin;
    match it.next() {
        None => return ParseResult::Incomplete,
        Some((i, ';')) => { x_begin = i+1; },
        Some((i, _)) => return ParseResult::Bad(i)
    };

    let x: u32;
    loop {
        match it.next() {
            None => return ParseResult::Incomplete,
            Some((i, '0'..='9')) => {
                if let Some((_, '0'..='9')) = it.peek() {
                } else {
                    x = buf[x_begin..i+1].parse().unwrap();// FIXME
                    break;
                }
            }
            Some((i, _)) => return ParseResult::Bad(i)
        }
    };

    let y_begin;
    match it.next() {
        None => return ParseResult::Incomplete,
        Some((i, ';')) => { y_begin = i+1; },
        Some((i, _)) => return ParseResult::Bad(i)
    };

    let y: u32;
    loop {
        match it.next() {
            None => return ParseResult::Incomplete,
            Some((i, '0'..='9')) => {
                if let Some((_, '0'..='9')) = it.peek() {
                } else {
                    y = buf[y_begin..i+1].parse().unwrap();// FIXME
                    break;
                }
            }
            Some((i, _)) => return ParseResult::Bad(i)
        }
    };

    let release: bool;
    let last_off: usize;
    match it.next() {
        None => return ParseResult::Incomplete,
        Some((i, 'M')) => { last_off = i; release = false; },
        Some((i, 'm')) => { last_off = i; release = true; },
        Some((i, _)) => return ParseResult::Bad(i)
    };

    ParseResult::Complete(last_off, MouseEvent { x, y, button: but, release })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tst() {
        assert_eq!(try_parse_mouse_event(""), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("a"), ParseResult::Bad(0));
        assert_eq!(try_parse_mouse_event("\x1b"), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("\x1ba"), ParseResult::Bad(1));
        assert_eq!(try_parse_mouse_event("a\x1b"), ParseResult::Bad(0));

        assert_eq!(try_parse_mouse_event("\x1b["), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("\x1b]"), ParseResult::Bad(1));

        assert_eq!(try_parse_mouse_event("\x1b[<"), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("\x1b[<f"), ParseResult::Bad(3));

        assert_eq!(try_parse_mouse_event("\x1b[<5"), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("\x1b[<Q"), ParseResult::Bad(3));
        assert_eq!(try_parse_mouse_event("\x1b[<;"), ParseResult::Bad(3));

        assert_eq!(try_parse_mouse_event("\x1b[<53"), ParseResult::Incomplete);

        // problems below
        assert_eq!(try_parse_mouse_event("\x1b[<5;"), ParseResult::Incomplete);
        assert_eq!(try_parse_mouse_event("\x1b[<53;"), ParseResult::Incomplete);

        assert_eq!(try_parse_mouse_event("\x1b[<53;12;23M"), ParseResult::Complete(11, MouseEvent { x: 12, y: 23, button: 53, release: false }));
        assert_eq!(try_parse_mouse_event("\x1b[<53;12;23m"), ParseResult::Complete(11, MouseEvent { x: 12, y: 23, button: 53, release: true }));
        assert_eq!(try_parse_mouse_event("\x1b[<53;12;23a"), ParseResult::Bad(11));
    }
}

#[derive(PartialEq, Debug)]
struct MouseEvent {
    x: u32,
    y: u32,
    button: u16,
    release: bool,
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
  " ", // 0

  "▘", // 1

  "▝",

  "▀",

  "▖",

  "▌",

  "▞",

  "▛",

  "▗",

  "▚",

  "▐",

  "▜",

  "▄",

  "▙",

  "▟",

  "█",
];


fn main() {
    init_term();
    let mut stdout = io::stdout();

    stdout.write(format!("{}{}{}{}", console_codes::CLEAR, console_codes::HIDE_CURSOR, console_codes::MOUSE_POS_REPORT, console_codes::MOUSE_PIXEL_REPORT).as_bytes()).unwrap();
    stdout.flush().unwrap();

    let mut image_buf = [[0u8; 1000]; 1000];
    let mut stdin = io::stdin().lock();
    loop {
        let buf = stdin.fill_buf().unwrap();

        let strslice = str::from_utf8(buf).unwrap();
        match try_parse_mouse_event(strslice) {
            ParseResult::Incomplete => (),
            ParseResult::Bad(n) => {
                stdin.consume(n+1);
            }
            ParseResult::Complete(n, mouse_ev) => {
                stdin.consume(n+1);
                if mouse_ev.release == false {
                    // storage is twice the density of a character in each axis
                    let stor_x = (mouse_ev.x / 5) as usize;
                    let stor_y = (mouse_ev.y / 10) as usize;
                    let char_x = stor_x >> 1;
                    let char_y = stor_y >> 1;
                    let sub_x = mouse_ev.x & 1;
                    let sub_y = mouse_ev.y & 1;
                    let current_char_pixels = &mut image_buf[stor_x][stor_y];
                    *current_char_pixels |= (1 << (sub_x | (sub_y << 1)));
                    stdout.write(format!("{}{}", console_codes::goto(char_y, char_x), SUBVAL_TO_UTF8[*current_char_pixels as usize]).as_bytes()).unwrap();
                    stdout.flush().unwrap();
                }
            }
        }
    }
}
