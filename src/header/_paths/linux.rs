//! Linux specific constants for paths.h.
//! Entirely borrowed from musl as there isn't a list of what to include.

pub const _PATH_DEFPATH: &str = "/usr/local/bin:/bin:/usr/bin";
pub const _PATH_STDPATH: &str = "/bin:/usr/bin:/sbin:/usr/sbin";

pub const _PATH_BSHELL: &str = "/bin/sh";
pub const _PATH_CONSOLE: &str = "/dev/console";
pub const _PATH_DEVNULL: &str = "/dev/null";
pub const _PATH_KLOG: &str = "/proc/kmsg";
pub const _PATH_LASTLOG: &str = "/var/log/lastlog";
pub const _PATH_MAILDIR: &str = "/var/mail";
pub const _PATH_MAN: &str = "/usr/share/man";
pub const _PATH_MNTTAB: &str = "/etc/fstab";
pub const _PATH_NOLOGIN: &str = "/etc/nologin";
pub const _PATH_SENDMAIL: &str = "/usr/sbin/sendmail";
pub const _PATH_SHADOW: &str = "/etc/shadow";
pub const _PATH_SHELLS: &str = "/etc/shells";
pub const _PATH_TTY: &str = "/dev/tty";
pub const _PATH_UTMP: &str = "/var/run/utmp";
pub const _PATH_WTMP: &str = "/var/log/wtmp";
pub const _PATH_VI: &str = "/usr/bin/vi";

// Trailing backslash intentional as these are dir paths.
pub const _PATH_DEV: &str = "/dev/";
pub const _PATH_TMP: &str = "/tmp/";
pub const _PATH_VARDB: &str = "/var/lib/misc/";
pub const _PATH_VARRUN: &str = "/var/run/";
pub const _PATH_VARTMP: &str = "/var/tmp/";
