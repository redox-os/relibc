#include <stdint.h>

uintptr_t __stack_chk_guard = 0xd048c37519fcadfe;

// manually define detailed abort function
void __abort(const char *func, const char *file, int line) __attribute__((noreturn));

__attribute__((noreturn))
void __stack_chk_fail(void) {
    // call detailed abort function
    __abort(__func__, __FILE__, __LINE__);
}
