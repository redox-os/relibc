#include <wchar.h>
#include <wctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <locale.h>

int main()
{
   setlocale(LC_ALL, "");
   FILE *stream;
   wint_t wc;
   wint_t wc2;

   if (NULL == (stream = fopen("wchar/ungetwc.in", "r+")))
      return 1;
 
   while (WEOF != (wc = fgetwc(stream)) && iswdigit(wc)) {}
 
   if (WEOF != wc)
      ungetwc(wc, stream);
   
   wc2 = fgetwc(stream);
   assert(WEOF != wc2);
   assert(wc == wc2);
   
   return 0;
}