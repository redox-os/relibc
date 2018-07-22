#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char ** argv) {
	printf("# mem #\n");
	char arr[100];
	memset(arr, 0, 100); // Compiler builtin, should work
	arr[50] = 1;
	if ((size_t)memchr((void *)arr, 1, 100) - (size_t)arr != 50) {
		printf("Incorrect memchr\n");
		exit(1);
	}
	printf("Correct memchr\n");
	char arr2[51];
	memset(arr2, 0, 51); // Compiler builtin, should work
	memccpy((void *)arr2, (void *)arr, 1, 100);
	if (arr[50] != 1) {
		printf("Incorrect memccpy\n");
		exit(1);
	}
	printf("Correct memccpy\n");
    return 0;
}
