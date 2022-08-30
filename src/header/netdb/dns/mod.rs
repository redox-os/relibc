// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub use self::{answer::DnsAnswer, query::DnsQuery};

use alloc::{string::String, vec::Vec};

mod answer;
mod query;

#[derive(Clone, Debug)]
pub struct Dns {
    pub transaction_id: u16,
    pub flags: u16,
    pub queries: Vec<DnsQuery>,
    pub answers: Vec<DnsAnswer>,
}

impl Dns {
    pub fn compile(&self) -> Vec<u8> {
        let mut data = Vec::new();

        macro_rules! push_u8 {
            ($value:expr) => {
                data.push($value);
            };
        }

        macro_rules! push_n16 {
            ($value:expr) => {
                data.extend_from_slice(&u16::to_be_bytes($value));
            };
        }

        push_n16!(self.transaction_id);
        push_n16!(self.flags);
        push_n16!(self.queries.len() as u16);
        push_n16!(self.answers.len() as u16);
        push_n16!(0);
        push_n16!(0);

        for query in self.queries.iter() {
            for part in query.name.split('.') {
                push_u8!(part.len() as u8);
                data.extend_from_slice(part.as_bytes());
            }
            push_u8!(0);
            push_n16!(query.q_type);
            push_n16!(query.q_class);
        }
        data
    }

    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let name_ind = 0b1100_0000;
        let mut i = 0;

        macro_rules! pop_u8 {
            () => {{
                i += 1;
                if i > data.len() {
                    return Err(format!("{}: {}: pop_u8", file!(), line!()));
                }
                data[i - 1]
            }};
        }

        macro_rules! pop_n16 {
            () => {{
                use core::convert::TryInto;
                i += 2;
                if i > data.len() {
                    return Err(format!("{}: {}: pop_n16", file!(), line!()));
                }
                let bytes: [u8; 2] = data[i - 2..i].try_into().unwrap();
                u16::from_be_bytes(bytes)
            }};
        }

        macro_rules! pop_data {
            () => {{
                let mut data = Vec::new();

                let data_len = pop_n16!();
                for _data_i in 0..data_len {
                    data.push(pop_u8!());
                }

                data
            }};
        }

        macro_rules! pop_name {
            () => {{
                let mut name = String::new();
                let old_i = i;

                loop {
                    let name_len = pop_u8!();
                    if name_len & name_ind == name_ind {
                        i -= 1;
                        i = (pop_n16!() - ((name_ind as u16) << 8)) as usize;
                        continue;
                    }
                    if name_len == 0 {
                        break;
                    }
                    if !name.is_empty() {
                        name.push('.');
                    }
                    for _name_i in 0..name_len {
                        name.push(pop_u8!() as char);
                    }
                }

                if i <= old_i {
                    i = old_i + 2;
                }

                name
            }};
        }

        let transaction_id = pop_n16!();
        let flags = pop_n16!();
        let queries_len = pop_n16!();
        let answers_len = pop_n16!();
        pop_n16!();
        pop_n16!();

        let mut queries = Vec::new();
        for _query_i in 0..queries_len {
            queries.push(DnsQuery {
                name: pop_name!(),
                q_type: pop_n16!(),
                q_class: pop_n16!(),
            });
        }

        let mut answers = Vec::new();
        for _answer_i in 0..answers_len {
            answers.push(DnsAnswer {
                name: pop_name!(),
                a_type: pop_n16!(),
                a_class: pop_n16!(),
                ttl_a: pop_n16!(),
                ttl_b: pop_n16!(),
                data: pop_data!(),
            });
        }

        Ok(Dns {
            transaction_id,
            flags,
            queries,
            answers,
        })
    }
}
