#include <assert.h>
#include <crypt.h>
#include <string.h>
#include <unistd.h>

int main()
{
    char *expected_output = "$8$3e8$salt$ZyTZgT5Pyp4CKdom6q6zHg0QrrAQO4ptWYCpWz4gu16";
    char *result = crypt("pleaseletmein", "$8$3e8$salt");
    assert(strcmp(result, expected_output) == 0);

    // No salt
    result = crypt("pleaseletmein", "$8$3e8$");
    assert(result != NULL);
    
    // Invalid encoded number for rounds
    result = crypt("pleaseletmein", "$8$$salt");
    assert(result == NULL);
    
    // Invalid encoded number for rounds
    result = crypt("pleaseletmein", "$8$.$salt");
    assert(result == NULL);

    return 0;
}
