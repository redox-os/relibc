use core::str::FromStr;

use alloc::{
    boxed::Box,
    ffi::CString,
    string::{String, ToString},
    vec::Vec,
};

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
        decimal_point: b".\0".as_ptr() as *mut c_char,
        thousands_sep: b"\0".as_ptr() as *mut c_char,
        grouping: b"\0".as_ptr() as *mut c_char,
        // local monetary
        int_curr_symbol: b"\0".as_ptr() as *mut c_char,
        currency_symbol: b"\0".as_ptr() as *mut c_char,
        mon_decimal_point: b"\0".as_ptr() as *mut c_char,
        mon_thousands_sep: b"\0".as_ptr() as *mut c_char,
        mon_grouping: b"\0".as_ptr() as *mut c_char,
        positive_sign: b"\0".as_ptr() as *mut c_char,
        negative_sign: b"\0".as_ptr() as *mut c_char,
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
    pub grouping: Vec<u8>,
    pub int_curr_symbol: CString,
    pub currency_symbol: CString,
    pub mon_decimal_point: CString,
    pub mon_thousands_sep: CString,
    pub mon_grouping: Vec<u8>,
    pub positive_sign: CString,
    pub negative_sign: CString,
}
unsafe impl Sync for LocaleData {}

impl LocaleData {
    pub fn new(name: CString, toml: RawLocale) -> Box<Self> {
        let mut data = Box::new(LocaleData {
            name,
            decimal_point: Self::to_cstring(toml.decimal_point),
            thousands_sep: Self::to_cstring(toml.thousands_sep),
            grouping: Self::to_grouping(toml.grouping),
            int_curr_symbol: Self::to_cstring(toml.int_curr_symbol),
            currency_symbol: Self::to_cstring(toml.currency_symbol),
            mon_decimal_point: Self::to_cstring(toml.mon_decimal_point),
            mon_thousands_sep: Self::to_cstring(toml.mon_thousands_sep),
            mon_grouping: Self::to_grouping(toml.mon_grouping),
            positive_sign: Self::to_cstring(toml.positive_sign),
            negative_sign: Self::to_cstring(toml.negative_sign),
            lconv: unsafe { core::mem::zeroed() },
        });

        data.lconv.int_frac_digits = toml.int_frac_digits.unwrap_or(c_char::MAX);
        data.lconv.frac_digits = toml.frac_digits.unwrap_or(c_char::MAX);
        data.lconv.p_cs_precedes = toml.p_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.p_sep_by_space = toml.p_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.n_cs_precedes = toml.n_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.n_sep_by_space = toml.n_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.p_sign_posn = toml.p_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.n_sign_posn = toml.n_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.int_p_cs_precedes = toml.int_p_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.int_p_sep_by_space = toml.int_p_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.int_n_cs_precedes = toml.int_n_cs_precedes.unwrap_or(c_char::MAX);
        data.lconv.int_n_sep_by_space = toml.int_n_sep_by_space.unwrap_or(c_char::MAX);
        data.lconv.int_p_sign_posn = toml.int_p_sign_posn.unwrap_or(c_char::MAX);
        data.lconv.int_n_sign_posn = toml.int_n_sign_posn.unwrap_or(c_char::MAX);

        data.update_lconv_pointers();
        data
    }

    pub fn posix() -> Box<Self> {
        LocaleData::new(CString::from_str("C").unwrap(), RawLocale::default())
    }

