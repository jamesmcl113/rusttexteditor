use termion::raw::IntoRawMode;
use std::io::stdout;

mod terminal;
mod document;
mod editor;
use editor::Editor;

fn main() {
    let mut _stdout = stdout().into_raw_mode().unwrap();
    let mut e = Editor::new(); 

    e.run();
}
