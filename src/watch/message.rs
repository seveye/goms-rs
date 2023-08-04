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

