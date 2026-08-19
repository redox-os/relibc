use core::num::ParseIntError;

use crate::platform::types::c_long;

#[derive(Debug)]
pub struct PosixTz<'a> {
    pub std: &'a str,
    pub dst: &'a str,
    pub daylight: bool,
    pub timezone: Option<c_long>,

    std_offset: Option<i32>, // offset in seconds
    dst_offset: Option<i32>, // offset in seconds
    start: Option<TransitionTime>,
    end: Option<TransitionTime>,
}

#[derive(Debug)]
struct TransitionTime {
    month: u8,         // 1-12
    week_of_month: u8, // 1-5
    day_of_week: u8,   // 0-6, 0 = Sun
    time: Option<u32>, // time of transition in seconds
}

impl<'a> PosixTz<'a> {
    fn default() -> Self {
        Self {
            std: "",
            dst: "",
            daylight: false,
            timezone: None,

            std_offset: None,
            dst_offset: None,
            start: None,
            end: None,
        }
    }

    pub fn parse(input: &'a str) -> PosixTz<'a> {
        let mut result = PosixTz::default();

        // Interesting empty case
        if input.is_empty() {
            result.std = "UTC";
            result.dst = "UTC";
            return result;
        }

        let mut input_split = input.split(',');
        let front = input_split.next().unwrap_or("");

        let (std, std_offset, remaining) = PosixTz::collect_tz_and_offset(front);
        result.std = std;
        if !std.is_empty() {
            result.std_offset = std_offset;
            result.timezone = Some(c_long::from(std_offset.unwrap_or(0)).clamp(-86400, 86400));

            // dst rules: same as std
            let (dst, _, _) = PosixTz::collect_tz_and_offset(remaining);
            result.dst = dst;

            if !dst.is_empty() {
                result.daylight = true;
            }

            // Likely defaults to std_offset - 3600. Will need to test in another method when needed
            result.dst_offset = std_offset; // Possibly keep as None if 0. Need to study other functions
        }

        // UNIMPLEMENTED: start/end aren't required by tzset, will implement after testing functions
        // that require it
        let mut has_transition_rules = false;
        if let Some(_start) = input_split.next() {
            has_transition_rules = true;
            result.daylight = true;
        }
        if let Some(_end) = input_split.next() {
            has_transition_rules = true;
            result.daylight = true;
        }

        // Set dst to std if:
        // 1. dst is not empty
        // 2. std has an offset
        // 3. didn't include start/end
        if result.dst.is_empty() && result.std_offset.is_some() && !has_transition_rules {
            result.dst = std;
        }

        result
    }

    fn collect_tz_and_offset(input: &str) -> (&str, Option<i32>, &str) {
        let mut result = ("", None, input);

        // TZ rules: 3+ ascii characters
        let std_end = input
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(input.len());
        let std = &input[..std_end];
        if std.len() >= 3 {
            result.0 = std;
        }
        let remaining = &input[std_end..];

        // offset rules: +-hh (optional :mm (optional :ss))
        let offset_end = remaining
            .find(|c: char| c.is_ascii_alphabetic())
            .unwrap_or(remaining.len());
        let offset_secs = PosixTz::time_to_seconds(&remaining[..offset_end]).ok();
        result.1 = offset_secs;
        result.2 = &remaining[offset_end..];

        result
    }

    fn time_to_seconds(time_str: &str) -> Result<i32, ParseIntError> {
        let mut result = 0;
        let mut time_split = time_str.split(':');
        let mut is_neg = false;
        if let Some(hour) = time_split.next() {
            let hours = hour.parse::<i32>()? * 60 * 60;
            is_neg = hours.is_negative();

            result += hours.abs();
        }
        if let Some(min) = time_split.next() {
            result += min.parse::<i32>()? * 60;
        }
        if let Some(sec) = time_split.next() {
            result += sec.parse::<i32>()?;
        }

        if is_neg {
            result *= -1;
        }
        Ok(result)
    }
}
