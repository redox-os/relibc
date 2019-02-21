#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
	printf("%s\n", strchr("hello", 'e')); // should be ello
	printf("%s\n", strchr("world", 'l')); // should be ld
	printf("%i\n", strchr("world", 0) == NULL); // should be 1
}
