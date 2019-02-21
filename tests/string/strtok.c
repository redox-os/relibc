#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char source[] = "I'd just like to interject for a moment.  What you're referring to as Linux, "
                    "is in fact, GNU/Linux, or as I've recently taken to calling it, GNU plus Linux.\n";

    char* token = strtok(source, " ");
    while (token) {
        printf("%s", token);
        if ((token = strtok(NULL, " "))) {
            printf("_");
        }
    }
}
