use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::{IntoIter, Vec},
};
use core::mem;

use crate::platform::{types::*, Pal, Sys};

use crate::header::{
    arpa_inet::htons,
    errno::*,
    netinet_in::{in_addr, sockaddr_in, IPPROTO_UDP},
    sys_socket::{
        self,
        constants::{AF_INET, SOCK_DGRAM},
        sockaddr, socklen_t,
    },
    time::{self, timespec},
};

use super::{
    dns::{Dns, DnsQuery},
    sys::get_dns_server,
};

pub struct LookupHost(IntoIter<in_addr>);

impl Iterator for LookupHost {
    type Item = in_addr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub fn lookup_host(host: &str) -> Result<LookupHost, c_int> {
    let dns_string = get_dns_server();

    let dns_vec: Vec<u8> = dns_string
        .trim()
        .split('.')
        .map(|octet| octet.parse::<u8>().unwrap_or(0))
        .collect();

    if dns_vec.len() == 4 {
        let mut dns_arr = [0u8; 4];
        for (i, octet) in dns_vec.iter().enumerate() {
            dns_arr[i] = *octet;
        }
        let dns_addr = u32::from_ne_bytes(dns_arr);

        let mut timespec = timespec::default();
        unsafe {
            Sys::clock_gettime(time::constants::CLOCK_REALTIME, &mut timespec);
        }
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![DnsQuery {
                name: host.to_string(),
                q_type: 0x0001,
                q_class: 0x0001,
            }],
            answers: vec![],
        };

        let packet_data = packet.compile();
        let packet_data_len = packet_data.len();

        let packet_data_box = packet_data.into_boxed_slice();
        let packet_data_ptr = Box::into_raw(packet_data_box) as *mut _ as *mut c_void;

        let dest = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(53),
            sin_addr: in_addr { s_addr: dns_addr },
            ..Default::default()
        };
        let dest_ptr = &dest as *const _ as *const sockaddr;

        let sock = unsafe {
            let sock = sys_socket::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP as i32);
            if sys_socket::connect(sock, dest_ptr, mem::size_of_val(&dest) as socklen_t) < 0 {
                return Err(EIO);
            }
            if sys_socket::send(sock, packet_data_ptr, packet_data_len, 0) < 0 {
                Box::from_raw(packet_data_ptr);
                return Err(EIO);
            }
            sock
        };

        unsafe {
            Box::from_raw(packet_data_ptr);
        }

        let i = 0 as socklen_t;
        let mut buf = vec![0u8; 65536];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;

        let count = unsafe { sys_socket::recv(sock, buf_ptr, 65536, 0) };
        if count < 0 {
            return Err(EIO);
        }

        match Dns::parse(&buf[..count as usize]) {
            Ok(response) => {
                let addrs: Vec<_> = response
                    .answers
                    .into_iter()
                    .filter_map(|answer| {
                        if answer.a_type == 0x0001
                            && answer.a_class == 0x0001
                            && answer.data.len() == 4
                        {
                            let addr = in_addr {
                                s_addr: u32::from_ne_bytes([
                                    answer.data[0],
                                    answer.data[1],
                                    answer.data[2],
                                    answer.data[3],
                                ]),
                            };
                            Some(addr)
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(LookupHost(addrs.into_iter()))
            }
            Err(_err) => Err(EINVAL),
        }
    } else {
        Err(EINVAL)
    }
}

pub fn lookup_addr(addr: in_addr) -> Result<Vec<Vec<u8>>, c_int> {
    let dns_string = get_dns_server();

    let dns_vec: Vec<u8> = dns_string
        .trim()
        .split('.')
        .map(|octet| octet.parse::<u8>().unwrap_or(0))
        .collect();

    let mut dns_arr = [0u8; 4];

    for (i, octet) in dns_vec.iter().enumerate() {
        dns_arr[i] = *octet;
    }

    let addr: [u8; 4] = addr.s_addr.to_ne_bytes();
    // Address intentionally backwards for reverse lookup
    let name = format!(
        "{}.{}.{}.{}.in-addr.arpa",
        addr[3], addr[2], addr[1], addr[0]
    );

    if dns_vec.len() == 4 {
        let mut timespec = timespec::default();
        unsafe { Sys::clock_gettime(time::constants::CLOCK_REALTIME, &mut timespec) };
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![DnsQuery {
                name,
                q_type: 0x000C,
                q_class: 0x0001,
            }],
            answers: vec![],
        };

        let packet_data = packet.compile();
        let packet_data_len = packet_data.len();
        let packet_data_box = packet_data.into_boxed_slice();
        let packet_data_ptr = Box::into_raw(packet_data_box) as *mut _ as *mut c_void;

        let dest = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(53),
            sin_addr: in_addr {
                s_addr: u32::from_ne_bytes(dns_arr),
            },
            ..Default::default()
        };

        let dest_ptr = &dest as *const _ as *const sockaddr;

        let sock = unsafe {
            let sock = sys_socket::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP as i32);
            if sys_socket::connect(sock, dest_ptr, mem::size_of_val(&dest) as socklen_t) < 0 {
                return Err(EIO);
            }
            sock
        };

        unsafe {
            if sys_socket::send(sock, packet_data_ptr, packet_data_len, 0) < 0 {
                return Err(EIO);
            }
        }

        unsafe {
            Box::from_raw(packet_data_ptr);
        }

        let i = mem::size_of::<sockaddr_in>() as socklen_t;
        let mut buf = [0u8; 65536];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;

        let count = unsafe { sys_socket::recv(sock, buf_ptr, 65536, 0) };
        if count < 0 {
            return Err(EIO);
        }

        match Dns::parse(&buf[..count as usize]) {
            Ok(response) => {
                let names = response
                    .answers
                    .into_iter()
                    .filter_map(|answer| {
                        if answer.a_type == 0x000C && answer.a_class == 0x0001 {
                            // answer.data is encoded kinda weird.
                            // Basically length-prefixed strings for each
                            // subsection of the domain.
                            // We need to parse this to insert periods where
                            // they belong (ie at the end of each string)
                            Some(parse_revdns_answer(&answer.data))
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(names)
            }
            Err(_err) => Err(EINVAL),
        }
    } else {
        Err(EINVAL)
    }
}

fn parse_revdns_answer(data: &[u8]) -> Vec<u8> {
    let mut cursor = 0;
    let mut index = 0;
    let mut output = data.to_vec();
    while index < data.len() - 1 {
        let offset = data[index] as usize;
        index = cursor + offset + 1;
        output[index] = b'.';
        cursor = index;
    }
    //we don't want an extra period at the end
    output.pop();
    output
}
