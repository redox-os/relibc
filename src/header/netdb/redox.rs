use crate::{fs::File, header::fcntl, io::Read};
use alloc::string::String;

pub fn get_dns_server() -> String {
    let mut string = String::new();
    let mut file = File::open(c_str!("/etc/net/dns"), fcntl::O_RDONLY).unwrap(); // TODO: error handling
    file.read_to_string(&mut string).unwrap(); // TODO: error handling
    string
}
