#include <stdio.h>
#include <stdlib.h>

int main(int argc, char ** argv) {
	setvbuf(stdout, 0, _IONBF, 0);
	FILE *f = fopen("stdio/stdio.in", "r");
	setvbuf(f, 0, _IONBF, 0);
	printf("%c\n", fgetc(f));
	ungetc('H', f);
	char *in = malloc(30);
	printf("%s\n", fgets(in, 30, f));
	printf("Hello\n");
	return 0;
}
