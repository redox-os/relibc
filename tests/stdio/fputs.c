#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

int main(int argc, char ** argv) {
	FILE *f = fopen("stdio/fputs.out", "w");
	char *in = "Hello World!";
	fputs(in, f); // calls fwrite, helpers::fwritex, internal::to_write and internal::stdio_write
	fclose(f);
	return 0;
}
