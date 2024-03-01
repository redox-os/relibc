#include <assert.h>
#include <crypt.h>
#include <string.h>
#include <unistd.h>

int main()
{
    char *expected_output = "$7$C6..../....$SodiumChloride$aAM7wxp7ayfEF.ZLedy2490m85oOR228oZTB7jPbmdG";
    char *result = crypt("pleaseletmein", "$7$C6..../....SodiumChloride");
    assert(strcmp(result, expected_output) == 0);

    // No salt
    result = crypt("pleaseletmein", "$7$C6..../....");
    assert(result != NULL);
    
    // Invalid encoded number for r
    result = crypt("pleaseletmein", "$7$C6.../....SodiumChloride");
    assert(result == NULL);
    
    // Invalid encoded number for p
    result = crypt("pleaseletmein", "$7$C6..../...SodiumChloride");
    assert(result == NULL);

    return 0;
}
