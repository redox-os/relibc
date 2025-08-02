#include <stdlib.h>
#include <syslog.h>

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
    syslog(LOG_EMERG, "This is a test message with formatting: %d", 5);

    // Only alert should print
    setlogmask(LOG_MASK(LOG_ALERT));
    syslog(LOG_ALERT, "Hank Hill");
    syslog(LOG_EMERG, "Sells propane and propane accessories");

    // Squelch all logs with a priority less than WARNING.
    setlogmask(LOG_UPTO(LOG_WARNING));
    // First line should be emitted while the second shouldn't.
    syslog(LOG_WARNING, "Foo has been bar'd");
    syslog(LOG_NOTICE, "I am a very spammy log message. Ha!");

    // All of these should print
    setlogmask(LOG_UPTO(LOG_DEBUG));
    syslog(LOG_DEBUG, "And now, Eeveelutions");
    syslog(LOG_DEBUG, "Espeon");
    syslog(LOG_INFO, "Umbreon");
    syslog(LOG_NOTICE, "Sylveon");
    syslog(LOG_WARNING, "Glaceon");
    syslog(LOG_ERR, "Leafeon");
    syslog(LOG_CRIT, "Vaporeon");
    syslog(LOG_ALERT, "Flareon");
    syslog(LOG_EMERG, "Jolteon");
    
    // The log file should automatically open even if closed.
    closelog();
    syslog(LOG_INFO, "Bye from relibc's syslog tests!");

    return EXIT_SUCCESS;
}
