#include <assert.h>
#include <stdio.h>
#include <wchar.h>

int test_initial_orientation(void) {
	FILE *f = tmpfile();
	assert(fwide(f, 0) == 0);
	return 0;
}

int test_manual_byte_orientation(void) {
	FILE *f = tmpfile();

	// set manually to byte orientation
	assert(fwide(f, -483) == -1);

	// Cannot change to wchar orientation
	assert(fwide(f, 1) == -1);

	fclose(f);
	return 0;
}

int test_manual_wchar_orientation(void) {
	FILE *f = tmpfile();

	// set manually to wchar orientation
	assert(fwide(f, 483) == 1);

	// Cannot change to byte orientation
	assert(fwide(f, -1) == 1);

	fclose(f);
	return 0;
}

int test_orientation_after_fprintf(void) {
	// open file and write bytes; implicitly setting the bytes orientation
	FILE *f = tmpfile();
	assert(fprintf(f, "blah\n") == 5);

	// Check that bytes orientation is set
	assert(fwide(f, 0) == -1);

	fclose(f);
	return 0;
}

int main() {
	int(*tests[])(void) = {
		&test_initial_orientation,
		&test_manual_byte_orientation,
		&test_manual_wchar_orientation,
		&test_orientation_after_fprintf,
	};
	for(int i=0; i<sizeof(tests)/sizeof(int(*)(void)); i++) {
		printf("%d\n", (*tests[i])());
	}
}

