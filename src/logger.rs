use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::Mutex;

lazy_static::lazy_static! {
    pub static ref LOG_FILE: Mutex<BufWriter<std::fs::File>> = {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.txt")
            .unwrap();

        Mutex::new(BufWriter::with_capacity(8 * 1024 * 1024, file))
    };
}