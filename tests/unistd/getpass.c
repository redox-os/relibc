#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// #include "test_helpers.h"

int main(void)
{
    const char *pass = "pass";
    const char *prompt = "Enter password: ";

    char *result = getpass(prompt);

    if(strcmp(pass, result)) {
        printf("incorrect password\n");
        exit(EXIT_FAILURE);
    }

    const char *pass_127_chars = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    result = getpass(prompt);

    if(strcmp(pass_127_chars, result)) {
        printf("incorrect password\n");
        exit(EXIT_FAILURE);
    }

    const char *pass_empty = "";
    result = getpass(prompt);

    if(strcmp(pass_empty, result)) {
        printf("incorrect password\n");
        exit(EXIT_FAILURE);
    }

    printf("matching passwords\n", result);

    return 0;
}