/* swscanf example */
#include <wchar.h>
#include <stdio.h>

#include "test_helpers.h"

int main ()
{
  wchar_t sentence [] = L"10";
  // wchar_t str [20];
  int i;

  printf("Jelou");
  wint_t status = swscanf (sentence,L"%d",&i);
  printf("%d", status);
  wprintf (L"%d\n",i);
  ERROR_IF(swscanf, status, == WEOF);

  return 0;
}
