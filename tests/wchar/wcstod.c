#include <sys/types.h>
#include <wchar.h>

void attempt(wchar_t *s) {
    wchar_t *end;
    double result = wcstod(s, &end);
    printf("strtod(%lls) = (%f, %lls)\n", s, result, end);
}
int main() {
    attempt(L"1.2345wowzah");
    attempt(L"53");
    attempt(L"-254352.5...");
    attempt(L"   19.2 wat");
    attempt(L"365.24 29.53");
    attempt(L" 29.53");
}
