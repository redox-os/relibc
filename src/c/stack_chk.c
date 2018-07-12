#include <stdint.h>

void abort();

uintptr_t __stack_chk_guard = 0xd048c37519fcadfe;

__attribute__((noreturn))
void __stack_chk_fail(void) {
	abort();
}
