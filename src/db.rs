use alloc::{string::String, vec::Vec};

use crate::{
    c_str::CStr,
    fs::File,
    header::fcntl,
    io::{self, BufRead, BufReader},
};

pub enum Separator {
    Character(char),
    Whitespace,
}

pub struct Db<R: BufRead> {
    reader: R,
    separator: Separator,
}

impl<R: BufRead> Db<R> {
    pub fn new(reader: R, separator: Separator) -> Self {
        Db { reader, separator }
    }

    pub fn read(&mut self) -> io::Result<Option<Vec<String>>> {
        let mut line = String::new();
        if self.reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }

        let vec = if let Some(not_comment) = line.trim().split('#').next() {
            match self.separator {
                Separator::Character(c) => not_comment.split(c).map(String::from).collect(),
                Separator::Whitespace => not_comment.split_whitespace().map(String::from).collect(),
            }
        } else {
            Vec::new()
        };

        Ok(Some(vec))
    }
}

pub type FileDb = Db<BufReader<File>>;

impl FileDb {
    pub fn open(path: &CStr, separator: Separator) -> io::Result<Self> {
        let file = File::open(path, fcntl::O_RDONLY | fcntl::O_CLOEXEC)?;
        Ok(Db::new(BufReader::new(file), separator))
    }
}
