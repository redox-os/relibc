#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
	printf("%s\n", strchr("hello", 'e')); // should be ello
	printf("%s\n", strchr("world", 'l')); // should be ld
	printf("%i\n", strchr("world", 0) == NULL); // should be 1

    return 0;
}
