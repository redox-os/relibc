#include <wchar.h>
#include <assert.h>

int main() {
   // size is 17
   wchar_t* str1 = L"こんにちは世界Привет мир";
   assert(wcsnlen(str1, 10) == 10);
   assert(wcsnlen(str1, 100) == 17);
}