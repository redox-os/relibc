#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

int main(int argc, char ** argv) {
	FILE *f = fopen("stdio/fwrite.out", "w");
	char *in = "Hello World!";
	fputs(in, f); // calls fwrite, helpers::fwritex, internal::to_write and internal::stdio_write
	fclose(f);
	return 0;
}
