#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char* string = (char*) calloc(20, sizeof(char));
    strcpy(string, "tempXXXXXX");
    mktemp(string);
    printf("%s\n",string);
    free(string);
}
