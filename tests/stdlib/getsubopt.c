#include "test_helpers.h"

int main(void) {
    char *const tokens[] = {
        "ro",
        "rw",
        "foo",
        "baz",
        NULL
    };

    // getsubopt modifies the string in-place
    char opt_str[] = "ro,foo=bar,bool,baz=,rw";
    char *options = opt_str;
    char *value = NULL;
    int idx;

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != 0);
    if (value != NULL) {
        printf("getsubopt failed: expected NULL value for 'ro', got '%s'\n", value);
        exit(EXIT_FAILURE);
    }

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != 2);
    if (value == NULL || strcmp(value, "bar") != 0) {
        printf("getsubopt failed: expected 'bar', got '%s'\n", value ? value : "NULL");
        exit(EXIT_FAILURE);
    }

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != -1);
    if (value == NULL || strcmp(value, "bool") != 0) {
        printf("getsubopt failed: expected 'bool' in value, got '%s'\n", value ? value : "NULL");
        exit(EXIT_FAILURE);
    }

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != 3);
    if (value == NULL || strcmp(value, "") != 0) {
        printf("getsubopt failed: expected empty string value, got '%s'\n", value ? value : "NULL");
        exit(EXIT_FAILURE);
    }

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != 1);
    if (value != NULL) {
        printf("getsubopt failed: expected NULL value for 'rw', got '%s'\n", value);
        exit(EXIT_FAILURE);
    }

    idx = getsubopt(&options, tokens, &value);
    UNEXP_IF(getsubopt, idx, != -1);
}
