use crate::platform::types::c_int;

/// Affects the behavior of regular expressions and the collation functions.
pub const LC_COLLATE: c_int = 0;
/// Affects the behabior of regular expressions, character classification,
/// character conversion functions, and wide-character functions.
pub const LC_CTYPE: c_int = 1;
/// Affects the affirmative and negative response expressions returned by
/// `nl_langinfo` and the way message catalogs are located. It may also affect
/// the behavior of functions that return or write message strings.
pub const LC_MESSAGES: c_int = 2;
/// Affects the behavior of functions that handle monetary values.
pub const LC_MONETARY: c_int = 3;
/// Affects the behavior of functions that handle numeric values.
pub const LC_NUMERIC: c_int = 4;
/// Affects the behavior of the time conversion functions.
pub const LC_TIME: c_int = 5;
/// Affects the entire global locale.
pub const LC_ALL: c_int = 6;
/// A bitmask to set the COLLATE category.
pub const LC_COLLATE_MASK: c_int = 0x1;
/// A bitmask to set the CTYPE category.
pub const LC_CTYPE_MASK: c_int = 0x2;
/// A bitmask to set the MESSAGES category.
pub const LC_MESSAGES_MASK: c_int = 0x4;
/// A bitmask to set the MONETARY category.
pub const LC_MONETARY_MASK: c_int = 0x8;
/// A bitmask to set the NUMERIC category.
pub const LC_NUMERIC_MASK: c_int = 0x10;
/// A bitmask to set the TIME category.
pub const LC_TIME_MASK: c_int = 0x20;
/// A bitmask to set all categories.
pub const LC_ALL_MASK: c_int = 0b111111;
