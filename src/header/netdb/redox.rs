use crate::{c_str::CString, fs::File, header::fcntl, io::Read};
use alloc::string::String;

pub fn get_dns_server() -> String {
    let mut string = String::new();
    let mut file = File::open(&CString::new("/etc/net/dns").unwrap(), fcntl::O_RDONLY).unwrap(); // TODO: error handling
    file.read_to_string(&mut string).unwrap(); // TODO: error handling
    string
}
