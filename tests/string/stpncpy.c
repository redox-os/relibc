#include <string.h>
#include <assert.h>

int main() {
    // Define some test strings
    const char *src = "Hello, World!";
    char dest[50];
    
    // Test 1: Copy exactly 5 characters from src to dest
    char *result = stpncpy(dest, src, 5);
    dest[5] = '\0';  // Ensure the destination string is null-terminated
    assert(strcmp(dest, "Hello") == 0);  // Verify the string in dest
    assert(result == &dest[5]);  // Ensure the return pointer points to the null terminator

    // Test 2: Copy 15 characters from src to dest (more than the length of src)
    result = stpncpy(dest, src, 15);
    dest[13] = '\0';  // Ensure the destination string is null-terminated
    assert(strcmp(dest, "Hello, World!") == 0);  // Verify the string in dest
    assert(result == &dest[13]);  // Ensure the return pointer points to the null terminator

    // Test 3: Copy 3 characters from src to dest
    result = stpncpy(dest, src, 3);
    dest[3] = '\0';  // Ensure the destination string is null-terminated
    assert(strcmp(dest, "Hel") == 0);  // Verify the string in dest
    assert(result == &dest[3]);  // Ensure the return pointer points to the null terminator

    // Test 4: Copy 0 characters from src to dest
    result = stpncpy(dest, src, 0);
    dest[0] = '\0';  // Ensure the destination is explicitly null-terminated
    assert(dest[0] == '\0');  // Ensure the destination is an empty string
    assert(result == dest);  // Ensure the return pointer points to the start of dest

    // Test 5: Copy exactly the length of the source string
    result = stpncpy(dest, src, strlen(src));
    dest[strlen(src)] = '\0';  // Ensure the destination string is null-terminated
    assert(strcmp(dest, "Hello, World!") == 0);  // Verify the string in dest
    assert(result == &dest[strlen(src)]);  // Ensure the return pointer points to the null terminator

    return 0;
}
