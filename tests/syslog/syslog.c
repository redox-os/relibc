#include <syslog.h>

int main() {
    int extraarg = 5;
    openlog("testprog", LOG_PID, LOG_USER);
    syslog(LOG_EMERG, "This is a test message with extra: %d", extraarg);
    closelog();
    return 0;
}
