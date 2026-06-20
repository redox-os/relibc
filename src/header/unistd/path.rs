use core::slice::Split;

use arrayvec::ArrayVec;

use crate::{c_str::CStr, header::limits::PATH_MAX};

pub struct PathSearchIter<'a> {
    file_bytes: &'a [u8],
    path_splits: Split<'a, u8, fn(&u8) -> bool>,
}

const PATH_SEPARATOR: u8 = b':';

impl<'a> PathSearchIter<'a> {
    /// Construct a new PATH parser.
    /// Safety: file must have no slashes
    pub fn new(file_bytes: &'a [u8], path_env: &'a CStr) -> Self {
        Self {
            file_bytes,
            path_splits: path_env.to_bytes().split(|&b| b == PATH_SEPARATOR),
        }
    }
}

impl<'a> Iterator for PathSearchIter<'a> {
    type Item = ArrayVec<u8, PATH_MAX>;

    fn next(&mut self) -> Option<Self::Item> {
        for path in &mut self.path_splits {
            let len = path.len() + self.file_bytes.len() + 2;
            if len > PATH_MAX {
                continue;
            }
            let mut program: ArrayVec<u8, PATH_MAX> = ArrayVec::new();
            program.try_extend_from_slice(path).unwrap();
            program.push(b'/');
            program.try_extend_from_slice(self.file_bytes).unwrap();
            program.push(b'\0');
            return Some(program);
        }

        None
    }
}
