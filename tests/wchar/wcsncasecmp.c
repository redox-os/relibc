#include <stdio.h>
#include <wchar.h>

int main() {
  wchar_t *s1 = L"This is string 1.";
  wchar_t *s2 = L"This is string 2.";
  printf("wcsncasecmp(s1, s1, 17) = %d\n", wcsncasecmp(s1, s1, 17));
  printf("wcsncasecmp(s1, s2, 17) = %d\n", wcsncasecmp(s1, s2, 17));
  printf("wcsncasecmp(s2, s1, 17) = %d\n", wcsncasecmp(s2, s1, 17));
  printf("wcsncasecmp(s2, s1, 15) = %d\n", wcsncasecmp(s2, s1, 15));
  printf("wcsncasecmp(s1, s2, 0) = %d\n", wcsncasecmp(s1, s2, 0));
}
