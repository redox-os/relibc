#include <stdio.h>
#include <setjmp.h>

int main() {
sigjmp_buf jb;

if (sigsetjmp(jb, 1)) {
printf("Jump done.\n");
} else {
printf ("Starting jump...\n");
siglongjmp(jb, 1);
}
return 0;
}
