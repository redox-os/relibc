#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
	printf("%s\n", strchr("hello", 'e')); // should be ello
	printf("%s\n", strchr("world", 'l')); // should be ld
	printf("%s\n", strchr("world", '\0')); // should be an empty, nul-terminated string
	printf("%p\n", strchr("world", 'x')); // should be a null pointer
}
