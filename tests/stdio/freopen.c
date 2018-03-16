#include <stdio.h>

int main(int argc, char ** argv) {
	freopen("stdio/stdio.in", "r", stdin);
	char in[6];
	fgets(in, 6, stdin);
	printf("%s\n", in); // should print Hello
	return 0;
}
