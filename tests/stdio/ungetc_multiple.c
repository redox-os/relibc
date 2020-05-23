#include <stdio.h>
int main() {
  ungetc('\n', stdin);
  ungetc('d', stdin);
  ungetc('l', stdin);
  ungetc('r', stdin);
  ungetc('o', stdin);
  ungetc('w', stdin);
  ungetc(' ', stdin);
  ungetc('o', stdin);
  ungetc('l', stdin);
  ungetc('l', stdin);
  ungetc('e', stdin);
  ungetc('h', stdin);
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
  putchar(getchar());
}
