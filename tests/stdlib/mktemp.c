#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "test_helpers.h"

int main(void) {
    char* string = (char*) calloc(20, sizeof(char));
    strcpy(string, "tempXXXXXX");
    #pragma GCC diagnostic push
    #pragma GCC diagnostic ignored "-Wdeprecated-declarations"
    mktemp(string);
    #pragma GCC diagnostic pop
    printf("%s\n",string);
    free(string);
}
