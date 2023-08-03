
#[derive(Debug)]
pub struct Message {
    cmd: String,
    seq: i64,
    values: Vec<String>,
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