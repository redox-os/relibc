use alloc::string::{String, ToString};
use alloc::vec::Vec;

use c_str::CStr;
use fs::File;
use header::fcntl;
use io::{self, BufRead, BufReader};

pub struct Db(BufReader<File>);

impl Db {
    pub fn new(path: &CStr) -> io::Result<Self> {
        File::open(path, fcntl::O_RDONLY)
            .map(BufReader::new)
            .map(Db)
    }

    pub fn read(&mut self) -> io::Result<Vec<String>> {
        let mut parts = Vec::new();

        let mut line = String::new();
        self.0.read_line(&mut line)?;
        if let Some(not_comment) = line.split('#').next() {
            for part in not_comment.split_whitespace() {
                parts.push(part.to_string());
            }
        }

        Ok(parts)
    }
}
