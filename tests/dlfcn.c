#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>


int add(int a, int b)
{
	return a + b;
}


int main()
{
	void* handle = dlopen(NULL, RTLD_LAZY);
	if (!handle) {
		printf("dlopen(NULL) failed\n");
		exit(1);
	}
	int (*f)(int, int) = dlsym(handle, "add");
	if (!f) {
		printf("dlsym(handle, add) failed\n");
		exit(2);
	}
	int a = 22;
	int b = 33;
	printf("add(%d, %d) = %d\n", a, b, f(a, b));
}

