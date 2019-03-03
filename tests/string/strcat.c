#include <string.h>
#include <stdio.h>

#include "test_helpers.h"

int main(void) {
    char dest1[12] = "hello";
	  printf("%s\n", strcat(dest1, " world")); // should be hello world

    char dest2[12] = "hello";
  	printf("%s\n", strncat(dest2, " world foobar", 6)); // should be hello world
}
