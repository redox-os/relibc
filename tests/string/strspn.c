#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
	char *hello = "hello";
	char *world = "world";
	char *banana = "banana";
	printf("%lu\n", strspn(hello, "hello")); // should be 5
	printf("%lu\n", strspn(world, "wendy")); // should be 1
	printf("%lu\n", strspn(banana, "apple")); // should be 0
}
