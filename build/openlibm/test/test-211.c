#include <stdio.h>
#include <math.h>
#include <assert.h>

int
main()
{
  float x = 0xd.65874p-4f;
  float y = 4.0f;
  float z = powf (x, y);
  assert(z==0x1.f74424p-2);
}
