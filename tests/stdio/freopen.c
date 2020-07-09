#include <assert.h>
#include <stdio.h>
#include <wchar.h>

#include "test_helpers.h"

int test_reopen_opens_file(void) {
    FILE *f = freopen("stdio/stdio.in", "r", stdin);
    ERROR_IF(freopen, f, == NULL);

    char in[6];
    fgets(in, 6, stdin);
    printf("%s\n", in); // should print Hello
    fclose(f);
    return 0;
}

int test_reopen_resets_orientation(void) {
    FILE *f = freopen("stdio/stdio.in", "r", stdin);
    assert(fwide(f, 0) == 0);
    assert(fwide(f, -1) == -1);

    f = freopen("stdio/stdio.in", "r", stdin);
    assert(fwide(f, 0) == 0);

    fclose(f);
    return 0;
}

int main(void) {
    int(*tests[])(void) = {
        &test_reopen_opens_file,
        &test_reopen_resets_orientation,
    };
    for(int i=0; i<sizeof(tests)/sizeof(int(*)(void)); i++) {
        printf("%d\n", (*tests[i])());
    }
}
