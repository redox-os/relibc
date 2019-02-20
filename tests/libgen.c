#include <libgen.h>
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

typedef struct {
  char * in;
  char * expected_out;
} test_case;

// API for basename and dirname allow the passed in string to
// be modified. This means we have to pass a modifiable copy.
char * get_mutable_string(char *str) {
  if (str == NULL)
    return NULL;
  char * copy = malloc(sizeof(char) * (strlen(str) + 1));
  copy = strcpy(copy, str);
  return copy;
}

void test_basename(void) {
  test_case test_cases[] =
  { {"/usr/lib", "lib"},
    {"//usr//lib//", "lib"},
    {"/usr/", "usr"},
    {"", "."},
    {"/", "/"},
    {"///","/"},
    {NULL, "."}
  };
  for (int i = 0;i < sizeof(test_cases)/sizeof(test_case);i++) {
    char * in = get_mutable_string(test_cases[i].in);
    char * out = basename(in);
    if (!out) {
      printf("Error on basename(%s), expected: '%s', got NULL\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out);
    } else if (strcmp(out, test_cases[i].expected_out) != 0) {
      printf("Error on basename(%s), expected: '%s', got: '%s'\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out, out); 
    } else {
      printf("OK on basename(%s), expected: '%s', got: '%s'\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out, out);
    }
    if (!in)
      free(in);
  }
  return;
}

void test_dirname(void) {
  test_case test_cases[] =
  { {"/usr/lib", "/usr"},
    {"//usr//lib//", "//usr"},
    {"/usr", "/"},
    {"usr", "."},
    {"/", "/"},
    {"///","/"},
    {".", "."},
    {"..", "."},
    {"", "."},
    {NULL, "."}
  };
  for (int i = 0;i < sizeof(test_cases)/sizeof(test_case);i++) {
    char * in = get_mutable_string(test_cases[i].in);
    char * out = dirname(in);
    if (!out) {
      printf("Error on dirname(%s), expected: '%s', got NULL\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out);
    } else if (strcmp(out, test_cases[i].expected_out) != 0) {
      printf("Error on dirname(%s), expected: '%s', got: '%s'\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out, out); 
    } else {
      printf("OK on dirname(%s), expected: '%s', got: '%s'\n", test_cases[i].in != 0 ? test_cases[i].in : "NULL", test_cases[i].expected_out, out);
    }
    if (!in)
      free(in);
  }
  return;
}

int main(void) {
  printf("Testing libgen.h\n");
  test_basename();
  test_dirname();
  return EXIT_SUCCESS;
}
