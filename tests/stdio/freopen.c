#include <stdio.h>
#include <stdlib.h>

int main(void) {
	freopen("stdio/stdio.in", "r", stdin);
	char in[6];
	fgets(in, 6, stdin);
	printf("%s\n", in); // should print Hello
	return EXIT_SUCCESS;
}
