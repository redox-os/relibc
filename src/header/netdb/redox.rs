use alloc::string::String;
use crate::c_str::CString;
use crate::fs::File;
use crate::header::fcntl;
use crate::io::Read;

pub fn get_dns_server() -> String {
    let mut string = String::new();
    let mut file = File::open(&CString::new("/etc/net/dns").unwrap(), fcntl::O_RDONLY).unwrap(); // TODO: error handling
    file.read_to_string(&mut string).unwrap(); // TODO: error handling
    string
}
