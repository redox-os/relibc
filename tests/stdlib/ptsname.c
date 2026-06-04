/* Test for ptsname/ptsname_r.
   Copyright (C) 2014-2024 Free Software Foundation, Inc.
   This file is part of the GNU C Library.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, see
   <https://www.gnu.org/licenses/>.  */

#define _XOPEN_SOURCE 600
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>

#define DEV_TTY		"/dev/tty"
#define PTSNAME_EINVAL	"./ptsname-einval"

int do_single_test(int fd, char * buf, size_t buflen, int expected_err) {

    int ret = ptsname_r(fd, buf, buflen);

    if (expected_err == 0) {
        if (ret != 0) {
            printf("ptsname_r: expected: return = 0\n");
            printf("           got: return = %d (%s)\n",
                ret, strerror(ret));
            return 1;
        }
    } else {
        if (ret == 0 || ret != expected_err) {
            printf("ptsname_r: expected: return = %d (%s)\n",
                expected_err, strerror(expected_err));
            printf("           got: return = %d (%s)\n",
                ret, strerror(ret));
            return 1;
        }
    }

    return 0;
}

int main(void) {
    char buf[TTY_NAME_MAX] = {0};
    int result = 0;
    errno = 0;
    /* Tests with a real PTS master.  */
    int fd = posix_openpt(O_RDWR);
    if (fd == -1) {
        printf("posix_openpt (O_RDWR) failed\nerrno %d (%s)\n",
            errno, strerror(errno));
    }
    errno = 0;
    if (grantpt(fd) == -1) {
        printf("grantpt(%d) failed\nerrno %d (%s)\n", fd, errno, strerror(errno));
    }
    errno = 0;
    if (unlockpt(fd) == -1) {
        printf("unlockpt(%d) failed\nerrno %d (%s)\n", fd, errno, strerror(errno));
    }
    result |= do_single_test(fd, buf, sizeof(buf), 0);
    result |= do_single_test(fd, buf, 1, ERANGE);
    close(fd);

    // TODO: open(DEV_TTY, O_RDONLY) gives error on CI 
    // /* Test with a terminal device which is not a PTS master.  */
    // fd = open(DEV_TTY, O_RDONLY);
    // if (fd != -1) {
    //     result |= do_single_test(fd, buf, sizeof(buf), ENOTTY);
    //     close(fd);
    // } else
    //     printf("open (\"%s\", O_RDWR) failed\nerrno %d (%s)\n",
    //         DEV_TTY, errno, strerror(errno));

    /* Test with a file.  */
    fd = open(PTSNAME_EINVAL, O_RDWR | O_CREAT, 0600);
    grantpt(fd); // grantpt() is a no-op on Linux and Redox but is needed for POSIX compliance.
    errno = 0;
    if (unlockpt(fd) != -1 && errno != EINVAL) {
        printf("unlockpt(%d) did not return an expected errno\ngot errno %d (%s)\n", fd, errno, strerror(errno));
    }
    if (fd == -1) {
        printf("open (\"%s\", O_RDWR | OCREAT) failed\nerrno %d (%s)\n",
            PTSNAME_EINVAL, errno, strerror(errno));
    }
    result |= do_single_test(fd, buf, sizeof(buf), ENOTTY);
    close(fd);
    unlink(PTSNAME_EINVAL);

    return result;
}
