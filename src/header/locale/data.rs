use core::str::FromStr;

use alloc::{boxed::Box, ffi::CString, string::String, vec::Vec};

use super::constants::*;
use crate::platform::types::{c_char, c_int};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/locale.h.html>.
/// this struct is not ordered like in the posix spec for readability
#[repr(C)]
#[derive(Clone)]
pub struct lconv {
    pub decimal_point: *mut c_char,
    pub thousands_sep: *mut c_char,
    pub grouping: *mut c_char,
    pub int_curr_symbol: *mut c_char,
    pub currency_symbol: *mut c_char,
    pub mon_decimal_point: *mut c_char,
    pub mon_thousands_sep: *mut c_char,
    pub mon_grouping: *mut c_char,
    pub positive_sign: *mut c_char,
    pub negative_sign: *mut c_char,
    pub int_frac_digits: c_char,
    pub frac_digits: c_char,
    pub p_cs_precedes: c_char,
    pub p_sep_by_space: c_char,
    pub n_cs_precedes: c_char,
    pub n_sep_by_space: c_char,
    pub p_sign_posn: c_char,
    pub n_sign_posn: c_char,
    pub int_p_cs_precedes: c_char,
    pub int_p_sep_by_space: c_char,
    pub int_n_cs_precedes: c_char,
    pub int_n_sep_by_space: c_char,
    pub int_p_sign_posn: c_char,
    pub int_n_sign_posn: c_char,
}
unsafe impl Sync for lconv {}

/// "POSIX" or "C" default
pub(crate) const fn posix_lconv() -> lconv {
    lconv {
        // numeric, non-monetary
        decimal_point: c".".as_ptr().cast_mut(),
        thousands_sep: c"".as_ptr().cast_mut(),
        grouping: c"".as_ptr().cast_mut(),
        // local monetary
        int_curr_symbol: c"".as_ptr().cast_mut(),
        currency_symbol: c"".as_ptr().cast_mut(),
        mon_decimal_point: c"".as_ptr().cast_mut(),
        mon_thousands_sep: c"".as_ptr().cast_mut(),
        mon_grouping: c"".as_ptr().cast_mut(),
        positive_sign: c"".as_ptr().cast_mut(),
        negative_sign: c"".as_ptr().cast_mut(),
        // delimiters, unspecified
        int_frac_digits: c_char::MAX,
        frac_digits: c_char::MAX,
        p_cs_precedes: c_char::MAX,
        p_sep_by_space: c_char::MAX,
        n_cs_precedes: c_char::MAX,
        n_sep_by_space: c_char::MAX,
        p_sign_posn: c_char::MAX,
        n_sign_posn: c_char::MAX,
        // international format
        int_p_cs_precedes: c_char::MAX,
        int_p_sep_by_space: c_char::MAX,
        int_n_cs_precedes: c_char::MAX,
        int_n_sep_by_space: c_char::MAX,
        int_p_sign_posn: c_char::MAX,
        int_n_sign_posn: c_char::MAX,
    }
}

#[repr(C)]
pub(crate) struct LocaleData {
    pub name: CString,
    pub lconv: lconv,
    // Owned memory buffers
    pub decimal_point: CString,
    pub thousands_sep: CString,
    pub grouping: Vec<c_char>,
    pub int_curr_symbol: CString,
    pub currency_symbol: CString,
    pub mon_decimal_point: CString,
    pub mon_thousands_sep: CString,
    pub mon_grouping: Vec<c_char>,
    pub positive_sign: CString,
    pub negative_sign: CString,
}
unsafe impl Sync for LocaleData {}

