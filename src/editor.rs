extern crate termion;
extern crate chrono;

use chrono::Timelike;

use termion::input::TermRead;
use std::io;
use termion::event::Key;
use termion::{color, style};

use crate::terminal::Terminal;
use crate::document::{Document, Erow, read_into_document};

pub struct Editor {
    x: usize,
    y: usize,
    rowoff: usize,
    coloff: usize,
    quit_now: bool,
    rows: Vec<Erow>,
    doc: Option<Document>,
    term: Terminal,
    status_message: String,
}

impl Editor {
    pub fn new() -> Editor {
        let term = Terminal::new();
        let arg = std::env::args().nth(1);

        let rows = match arg {
            Some(filename) => read_into_document(filename.as_str()),
            None => (Vec::new(), None),
        };
        
        Editor { x: 1, y: 1, rowoff: 0, coloff: 0, quit_now: false, rows: rows.0, doc: rows.1, term, status_message: String::from("Welcome to Text Editor! CTRL-Q to quit.") }
    }

    pub fn run(&mut self) {
        self.term.clear_term();

        loop {
            if self.quit_now { break; }
            self.draw_rows();
            if let Err(e) = self.handle_keypress() { Editor::die(e, "Couldn't get keypress!"); }
        }

        self.term.clear_term();
        self.term.move_cursor_to(1, 1);
    }
    
    ///*** input ***///
    fn move_cursor(&mut self, key: Key) -> Result<(), std::io::Error> {

        match key {
            Key::Right => self.x = self.x.saturating_add(1),
            Key::Left => self.x = self.x.saturating_sub(1),
            Key::Down => self.y = self.y.saturating_add(1),
            Key::Up => self.y = self.y.saturating_sub(1),
            _ => (),
        }
        
        // vertical scrolling
        let max_scroll_h = match self.rows.len() {
            0 => 1,
            _ => std::cmp::min(self.term.height - 1, self.rows.len()),
        };
        
        if self.y < 1 {
            self.y = 1;
            if self.rowoff > 0 {
                self.rowoff -= 1;
            }    
        }
        else if self.y > max_scroll_h {
            self.y = max_scroll_h;
            if self.rows.len() > self.get_filerow() {
                self.rowoff += 1;
            }
        }

        // horizontal scrolling
        let max_scroll_w = match self.rows.len() {
            0 => 1,
            _ => std::cmp::min(self.term.width, self.rows[self.get_filerow() - 1].data.len() + 1),
        }; 

        if self.x < 1 { self.x = 1 }
        else if self.x > max_scroll_w { self.x = max_scroll_w }

        self.term.move_cursor_to(self.x as u16, self.y as u16); 

        Ok(())
    }

