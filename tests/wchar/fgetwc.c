#include <stdio.h>
#include <wchar.h>
#include <locale.h>

int main(void)
{
    setlocale(LC_ALL, "");
    wint_t wc;
    FILE *fp = fopen("wchar/fgetwc.in", "r");
    if (!fp) {
        return 1;
    }

    while (WEOF != (wc = fgetwc(fp)))                      
      printf("%lc", wc);

    fclose(fp);
    return 0;
}
