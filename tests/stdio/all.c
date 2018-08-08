#include <stdio.h>
#include <stdlib.h>

int main(int argc, char ** argv) {
	FILE *f = fopen("stdio/stdio.in", "r");
	printf("%c\n", fgetc(f));
	ungetc('H', f);
	char *in = malloc(30);
	printf("%s\n", fgets(in, 30, f));
	setvbuf(stdout, 0, _IONBF, 0);
	printf("Hello\n");
	return 0;
}
