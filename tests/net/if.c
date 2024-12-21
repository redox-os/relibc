#include <net/if.h>
#include <string.h>

#include "../test_helpers.h"

#define assert_eq(value, expected)                                             \
  {                                                                            \
    if (value != expected) {                                                   \
      fprintf(stderr, "%s:%d: failed\n", __FILE__, __LINE__);                  \
      exit(EXIT_FAILURE);                                                      \
    }                                                                          \
  }

int main(void) {
  const struct if_nameindex *list = if_nameindex();

  // Currently always returning a stub
  const struct if_nameindex *first = &(list[0]);
  assert_eq(first->if_index, 1);
  assert_eq(strcmp(first->if_name, "stub"), 0);

  // Last item with 0 values determines the end of the list
  const struct if_nameindex *second = &(list[1]);
  assert_eq(second->if_index, 0);
  assert_eq(second->if_name, 0);

  unsigned idx;
  idx = if_nametoindex(0);
  assert_eq(idx, 0);
  idx = if_nametoindex("any");
  assert_eq(idx, 0);
  idx = if_nametoindex("stub");
  assert_eq(idx, 1);

  const char *name;
  name = if_indextoname(0, 0);
  assert_eq(name, 0);

  name = if_indextoname(1, 0);
  assert_eq(strcmp(name, "stub"), 0);

  printf("OK\n");
}
