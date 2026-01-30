// This test file exist only to check dependency graph, using `redoxer cc includes.c -H`

// generate: find src/header -mindepth 1 -maxdepth 1 -type d -not -name "_*" -printf "#include <%f.h>\n" | sed -e 's|_|/|g' | sort

// TODO: No include guard in these headers, probably expected?
// #include <arch/aarch64/user.h>
// #include <arch/riscv64/user.h>
// #include <arch/x64/user.h>

#include <arpa/inet.h>
#include <assert.h>
#include <bits/locale-t.h>
#include <bits/pthread.h>
#include <bits/sched.h>
#include <crypt.h>
#include <ctype.h>
#include <dirent.h>
#include <dlfcn.h>
#include <dl-tls.h>
#include <elf.h>
#include <endian.h>
#include <err.h>
#include <errno.h>
#include <fcntl.h>
#include <float.h>
#include <fnmatch.h>
#include <getopt.h>
#include <glob.h>
#include <grp.h>
#include <ifaddrs.h>
#include <inttypes.h>
#include <langinfo.h>
#include <libgen.h>
#include <limits.h>
#include <locale.h>
#include <malloc.h>
#include <monetary.h>
#include <netdb.h>
#include <net/if.h>
#include <netinet/in.h>
#include <netinet/ip.h>
#include <netinet/tcp.h>
#include <poll.h>
#include <pthread.h>
#include <pty.h>
#include <pwd.h>
#include <regex.h>
#include <sched.h>
#include <semaphore.h>
#include <setjmp.h>
#include <sgtty.h>
#include <shadow.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/auxv.h>
#include <sys/epoll.h>
#include <sys/file.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/procfs.h>
#include <sys/ptrace.h>
#include <sys/random.h>
#include <sys/resource.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/syslog.h>
#include <sys/timeb.h>
#include <sys/time.h>
#include <sys/times.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <sys/un.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <tar.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>
#include <utime.h>
#include <utmp.h>
#include <wchar.h>
#include <wctype.h>

int main() {
}
