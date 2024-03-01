#include <assert.h>
#include <crypt.h>
#include <string.h>
#include <unistd.h>

int main()
{
    char *expected_output = "$2y$12$CJr4uRfziaGp4CWIBk0fB.I2tCOHYe3pomaWbC53/92p";
    char *result = crypt("correctbatteryhorsestapler", "$2y$12$L6Bc/AlTQHyd9liGgGEZyO");
    assert(strcmp(result, expected_output) == 0);

    // Invalid salt for Blowfish
    result = crypt("correctbatteryhorsestapler", "$2t$12$L6Bc/AlTQHyd9liGgGEZyO");
    assert(result == NULL);
    
    expected_output = "$2a$4$IAwt7hxuME3DekssMMTWU.xnJub2Xn45xK/oP.wWt3UC"; 
    result = crypt("password", "$2a$04$UuTkLRZZ6QofpDOlMz32Mu");
    assert(strcmp(result, expected_output) == 0);
    
    // Invalid salt for Blowfish
    result = "$2b$10$testtesttesttes";
    result = crypt("correctbatteryhorsestapler", "$2y$12$L6Bc/AlTQHyd9liGgGEZyO");
    assert(result != NULL);

    return 0;
}