impl LocaleData {
    pub fn new(name: CString, defs: PosixLocaleDef) -> Box<Self> {
        let mut data = Box::new(LocaleData {
            name,
            decimal_point: Self::to_cstring(defs.decimal_point),
            thousands_sep: Self::to_cstring(defs.thousands_sep),
            grouping: Self::to_grouping_char(defs.grouping),
            int_curr_symbol: Self::to_cstring(defs.int_curr_symbol),
            currency_symbol: Self::to_cstring(defs.currency_symbol),
            mon_decimal_point: Self::to_cstring(defs.mon_decimal_point),
            mon_thousands_sep: Self::to_cstring(defs.mon_thousands_sep),
            mon_grouping: Self::to_grouping_char(defs.mon_grouping),
            positive_sign: Self::to_cstring(defs.positive_sign),
            negative_sign: Self::to_cstring(defs.negative_sign),
            lconv: unsafe { core::mem::zeroed() },
        });

        data.lconv.int_frac_digits = defs.int_frac_digits.unwrap_or(c_char::MAX);
        data.lconv.frac_digits = defs.frac_digits.unwrap_or(c_char::MAX);
        data.lconv.p_cs_precedes = defs.p_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.p_sep_by_space = defs.p_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.n_cs_precedes = defs.n_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.n_sep_by_space = defs.n_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.p_sign_posn = defs.p_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.n_sign_posn = defs.n_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.int_p_cs_precedes = defs.int_p_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.int_p_sep_by_space = defs.int_p_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.int_n_cs_precedes = defs.int_n_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.int_n_sep_by_space = defs.int_n_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.int_p_sign_posn = defs.int_p_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.int_n_sign_posn = defs.int_n_sign_posn.unwrap_or(c_char::MAX);

        data.update_lconv_pointers();
        data
    }

    pub fn posix() -> Box<Self> {
        LocaleData::new(CString::from_str("C").unwrap(), PosixLocaleDef::default())
    }

    fn update_lconv_pointers(&mut self) {
        self.lconv.decimal_point = self.decimal_point.as_ptr().cast_mut();
        self.lconv.thousands_sep = self.thousands_sep.as_ptr().cast_mut();
        self.lconv.grouping = self.grouping.as_ptr().cast_mut();
        self.lconv.int_curr_symbol = self.int_curr_symbol.as_ptr().cast_mut();
        self.lconv.currency_symbol = self.currency_symbol.as_ptr().cast_mut();
        self.lconv.mon_decimal_point = self.mon_decimal_point.as_ptr().cast_mut();
        self.lconv.mon_thousands_sep = self.mon_thousands_sep.as_ptr().cast_mut();
        self.lconv.mon_grouping = self.mon_grouping.as_ptr().cast_mut();
        self.lconv.positive_sign = self.positive_sign.as_ptr().cast_mut();
        self.lconv.negative_sign = self.negative_sign.as_ptr().cast_mut();
    }

    pub fn copy_category(&mut self, other: &Self, category: c_int) {
        match category {
            LC_NUMERIC => {
                self.decimal_point = other.decimal_point.clone();
                self.thousands_sep = other.thousands_sep.clone();
                self.grouping = other.grouping.clone();
                self.lconv.frac_digits = other.lconv.frac_digits;
            }
            LC_MONETARY => {
                self.int_curr_symbol = other.int_curr_symbol.clone();
                self.currency_symbol = other.currency_symbol.clone();
                self.mon_decimal_point = other.mon_decimal_point.clone();
                self.mon_thousands_sep = other.mon_thousands_sep.clone();
                self.mon_grouping = other.mon_grouping.clone();
                self.positive_sign = other.positive_sign.clone();
                self.negative_sign = other.negative_sign.clone();
                self.lconv.int_frac_digits = other.lconv.int_frac_digits;
                self.lconv.p_cs_precedes = other.lconv.p_cs_precedes;
                self.lconv.p_sep_by_space = other.lconv.p_sep_by_space;
                self.lconv.n_cs_precedes = other.lconv.n_cs_precedes;
                self.lconv.n_sep_by_space = other.lconv.n_sep_by_space;
                self.lconv.p_sign_posn = other.lconv.p_sign_posn;
                self.lconv.n_sign_posn = other.lconv.n_sign_posn;
                self.lconv.int_p_cs_precedes = other.lconv.int_p_cs_precedes;
                self.lconv.int_p_sep_by_space = other.lconv.int_p_sep_by_space;
                self.lconv.int_n_cs_precedes = other.lconv.int_n_cs_precedes;
                self.lconv.int_n_sep_by_space = other.lconv.int_n_sep_by_space;
                self.lconv.int_p_sign_posn = other.lconv.int_p_sign_posn;
                self.lconv.int_n_sign_posn = other.lconv.int_n_sign_posn;
            }
            LC_ALL => {
                *self = other.clone();
            }
            _ => {}
        }

        self.update_lconv_pointers();
    }

