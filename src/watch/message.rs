#[derive(Debug)]
pub struct Message {
    pub cmd: String,
    pub seq: i64,
    pub values: Vec<String>,
}

pub fn new_message(cmd: String, seq: i64, values: Vec<String>) -> Message {
    Message {
        cmd,
        seq,
        values,
    }
}

