#include <string.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
  char s0[] = "hello, world";
  char* ptr = strrchr(s0, 'l');
  if (ptr != &s0[10]) {
    printf("%p != %p\n", ptr, &s0[10]);
    printf("strrchr FAIL , exit with status code %d\n", 1);
    exit(EXIT_FAILURE);
  }
  char s1[] = "";
  ptr = strrchr(s1, 'a');
  if (ptr != NULL) {
    printf("%p != 0\n", ptr);
    printf("strrchr FAIL, exit with status code %d\n", 1);
    exit(EXIT_FAILURE);
  }
  printf("strrch PASS, exiting with status code %d\n", 0);
}
