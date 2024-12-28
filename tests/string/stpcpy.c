#include <string.h>
#include <assert.h>

void test_stpcpy(const char *src, char *dest) {
    char *end = stpcpy(dest, src);
    assert(strcmp(dest, src) == 0);
    assert(*end == '\0');
    assert(end == dest + strlen(dest));
}

int main() {
    char dest[20];

    test_stpcpy("Hello, World!", dest);

    // Test case 2: Empty string
    test_stpcpy("", dest);

    // Test case 3: String with special characters
    test_stpcpy("Special chars: !@#$%^&*()", dest);

    // Test case 4: String with spaces
    test_stpcpy("A string with spaces", dest);

    return 0;
}