    fn get_keypress(&self) -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            } 
        }
    }

    fn handle_keypress(&mut self) -> Result<(), std::io::Error> {
        let key = self.get_keypress()?;

        match key {
            Key::Ctrl('q') => self.quit_now = true,
            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(key)?,
            Key::Char(c) => { 
                if c == '\n' {
                    self.insert_row(self.get_filerow());
                    let slc = self.rows[self.get_filerow() - 1].data.clone();
                    self.append_str_to_row(self.get_filerow(), &slc.as_str()[self.x - 1..]);
                    self.clear_row_to_end(self.get_filerow() - 1, self.x - 1);
                    self.move_cursor(Key::Down)?;
                    self.x = 1;
                } else {
                    self.row_insert_char(self.get_filerow() - 1, self.x - 1, c);
                    self.move_cursor(Key::Right)?;
                }
            },
            Key::Backspace => {
                if self.get_filerow() > 1 && self.x == 1 {
                    let orig_len = self.rows[self.get_filerow() - 2].data.len();
                    self.append_str_to_row(self.get_filerow() - 2, self.rows[self.get_filerow() - 1].data.clone().as_str()); 
                    self.clear_row_to_end(self.get_filerow() - 1, 0);
                    self.x = orig_len; self.y -= 1;
                } else {
                    self.row_delete_char(self.get_filerow() - 1, self.x - 1);
                    self.move_cursor(Key::Left)?;
                }
            },
            Key::Ctrl('o') => self.insert_row(self.get_filerow()),
            Key::Ctrl('s') => self.save_document(),
            _ => (), 
        };
        Ok(())
    } 

    ///*** screen output ***///
    fn draw_rows(&mut self) {
        print!("{}", termion::cursor::Goto(1,1));

        for r in 0..self.term.height - 1 {
            self.term.clear_term_line();           

            if r < self.rows.len() {
                let row = String::from(&self.rows[r + self.rowoff].data);
                let last_char = std::cmp::min(row.len(), self.term.width + self.coloff);
                print!("{}", &row[self.coloff..last_char]);
            } 
            
            else if r == self.term.height / 3 && self.rows.len() == 0 {
                let msg = "--------- Text Editor ---------";
                let offset = (self.term.width - msg.len()) / 2;
                let padding = " ".repeat(offset);    

                print!("~{}{}{}{}", padding, color::Fg(color::LightCyan), msg, color::Fg(color::Reset)); 
            }
            //else if r == self.t_height - 1 { print!("X"); } // status bar eventually
            else if r < self.term.height - 1 { print!("~"); } 

            // dont add newline, cr if we're on the last line
            if r != self.term.height - 1 { print!("\n\r") }
        }
         
        self.draw_status_line();

        self.term.move_cursor_to(self.x as u16, self.y as u16); 
    }

    fn draw_status_line(&mut self) {
        self.term.move_cursor_to(1, self.term.height as u16);
        self.term.clear_term_line();

        print!("{}{}", color::Bg(color::Rgb(111, 222, 150)), color::Fg(color::Rgb(0,0,0)));
        
        
        let status_line_file = match &self.doc {
            Some(d) => format!("> [{}]", d.filename),
            None => format!("> [NOFILE]"),
        };
        let status_line_pos = format!("{:3}, {:3}", self.y + self.rowoff, self.x + self.coloff);
        let offset = self.term.width / 3;
        let status_line = format!("{:3$}{:^3$}{:3}", status_line_file, status_line_pos, self.status_message, offset);

        print!("{:1$}", status_line, self.term.width);
            
        print!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset));
    }

    fn set_status_message(&mut self, msg: &str, show_timestamp: bool) {
        self.status_message = match show_timestamp {
            true => {
                let now = chrono::Local::now();
                format!("{} ({}.{}.{})", msg, now.hour(), now.minute(), now.second())
            },
            false => {
                format!("{}", msg)
            }
        };

    }

    fn prompt (&mut self, prompt: &str) -> String {
        let mut user_input = String::new();
        loop {
            self.set_status_message(format!("{}{}", prompt, user_input).as_str(), false);
            self.draw_status_line();
            self.term.refresh_screen();

            let key = self.get_keypress().unwrap();
            
            match key {
                Key::Char(c) => {
                    if (c == '\n' || c == '\r') && user_input.len() != 0 {
                        self.set_status_message("", false);
                        return user_input;
                    } else { user_input.push(c); } 
                },
                Key::Backspace => {
                    if user_input.len() > 0 { user_input.pop(); }
                }
                _ => (),
            }

        }

    }
    
    ///*** document editing ***///

    /// Insert a char into a document row at given index
    fn row_insert_char(&mut self, row: usize, at: usize, c: char) {
        if row >= self.rows.len() { self.insert_row(row); }
        if at > self.rows[row].data.len() { return }
        self.rows[row].data.insert(at, c);
    }

    /// Delete a char in a row at a given index
    fn row_delete_char(&mut self, row: usize, at: usize) {
        if at <= 0 { return }
        self.rows[row].data.remove(at - 1);
    }

    /// Insert a row into the document at a given position
    fn insert_row(&mut self, at: usize) {
        if at > self.rows.len() { return }
        self.rows.insert(at, Erow::default());
    }

    /// Append &str to a given row
    fn append_str_to_row(&mut self, at: usize, contents: &str) {
        if at > self.rows.len() - 1 { return } 
        self.rows[at].data.push_str(contents);
    }

    /// Delete all characters in a row from at to the row size
    fn clear_row_to_end(&mut self, row: usize, at: usize) {
        //if row > self.rows.len() - 1|| at > self.rows[row].data.len() - 1 { return } 

        self.rows[row].data.split_off(at).truncate(0);
    }

    fn save_document(&mut self) {
        match &self.doc {
            Some(doc) => {
                // Just write back to file
                doc.write_to_file(&self.rows);
            },
            None => {
                // Prompt the user for a name
                // Create a new document
                self.term.hide_cursor();
                let filename = self.prompt("Enter a filename: ");
                self.term.show_cursor();
                self.doc = Some(Document { filename: filename.clone(), saved: true });
                
                match &self.doc {
                    Some(doc) => doc.write_to_file(&self.rows),
                    None => (),
                }
            }
        }

        //self.set_status_message(format!("Document saved as \"{}\"!", &filename).as_str());
        self.set_status_message(format!("Document saved as \"{}\"!", self.doc.clone().unwrap().filename).as_str(), true);
    }

    fn get_filerow(&self) -> usize {
        self.y + self.rowoff
    }

    fn die(e: std::io::Error, msg: &str)
    {
        panic!("{} with error {}", e, msg);
    }
}
