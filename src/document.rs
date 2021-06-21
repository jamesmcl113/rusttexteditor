use std::fs::File;
use std::io::{prelude::*, BufReader};

#[derive(Clone)]
pub struct Document {
    pub filename: String, // change this to an Option
    pub saved: bool,
}

impl Document {
    pub fn write_to_file(&self, rows: &Vec<Erow>) {
        let mut f = File::create(&self.filename).unwrap();
        for r in rows {
            f.write(r.data.as_bytes()).unwrap();
            f.write(b"\n").unwrap();
        }
    }
}

/// Represents a row of text in a document
pub struct Erow {
    pub data: String,
    pub data_rendered: String,
}

impl Erow {
    pub fn default() -> Erow {
       Erow { data: String::new(), data_rendered: String::new() } 
    }
}

/// Reads contents of a file into a Vec of Erows
/// Creates a new file if one doesn't exists
/// TODO: add proper error handling, fix 
pub fn read_into_document(filename: &str) -> (Vec<Erow>, Option<Document>) {
    match File::open(filename) {
        Ok(f) => {
            let reader = BufReader::new(f);

            let mut rows: Vec<Erow> = Vec::new();
            for line in reader.lines() {
                let data = line.unwrap();
                let data_rendered = data.clone();
                rows.push(Erow {data , data_rendered });
            }

            (rows, Some(Document { filename: String::from(filename), saved: true }))
        },
        Err(_e) => {
            File::create(filename).unwrap();
            let rows: Vec<Erow> = Vec::new();

            (rows, Some(Document {filename: String::from(filename), saved: true}))
        }
    }
}
