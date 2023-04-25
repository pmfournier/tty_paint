#[derive(PartialEq, Debug)]
pub enum ParseResult {
    Incomplete,
    Bad(usize),
    Complete(usize, MouseEvent),
}

pub fn try_parse_mouse_event(buf: &str) -> ParseResult {
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
pub struct MouseEvent {
    pub x: u32,
    pub y: u32,
    pub button: u16,
    pub release: bool,
}


