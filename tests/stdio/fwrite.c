#include <stdio.h>
#include <stdlib.h>
#include <errno.h>

int main(void) {
	FILE *f = fopen("stdio/fwrite.out", "w");
	const char ptr[] = "Hello World!";

	if (fwrite(ptr, 0, 17, f)) {
		return -1;
	}

	if (fwrite(ptr, 7, 0, f)) {
		return -1;
	}

	if (fwrite(ptr, 0, 0, f)) {
		return -1;
	}

	fwrite(ptr, sizeof(ptr), 1, f);
	fclose(f);
	return 0;
}
