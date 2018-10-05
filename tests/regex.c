#include <regex.h>
#include <stdio.h>

int main() {
    regex_t regex;
    char error_buf[256];

    int error = regcomp(&regex, "h.llo \\(w.rld\\)", REG_ICASE);
    if (error) {
        regerror(error, &regex, error_buf, 255);
        error_buf[255] = 0;
        printf("regcomp error: %d = %s\n", error, error_buf);
        return -1;
    }

    regmatch_t matches[3] = { 0 };

    error = regexec(&regex, "Hey, how are you? Hello? Hallo Wurld??", 3, matches, 0);

    regfree(&regex);

    if (error) {
        regerror(error, &regex, error_buf, 255);
        printf("regexec error: %d = %s\n", error, error_buf);
        return -1;
    }

    for (int group = 0; group < 3; group += 1) {
        printf("Matching group: %d - %d\n", matches[group].rm_so, matches[group].rm_eo);
    }
}
