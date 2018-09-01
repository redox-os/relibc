use alloc::string::String;
use c_str::CString;
use platform::rawfile::file_read_all;

pub fn get_dns_server() -> String {
    String::from_utf8(file_read_all(&CString::new("/etc/net/dns").unwrap()).unwrap()).unwrap()
}
