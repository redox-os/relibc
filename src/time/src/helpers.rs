use constants::*;
use platform::types::*;

// compute year, month, day & day of year
// for description of this algorithm see
// http://howardhinnant.github.io/date_algorithms.html#civil_from_days
#[inline(always)]
pub(crate) fn civil_from_days(days: c_long) -> (c_int, c_int, c_int, c_int) {
    let (era, year): (c_int, c_int);
    let (erayear, mut yearday, mut month, day): (c_int, c_int, c_int, c_int);
    let eraday: c_ulong;

    era = (if days >= 0 {
        days
    } else {
        days - (DAYS_PER_ERA - 1)
    } / DAYS_PER_ERA) as c_int;
    eraday = (days - era as c_long * DAYS_PER_ERA) as c_ulong;
    let a = eraday / (DAYS_PER_4_YEARS - 1);
    let b = eraday / DAYS_PER_CENTURY;
    let c = eraday / (DAYS_PER_ERA as c_ulong - 1);
    erayear = ((eraday - a + b - c) / 365) as c_int;
    let d = DAYS_PER_YEAR * erayear + erayear / 4 - erayear / 100;
    yearday = (eraday - d as c_ulong) as c_int;
    month = (5 * yearday + 2) / 153;
    day = yearday - (153 * month + 2) / 5 + 1;
    month += if month < 10 { 2 } else { -10 };
    year = ADJUSTED_EPOCH_YEAR + erayear + era * YEARS_PER_ERA + (month <= 1) as c_int;
    yearday += if yearday >= DAYS_PER_YEAR - DAYS_IN_JANUARY - DAYS_IN_FEBRUARY {
        -(DAYS_PER_YEAR - DAYS_IN_JANUARY - DAYS_IN_FEBRUARY)
    } else {
        DAYS_IN_JANUARY + DAYS_IN_FEBRUARY + is_leap(erayear)
    };
    return (year, month, day, yearday);
}

#[inline(always)]
fn is_leap(y: c_int) -> c_int {
    ((y % 4 == 0 && y % 100 != 0) || y % 400 == 0) as c_int
}
