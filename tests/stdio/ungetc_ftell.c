#include <stdio.h>
int main() {
	FILE *f = fopen("stdio/ungetc_ftell.c", "r");
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	ungetc('\n', f);ungetc('d', f);
	ungetc('l', f);	ungetc('r', f);
	ungetc('o', f);	ungetc('w', f);
	ungetc(' ', f);	ungetc('o', f);
	ungetc('l', f);	ungetc('l', f);
	ungetc('e', f);	ungetc('h', f);
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
	printf("%c, %ld\n", getc(f), ftell(f));
}
