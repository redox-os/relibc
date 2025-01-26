#include <string.h>
#include <assert.h>


int main() {
    // Test 1: Character is present in the string
    const char *str1 = "Hello, World!";
    const char *result1 = strchrnul(str1, 'o');
    assert(result1 == &str1[4]); // 'o' is at position 4 in "Hello, World!"

    // Test 2: Character is not in the string (should return the null terminator)
    const char *str2 = "Hello, World!";
    const char *result2 = strchrnul(str2, 'z');
    assert(result2 == &str2[13]); // 'z' is not present, so it returns the null terminator

    // Test 3: Character is the first character in the string
    const char *str3 = "abcdef";
    const char *result3 = strchrnul(str3, 'a');
    assert(result3 == &str3[0]); // 'a' is at position 0

    // Test 4: Character is the last character in the string
    const char *str4 = "abcdef";
    const char *result4 = strchrnul(str4, 'f');
    assert(result4 == &str4[5]); // 'f' is at position 5, the last character

    // Test 5: Searching for the null terminator itself
    const char *str5 = "abcdef";
    const char *result5 = strchrnul(str5, '\0');
    assert(result5 == &str5[6]); // The null terminator is at position 6 (end of the string)
    return 0;
}
