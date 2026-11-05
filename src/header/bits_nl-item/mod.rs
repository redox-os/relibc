use crate::platform::types::size_t;

/// POSIX type for items used with `nl_langinfo`
/// In practice, this is an index into the string table.
pub type nl_item = size_t;
