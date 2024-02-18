/* swscanf example */
#include <wchar.h>
#include <stdio.h>

#include "test_helpers.h"

int main ()
{
  wchar_t sentence [] = L"Michael 10";
  wchar_t str [20];
  int i;

  printf("Jelou\n");
  // wint_t status = swscanf (sentence,L"%*s %d",str,&i);
  wint_t status = swscanf (sentence,L"%s %d",str,&i);
  printf("%d\n", status);
  printf("%d\n", i);
  printf("%s\n", str);
  wprintf (L"%ls -> %d\n",str,i);
  ERROR_IF(swscanf, status, == WEOF);

  return 0;
}