    fn to_cstring(opt: Option<CString>) -> CString {
        opt.unwrap_or_else(|| CString::new("").unwrap())
    }

    fn to_grouping_char(opt: Vec<Option<c_char>>) -> Vec<c_char> {
        let mut v: Vec<c_char> = opt.into_iter().map(Self::to_char).collect();
        v.push(0);
        v
    }

    fn to_char(opt: Option<c_char>) -> c_char {
        opt.unwrap_or(c_char::MAX)
    }
}

impl Clone for LocaleData {
    fn clone(&self) -> Self {
        let mut data = Self {
            name: self.name.clone(),
            lconv: self.lconv.clone(),
            decimal_point: self.decimal_point.clone(),
            thousands_sep: self.thousands_sep.clone(),
            grouping: self.grouping.clone(),
            int_curr_symbol: self.int_curr_symbol.clone(),
            currency_symbol: self.currency_symbol.clone(),
            mon_decimal_point: self.mon_decimal_point.clone(),
            mon_thousands_sep: self.mon_thousands_sep.clone(),
            mon_grouping: self.mon_grouping.clone(),
            positive_sign: self.positive_sign.clone(),
            negative_sign: self.negative_sign.clone(),
        };
        data.update_lconv_pointers();
        data
    }
}

#[derive(Clone)]
pub(crate) struct GlobalLocaleData {
    // names per LC_* constant
    pub names: [CString; 7],
    pub data: LocaleData,
}

impl GlobalLocaleData {
    pub fn new() -> Box<Self> {
        let data = LocaleData::posix();
        let names = [
            data.name.clone(),
            data.name.clone(),
            data.name.clone(),
            data.name.clone(),
            data.name.clone(),
            data.name.clone(),
            data.name.clone(),
        ];
        let mut r = Box::new(GlobalLocaleData { data: *data, names });
        r.data.update_lconv_pointers();
        r
    }

    pub fn get_name(&self, category: i32) -> Option<&CString> {
        self.names.get(category as usize)
    }
    pub fn set_name(&mut self, category: i32, name: CString) -> Option<&CString> {
        if self.names.get(category as usize).is_some() {
            self.names[category as usize] = name;
            self.names.get(category as usize)
        } else {
            None
        }
    }
}
unsafe impl Sync for GlobalLocaleData {}

