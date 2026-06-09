#include <assert.h>
#include <crypt.h>
#include <errno.h>
#include <stdio.h>
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

    // Non-UTF-8 key should succeed (treated as raw bytes)
    char key[] = {(char)0xC0, (char)0x01, '\0'};
    result = crypt(key, "$5$saltsalt$");
    assert(result != NULL);

    // Non-UTF-8 setting should return NULL with errno=EINVAL
    char setting[] = {'$', '5', '$', (char)0xFE, (char)0xFF, '$', '\0'};
    errno = 0;
    result = crypt("password", setting);
    assert(result == NULL);
    assert(errno == EINVAL);

    return 0;
}
