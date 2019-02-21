#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
	char *world = "world";
	printf("%ld\n", strcspn("hello", world)); // should be 2
	printf("%ld\n", strcspn("banana", world)); // should be 6
}
