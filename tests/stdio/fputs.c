#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

int main(void) {
	FILE *f = fopen("stdio/fputs.out", "w");
	char *in = "Hello World!";
	fputs(in, f); // calls fwrite, helpers::fwritex, internal::to_write and internal::stdio_write
	fclose(f);
	return EXIT_SUCCESS;
}
