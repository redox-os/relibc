#ifdef OPENLIBM_USE_HOST_FENV_H
#include <fenv.h>
#else /* !OPENLIBM_USE_HOST_FENV_H */

#if defined(__aarch64__)
#include <openlibm_fenv_aarch64.h>
#elif defined(__arm__)
#include <openlibm_fenv_arm.h>
#elif defined(__x86_64__)
#include <openlibm_fenv_amd64.h>
#elif defined(__i386__)
#include <openlibm_fenv_i387.h>
#elif defined(__powerpc__) || defined(__POWERPC__)
#include <openlibm_fenv_powerpc.h>
#elif defined(__mips__)
#include <openlibm_fenv_mips.h>
#elif defined(__s390__)
#include <openlibm_fenv_s390.h>
#elif defined(__riscv)
#include <openlibm_fenv_riscv.h>
#elif defined(__loongarch64)
#include <openlibm_fenv_loongarch64.h>
#else
#error "Unsupported platform"
#endif

#endif /* OPENLIBM_USE_HOST_FENV_H */
