#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
    char source[] = "I'd just like to interject for a moment.  What you're referring to as Linux, "
                    "is in fact, GNU/Linux, or as I've recently taken to calling it, GNU plus Linux.\n";
    char* sp;

    char* token = strtok_r(source, " ", &sp);
    while (token) {
        printf("%s", token);
        if (token = strtok_r(NULL, " ", &sp)) {
            printf("_");
        }
    }

    return 0;
}