#[derive(Default)]
pub(crate) struct PosixLocaleDef {
    pub decimal_point: Option<CString>,
    pub thousands_sep: Option<CString>,
    pub grouping: Vec<Option<c_char>>,
    pub int_curr_symbol: Option<CString>,
    pub currency_symbol: Option<CString>,
    pub mon_decimal_point: Option<CString>,
    pub mon_thousands_sep: Option<CString>,
    pub mon_grouping: Vec<Option<c_char>>,
    pub positive_sign: Option<CString>,
    pub negative_sign: Option<CString>,
    pub int_frac_digits: Option<c_char>,
    pub frac_digits: Option<c_char>,
    pub p_cs_precedes: Option<c_char>,
    pub p_sep_by_space: Option<c_char>,
    pub n_cs_precedes: Option<c_char>,
    pub n_sep_by_space: Option<c_char>,
    pub p_sign_posn: Option<c_char>,
    pub n_sign_posn: Option<c_char>,
    pub int_p_cs_precedes: Option<c_char>,
    pub int_p_sep_by_space: Option<c_char>,
    pub int_n_cs_precedes: Option<c_char>,
    pub int_n_sep_by_space: Option<c_char>,
    pub int_p_sign_posn: Option<c_char>,
    pub int_n_sign_posn: Option<c_char>,
}
impl PosixLocaleDef {
    //! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap07.html>
    pub fn parse(content: &str) -> Self {
        let mut locale = PosixLocaleDef::default();

        let mut lines = content.lines();
        loop {
            let Some(line) = lines.next() else {
                break;
            };

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut parts = trimmed.split_ascii_whitespace();
            let Some(key) = parts.next() else {
                continue;
            };
            let mut val: String = String::new();
            loop {
                let Some(chunk) = parts.next() else {
                    break;
                };
                if !chunk.ends_with('\\') {
                    val.push_str(chunk);
                    break;
                }
                // multiline values
                val.push_str(&chunk[0..chunk.len() - 1]);
                let Some(next_line) = lines.next() else {
                    break;
                };
                parts = next_line.split_ascii_whitespace();
            }

            if key.is_empty() || val.is_empty() {
                continue;
            }

            match key {
                "decimal_point" => locale.decimal_point = Self::parse_str(&val),
                "thousands_sep" => locale.thousands_sep = Self::parse_str(&val),
                "int_curr_symbol" => locale.int_curr_symbol = Self::parse_str(&val),
                "currency_symbol" => locale.currency_symbol = Self::parse_str(&val),
                "mon_decimal_point" => locale.mon_decimal_point = Self::parse_str(&val),
                "mon_thousands_sep" => locale.mon_thousands_sep = Self::parse_str(&val),
                "positive_sign" => locale.positive_sign = Self::parse_str(&val),
                "negative_sign" => locale.negative_sign = Self::parse_str(&val),
                "grouping" => locale.grouping = Self::parse_int_group(&val),
                "mon_grouping" => locale.mon_grouping = Self::parse_int_group(&val),
                "int_frac_digits" => locale.int_frac_digits = Self::parse_int(&val),
                "frac_digits" => locale.frac_digits = Self::parse_int(&val),
                "p_cs_precedes" => locale.p_cs_precedes = Self::parse_int(&val),
                "p_sep_by_space" => locale.p_sep_by_space = Self::parse_int(&val),
                "n_cs_precedes" => locale.n_cs_precedes = Self::parse_int(&val),
                "n_sep_by_space" => locale.n_sep_by_space = Self::parse_int(&val),
                "p_sign_posn" => locale.p_sign_posn = Self::parse_int(&val),
                "n_sign_posn" => locale.n_sign_posn = Self::parse_int(&val),
                "int_p_cs_precedes" => locale.int_p_cs_precedes = Self::parse_int(&val),
                "int_p_sep_by_space" => locale.int_p_sep_by_space = Self::parse_int(&val),
                "int_n_cs_precedes" => locale.int_n_cs_precedes = Self::parse_int(&val),
                "int_n_sep_by_space" => locale.int_n_sep_by_space = Self::parse_int(&val),
                "int_p_sign_posn" => locale.int_p_sign_posn = Self::parse_int(&val),
                "int_n_sign_posn" => locale.int_n_sign_posn = Self::parse_int(&val),
                _ => {}
            }
        }
        locale
    }

    /// parse e.g. `3;3;0` -> [ 3,3,0 ], `-1` -> [ None ]
    fn parse_int_group(val: &str) -> Vec<Option<c_char>> {
        val.split(';').map(Self::parse_int).collect()
    }

    /// parse e.g. `-1` -> None, `1` -> Some(1)
    fn parse_int(val: &str) -> Option<c_char> {
        let r = val.trim().parse::<c_char>().ok();
        if r.is_some_and(|i| i < 0) {
            return None;
        }
        r
    }

    /// parse e.g. `""`
    fn parse_str(val: &str) -> Option<CString> {
        let mut r = String::new();
        let mut v = val.chars();
        if v.next() != Some('"') {
            return None;
        }
        while let Some(c) = v.next() {
            if c == '"' {
                if v.next().is_some() {
                    return None;
                }
                break;
            }
            // TODO: Parse <..>
            r.push(c);
        }
        CString::new(r).ok()
    }
}
