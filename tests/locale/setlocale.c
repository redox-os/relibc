#include <locale.h>
#include <stdio.h>
#include <limits.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char cwd[PATH_MAX] = {0};
    if (!getcwd(cwd, PATH_MAX - 8)) {
        perror("getcwd");
    }
    strcat(cwd, "/locale");
    setenv("LOCPATH", cwd, 1);
    char* val = setlocale(LC_ALL, NULL);
    if (strcmp(val, "C") != 0) {
        printf("locale was not C: %s\n", val);
        return 1;
    }

    val = setlocale(LC_ALL, "fr.in");
    if (val == NULL) {
        perror("setlocale");
        return 1;
    }

    char *endptr;
    double expected = 123.45;
    double result = strtod("123,45", &endptr);

    if (result == expected && *endptr == '\0') {
        // TODO: apply locales to printf
        printf("strtod: %.2f\n", result);
        return 0;
    } else {
        printf("strtod: expected %f, got %f\n", expected, result);
        return 1;
    }
}
