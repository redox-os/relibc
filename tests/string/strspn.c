#include <string.h>
#include <stdio.h>

int main(int argc, char* argv[]) {
	char *hello = "hello";
	char *world = "world";
	char *banana = "banana";
	printf("%lu\n", strspn(hello, "hello")); // should be 5
	printf("%lu\n", strspn(world, "wendy")); // should be 1
	printf("%lu\n", strspn(banana, "apple")); // should be 0

    return 0;
}
