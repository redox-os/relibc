#include <stdlib.h>
#include <stdio.h>

int main(int argc, char** argv) {
    printf("%d\n", rand());
    srand(259);
    printf("%d\n", rand());
}
