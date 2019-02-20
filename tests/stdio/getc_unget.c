#include <stdio.h>

int main(void) {
	ungetc('h', stdin);
	char c;
	if ((c = getchar()) == 'h') {
		printf("Worked!\n");
		return 0;
	}
	printf("failed :( %c\n", c);
	return 0;
}
