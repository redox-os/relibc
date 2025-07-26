#include <syslog.h>
#include <stdlib.h>

int main(void) {
    // Test that syslog succeeds without explicitly calling openlog
    syslog(LOG_INFO, "Testing syslog; disregard");
    closelog();

    // The rest of the tests use LOG_PERROR so we get verifiable output on stderr
    openlog("relibc_test",
            LOG_CONS | LOG_PERROR | LOG_NDELAY,
            LOG_LOCAL2
    );

    // Basic
    syslog(LOG_EMERG, "This is a test message with extra: %d", 5);

    // (This isn't available in Redox yet)
    // Squelch all logs with a priority less than WARNING.
    // setlogmask(LOG_UPTO(LOG_WARNING));

    // First line should be emitted while the second shouldn't.
    syslog(LOG_WARNING, "Foo has been bar'd");
    syslog(LOG_NOTICE, "I am a very spammy log message. Ha!");

    // TODO: LOG_MASK after implemented

    return EXIT_SUCCESS;
}
