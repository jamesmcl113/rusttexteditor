use std::io::{Write, Stdout};

use termion;
use termion::raw::{RawTerminal, IntoRawMode};

pub struct Terminal {
    stdout: RawTerminal<Stdout>,
    pub height: usize,
    pub width: usize,
}

impl Terminal {
    pub fn new() -> Terminal {
        let stdout = std::io::stdout().into_raw_mode().unwrap();
        let t_size = termion::terminal_size().unwrap();
        Terminal { stdout, height: t_size.1 as usize, width: t_size.0 as usize }
    }

    pub fn hide_cursor(&mut self) {
        print!("{}", termion::cursor::Hide); 
        self.refresh_screen();
    }

    pub fn show_cursor(&mut self) {
        print!("{}", termion::cursor::Show); 
        self.refresh_screen();
    }

    pub fn refresh_screen(&mut self) {
        self.stdout.flush().unwrap();
    }

    pub fn clear_term_line(&mut self) {
        print!("{}", termion::clear::CurrentLine);
        self.refresh_screen();
    }

    pub fn clear_term(&mut self) {
        print!("{}", termion::clear::All);
        self.refresh_screen();
    }

    pub fn move_cursor_to(&mut self, x: u16, y: u16) {
        print!("{}", termion::cursor::Goto(x, y));
        self.refresh_screen();
    }

}

