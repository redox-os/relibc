#include <stdio.h>
#include <wchar.h>

int main() {
  wchar_t *s1 = L"ThIs Is StRiNg 1.";
  wchar_t *s2 = L"tHiS iS sTrInG 2.";
  printf("wcscasecmp(s1, s1) = %d\n", wcscasecmp(s1, s1));
  printf("wcscasecmp(s1, s2) = %d\n", wcscasecmp(s1, s2));
  printf("wcscasecmp(s2, s1) = %d\n", wcscasecmp(s2, s1));
  printf("wcscasecmp(s2, s2) = %d\n", wcscasecmp(s2, s2));
}
