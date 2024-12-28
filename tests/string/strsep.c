#include <string.h>
#include <assert.h>

int main() {
    // Test case 1: Basic case with multiple tokens
    char str1[] = "apple,orange,banana";
    char *delim = ",";
    char *token = str1;
    char *result = NULL;

    // First token should be "apple"
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "apple") == 0);

    // Second token should be "orange"
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "orange") == 0);

    // Third token should be "banana"
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "banana") == 0);

    // No more tokens
    result = strsep(&token, delim);
    assert(result == NULL);

    // Test case 2: Empty string
    char str2[] = "";
    token = str2;
    result = strsep(&token, delim);

    // Test case 3: String with no delimiter
    char str3[] = "apple";
    token = str3;
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "apple") == 0);
    assert(token == NULL);  // No more tokens

    // Test case 4: String starts with delimiter
    char str4[] = ",apple,orange";
    token = str4;
    result = strsep(&token, delim);
    assert(result != NULL && strlen(result) == 0);  // First token should be an empty string ("")

    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "apple") == 0);

    // Test case 5: Multiple delimiters in a row
    char str5[] = "apple,,orange";
    token = str5;
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "apple") == 0);
    result = strsep(&token, delim);
    assert(result != NULL && strlen(result) == 0);  // Empty token due to consecutive delimiters
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "orange") == 0);

    // Test case 6: Delimiters at the end of the string
    char str6[] = "apple,orange,";
    token = str6;
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "apple") == 0);
    result = strsep(&token, delim);
    assert(result != NULL && strcmp(result, "orange") == 0);
    result = strsep(&token, delim);

    return 0;
}
