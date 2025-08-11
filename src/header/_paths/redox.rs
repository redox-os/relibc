//! Redox specific constants for paths.h.

// NOTE: Cross check that new entries correspond to files on Redox before adding.

pub const _PATH_BSHELL: &str = "/bin/sh";
pub const _PATH_DEVNULL: &str = "/dev/null";
pub const _PATH_MAN: &str = "/usr/share/man";
pub const _PATH_TTY: &str = "/dev/tty";

// Trailing backslash intentional as these are dir paths.
pub const _PATH_DEV: &str = "/dev/";
pub const _PATH_TMP: &str = "/tmp/";
pub const _PATH_VARTMP: &str = "/var/tmp/";
