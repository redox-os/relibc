#include <stdio.h>
#include <stdlib.h>

int main(void) {
	ungetc('h', stdin);
	char c;
	if ((c = getchar()) == 'h') {
		printf("Worked!\n");
		exit(EXIT_SUCCESS);
	}
	printf("failed :( %c\n", c);
	exit(EXIT_FAILURE);
}
