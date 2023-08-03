use std::{io::{Cursor, Error}};

#[derive(Debug)]
pub struct Message {
    pub cmd: String,
    pub seq: i64,
    pub values: Vec<String>,
}

impl Message {
    pub fn new(cmd: String, seq: i64, values: Vec<String>) -> Self {
        Self {
            cmd,
            seq,
            values,
        }
    }
}


// / Find a line
pub fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 1) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::new(std::io::ErrorKind::Other, "No line found"))
}