    fn update_lconv_pointers(&mut self) {
        self.lconv.decimal_point = self.decimal_point.as_ptr() as *mut c_char;
        self.lconv.thousands_sep = self.thousands_sep.as_ptr() as *mut c_char;
        self.lconv.grouping = self.grouping.as_ptr() as *mut c_char;
        self.lconv.int_curr_symbol = self.int_curr_symbol.as_ptr() as *mut c_char;
        self.lconv.currency_symbol = self.currency_symbol.as_ptr() as *mut c_char;
        self.lconv.mon_decimal_point = self.mon_decimal_point.as_ptr() as *mut c_char;
        self.lconv.mon_thousands_sep = self.mon_thousands_sep.as_ptr() as *mut c_char;
        self.lconv.mon_grouping = self.mon_grouping.as_ptr() as *mut c_char;
        self.lconv.positive_sign = self.positive_sign.as_ptr() as *mut c_char;
        self.lconv.negative_sign = self.negative_sign.as_ptr() as *mut c_char;
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

    fn to_cstring(opt: Option<String>) -> CString {
        opt.and_then(|s| CString::new(s).ok())
            .unwrap_or_else(|| CString::new("").unwrap())
    }

    fn to_grouping(opt: Option<Vec<c_char>>) -> Vec<u8> {
        let mut v: Vec<u8> = opt
            .unwrap_or_default()
            .into_iter()
            .map(|x| x as u8)
            .collect();
        v.push(0);
        v
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
pub(crate) struct RawLocale {
    pub decimal_point: Option<String>,
    pub thousands_sep: Option<String>,
    pub grouping: Option<Vec<c_char>>,
    pub int_curr_symbol: Option<String>,
    pub currency_symbol: Option<String>,
    pub mon_decimal_point: Option<String>,
    pub mon_thousands_sep: Option<String>,
    pub mon_grouping: Option<Vec<c_char>>,
    pub positive_sign: Option<String>,
    pub negative_sign: Option<String>,
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
impl RawLocale {
    pub fn parse(content: &str) -> Self {
        let mut locale = RawLocale::default();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut parts = trimmed.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim();

            if key.is_empty() || val.is_empty() {
                continue;
            }

            match key {
                "decimal_point" => locale.decimal_point = Some(val.to_string()),
                "thousands_sep" => locale.thousands_sep = Some(val.to_string()),
                "int_curr_symbol" => locale.int_curr_symbol = Some(val.to_string()),
                "currency_symbol" => locale.currency_symbol = Some(val.to_string()),
                "mon_decimal_point" => locale.mon_decimal_point = Some(val.to_string()),
                "mon_thousands_sep" => locale.mon_thousands_sep = Some(val.to_string()),
                "positive_sign" => locale.positive_sign = Some(val.to_string()),
                "negative_sign" => locale.negative_sign = Some(val.to_string()),
                "grouping" => locale.grouping = Some(Self::parse_grouping(val)),
                "mon_grouping" => locale.mon_grouping = Some(Self::parse_grouping(val)),
                "int_frac_digits" => locale.int_frac_digits = val.parse().ok(),
                "frac_digits" => locale.frac_digits = val.parse().ok(),
                "p_cs_precedes" => locale.p_cs_precedes = val.parse().ok(),
                "p_sep_by_space" => locale.p_sep_by_space = val.parse().ok(),
                "n_cs_precedes" => locale.n_cs_precedes = val.parse().ok(),
                "n_sep_by_space" => locale.n_sep_by_space = val.parse().ok(),
                "p_sign_posn" => locale.p_sign_posn = val.parse().ok(),
                "n_sign_posn" => locale.n_sign_posn = val.parse().ok(),
                "int_p_cs_precedes" => locale.int_p_cs_precedes = val.parse().ok(),
                "int_p_sep_by_space" => locale.int_p_sep_by_space = val.parse().ok(),
                "int_n_cs_precedes" => locale.int_n_cs_precedes = val.parse().ok(),
                "int_n_sep_by_space" => locale.int_n_sep_by_space = val.parse().ok(),
                "int_p_sign_posn" => locale.int_p_sign_posn = val.parse().ok(),
                "int_n_sign_posn" => locale.int_n_sign_posn = val.parse().ok(),
                _ => {}
            }
        }
        locale
    }

    /// parse e.g. "3,3,0"
    fn parse_grouping(val: &str) -> Vec<c_char> {
        val.split(',')
            .filter_map(|s| s.trim().parse::<c_char>().ok())
            .collect()
    }
}
