#include <assert.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <termios.h>
#include <unistd.h>

int main(void) {
    // Check that the tty supports VDISABLE
    int vdisable = pathconf("/dev/tty", _PC_VDISABLE);
    assert(vdisable == _POSIX_VDISABLE);

    int termfd = open("/dev/tty", O_RDWR, O_NDELAY | O_NOCTTY);
    if (termfd < 0) {
        perror("open");
        return EXIT_FAILURE;
    }

    // Currently set/default options
    struct termios termios = {0};
    if (tcgetattr(termfd, &termios) < 0) {
        perror("tcgetattr");
        close(termfd);
        return EXIT_FAILURE;
    }
    struct termios term_restore = termios;

    const cc_t verase = termios.c_cc[VERASE];
    assert(verase != _POSIX_VDISABLE);
    // Disable backspace key control char
    termios.c_cc[VERASE] = _POSIX_VDISABLE;

    if (tcsetattr(termfd, TCSANOW, &termios) < 0) {
        perror("tcsetattr (setting VDISABLE)");
        close(termfd);
        return EXIT_FAILURE;
    }

    // Check that it was actually set
    struct termios term_vdisable = {0};
    if (tcgetattr(termfd, &term_vdisable) < 0) {
        perror("tcgetattr (after setting VDISABLE)");
        close(termfd);
        return EXIT_FAILURE;
    }
    assert(term_vdisable.c_cc[VERASE] == _POSIX_VDISABLE);

    // Restore old config
    if (tcsetattr(termfd, TCSAFLUSH, &term_restore) < 0) {
        perror("tcsetattr (restoring settings)");
        close(termfd);
        return EXIT_FAILURE;
    }

    close(termfd);
    return EXIT_SUCCESS;
}
