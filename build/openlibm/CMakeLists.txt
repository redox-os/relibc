cmake_minimum_required(VERSION 3.25)

# Get version string from Make.inc
file(READ "${CMAKE_CURRENT_LIST_DIR}/Make.inc" MAKE_FILE)
string(REGEX MATCH "VERSION = ([0-9\.]+)" _ ${MAKE_FILE})

project(openlibm
        VERSION ${CMAKE_MATCH_1}
        LANGUAGES C ASM)

option(BUILD_SHARED_LIBS "Build using shared libraries" ON)

add_library("${PROJECT_NAME}")

# Find the relevant folder depending on the architecture
set(OPENLIBM_ARCH_FOLDER ${CMAKE_SYSTEM_PROCESSOR})
string(TOLOWER "${OPENLIBM_ARCH_FOLDER}" OPENLIBM_ARCH_FOLDER)

if(${OPENLIBM_ARCH_FOLDER} STREQUAL "amd64" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "x86_64")
    set(OPENLIBM_ARCH_FOLDER "amd64")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "arm64" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "aarch64")
    set(OPENLIBM_ARCH_FOLDER "aarch64")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "armv7-a")
    set(OPENLIBM_ARCH_FOLDER "arm")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "x86" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "i686")
    set(OPENLIBM_ARCH_FOLDER "i387")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "powerpc")
    set(OPENLIBM_ARCH_FOLDER "powerpc")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "riscv64")
    set(OPENLIBM_ARCH_FOLDER "riscv64")
else()
    message(FATAL_ERROR "${PROJECT_NAME} not set up for detected architecture: ${OPENLIBM_ARCH_FOLDER}")
endif()


# Compile flags
list(APPEND C_ASM_COMPILE_FLAGS "-ffp-contract=off" "-fno-fast-math" "-fno-rounding-math" "-fno-math-errno")
list(APPEND C_ASM_COMPILE_FLAGS "-fPIC" "-std=c99" "-fno-builtin")
list(APPEND C_ASM_COMPILE_FLAGS "-Wall" "-Wno-implicit-function-declaration")
list(APPEND C_ASM_COMPILE_FLAGS "-DASSEMBLER" "-D__BSD_VISIBLE" "-O3")

# Compiler-specific compile flags
if("${CMAKE_C_COMPILER_ID}" MATCHES "Clang")
    list(APPEND C_ASM_COMPILE_FLAGS "-fno-strict-aliasing" "-ffp-exception-behavior=strict")
elseif("${CMAKE_C_COMPILER_ID}" STREQUAL "GNU")
    list(APPEND C_ASM_COMPILE_FLAGS "-fno-gnu89-inline")
else()
    message(FATAL_ERROR "${PROJECT_NAME} not set up to be compiled with ${CMAKE_C_COMPILER_ID}")
endif()

# Architecture-specific compile flags - take advantage of sse on x86
if(${OPENLIBM_ARCH_FOLDER} STREQUAL "i387")
    list(APPEND C_ASM_COMPILE_FLAGS "-march=i686" "-m32" "-msse2" "-mfpmath=sse")
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "amd64")
    list(APPEND C_ASM_COMPILE_FLAGS "-m64" "-msse2" "-mfpmath=sse")
endif()

# Suppress warnings if requested
if(OPENLIBM_SUPPRESS_WARNINGS)
    list(APPEND C_ASM_COMPILE_FLAGS "-w")
endif()

# Add compile flags
target_compile_options("${PROJECT_NAME}" PUBLIC ${C_ASM_COMPILE_FLAGS})

# Project Source
set(PROJECT_SRC "${CMAKE_CURRENT_SOURCE_DIR}")

# Common
list(APPEND OPENLIBM_C_SOURCE
        # src
        "${PROJECT_SRC}/src/common.c"
        "${PROJECT_SRC}/src/e_acos.c"
        "${PROJECT_SRC}/src/e_acosf.c"
        "${PROJECT_SRC}/src/e_acosh.c"
        "${PROJECT_SRC}/src/e_acoshf.c"
        "${PROJECT_SRC}/src/e_asin.c"
        "${PROJECT_SRC}/src/e_asinf.c"
        "${PROJECT_SRC}/src/e_atan2.c"
        "${PROJECT_SRC}/src/e_atan2f.c"
        "${PROJECT_SRC}/src/e_atanh.c"
        "${PROJECT_SRC}/src/e_atanhf.c"
        "${PROJECT_SRC}/src/e_cosh.c"
        "${PROJECT_SRC}/src/e_coshf.c"
        "${PROJECT_SRC}/src/e_exp.c"
        "${PROJECT_SRC}/src/e_expf.c"
        "${PROJECT_SRC}/src/e_fmod.c"
        "${PROJECT_SRC}/src/e_fmodf.c"
        "${PROJECT_SRC}/src/e_hypot.c"
        "${PROJECT_SRC}/src/e_hypotf.c"
        "${PROJECT_SRC}/src/e_j0.c"
        "${PROJECT_SRC}/src/e_j0f.c"
        "${PROJECT_SRC}/src/e_j1.c"
        "${PROJECT_SRC}/src/e_j1f.c"
        "${PROJECT_SRC}/src/e_jn.c"
        "${PROJECT_SRC}/src/e_jnf.c"
        "${PROJECT_SRC}/src/e_lgamma.c"
        "${PROJECT_SRC}/src/e_lgamma_r.c"
        "${PROJECT_SRC}/src/e_lgammaf.c"
        "${PROJECT_SRC}/src/e_lgammaf_r.c"
        "${PROJECT_SRC}/src/e_log.c"
        "${PROJECT_SRC}/src/e_log10.c"
        "${PROJECT_SRC}/src/e_log10f.c"
        "${PROJECT_SRC}/src/e_log2.c"
        "${PROJECT_SRC}/src/e_log2f.c"
        "${PROJECT_SRC}/src/e_logf.c"
        "${PROJECT_SRC}/src/e_pow.c"
        "${PROJECT_SRC}/src/e_powf.c"
        "${PROJECT_SRC}/src/e_remainder.c"
        "${PROJECT_SRC}/src/e_remainderf.c"
        "${PROJECT_SRC}/src/e_rem_pio2.c"
        "${PROJECT_SRC}/src/e_rem_pio2f.c"
        "${PROJECT_SRC}/src/e_sinh.c"
        "${PROJECT_SRC}/src/e_sinhf.c"
        "${PROJECT_SRC}/src/e_sqrt.c"
        "${PROJECT_SRC}/src/e_sqrtf.c"
        "${PROJECT_SRC}/src/k_cos.c"
        "${PROJECT_SRC}/src/k_exp.c"
        "${PROJECT_SRC}/src/k_expf.c"
        "${PROJECT_SRC}/src/k_rem_pio2.c"
        "${PROJECT_SRC}/src/k_sin.c"
        "${PROJECT_SRC}/src/k_tan.c"
        "${PROJECT_SRC}/src/k_cosf.c"
        "${PROJECT_SRC}/src/k_sinf.c"
        "${PROJECT_SRC}/src/k_tanf.c"
        "${PROJECT_SRC}/src/s_asinh.c"
        "${PROJECT_SRC}/src/s_asinhf.c"
        "${PROJECT_SRC}/src/s_atan.c"
        "${PROJECT_SRC}/src/s_atanf.c"
        "${PROJECT_SRC}/src/s_carg.c"
        "${PROJECT_SRC}/src/s_cargf.c"
        "${PROJECT_SRC}/src/s_cbrt.c"
        "${PROJECT_SRC}/src/s_cbrtf.c"
        "${PROJECT_SRC}/src/s_ceil.c"
        "${PROJECT_SRC}/src/s_ceilf.c"
        "${PROJECT_SRC}/src/s_copysign.c"
        "${PROJECT_SRC}/src/s_copysignf.c"
        "${PROJECT_SRC}/src/s_cos.c"
        "${PROJECT_SRC}/src/s_cosf.c"
        "${PROJECT_SRC}/src/s_csqrt.c"
        "${PROJECT_SRC}/src/s_csqrtf.c"
        "${PROJECT_SRC}/src/s_erf.c"
        "${PROJECT_SRC}/src/s_erff.c"
        "${PROJECT_SRC}/src/s_exp2.c"
        "${PROJECT_SRC}/src/s_exp2f.c"
        "${PROJECT_SRC}/src/s_expm1.c"
        "${PROJECT_SRC}/src/s_expm1f.c"
        "${PROJECT_SRC}/src/s_fabs.c"
        "${PROJECT_SRC}/src/s_fabsf.c"
        "${PROJECT_SRC}/src/s_fdim.c"
        "${PROJECT_SRC}/src/s_floor.c"
        "${PROJECT_SRC}/src/s_floorf.c"
        "${PROJECT_SRC}/src/s_fmax.c"
        "${PROJECT_SRC}/src/s_fmaxf.c"
        "${PROJECT_SRC}/src/s_fmin.c"
        "${PROJECT_SRC}/src/s_fminf.c"
        "${PROJECT_SRC}/src/s_fpclassify.c"
        "${PROJECT_SRC}/src/s_frexp.c"
        "${PROJECT_SRC}/src/s_frexpf.c"
        "${PROJECT_SRC}/src/s_ilogb.c"
        "${PROJECT_SRC}/src/s_ilogbf.c"
        "${PROJECT_SRC}/src/s_isinf.c"
        "${PROJECT_SRC}/src/s_isfinite.c"
        "${PROJECT_SRC}/src/s_isnormal.c"
        "${PROJECT_SRC}/src/s_isnan.c"
        "${PROJECT_SRC}/src/s_log1p.c"
        "${PROJECT_SRC}/src/s_log1pf.c"
        "${PROJECT_SRC}/src/s_logb.c"
        "${PROJECT_SRC}/src/s_logbf.c"
        "${PROJECT_SRC}/src/s_modf.c"
        "${PROJECT_SRC}/src/s_modff.c"
        "${PROJECT_SRC}/src/s_nextafter.c"
        "${PROJECT_SRC}/src/s_nextafterf.c"
        "${PROJECT_SRC}/src/s_nexttowardf.c"
        "${PROJECT_SRC}/src/s_remquo.c"
        "${PROJECT_SRC}/src/s_remquof.c"
        "${PROJECT_SRC}/src/s_rint.c"
        "${PROJECT_SRC}/src/s_rintf.c"
        "${PROJECT_SRC}/src/s_round.c"
        "${PROJECT_SRC}/src/s_roundf.c"
        "${PROJECT_SRC}/src/s_scalbln.c"
        "${PROJECT_SRC}/src/s_scalbn.c"
        "${PROJECT_SRC}/src/s_scalbnf.c"
        "${PROJECT_SRC}/src/s_signbit.c"
        "${PROJECT_SRC}/src/s_signgam.c"
        "${PROJECT_SRC}/src/s_sin.c"
        "${PROJECT_SRC}/src/s_sincos.c"
        "${PROJECT_SRC}/src/s_sinf.c"
        "${PROJECT_SRC}/src/s_sincosf.c"
        "${PROJECT_SRC}/src/s_tan.c"
        "${PROJECT_SRC}/src/s_tanf.c"
        "${PROJECT_SRC}/src/s_tanh.c"
        "${PROJECT_SRC}/src/s_tanhf.c"
        "${PROJECT_SRC}/src/s_tgammaf.c"
        "${PROJECT_SRC}/src/s_trunc.c"
        "${PROJECT_SRC}/src/s_truncf.c"
        "${PROJECT_SRC}/src/s_cpow.c"
        "${PROJECT_SRC}/src/s_cpowf.c"
        "${PROJECT_SRC}/src/w_cabs.c"
        "${PROJECT_SRC}/src/w_cabsf.c"

        "${PROJECT_SRC}/src/s_fma.c"
        "${PROJECT_SRC}/src/s_fmaf.c"
        "${PROJECT_SRC}/src/s_lrint.c"
        "${PROJECT_SRC}/src/s_lrintf.c"
        "${PROJECT_SRC}/src/s_lround.c"
        "${PROJECT_SRC}/src/s_lroundf.c"
        "${PROJECT_SRC}/src/s_llrint.c"
        "${PROJECT_SRC}/src/s_llrintf.c"
        "${PROJECT_SRC}/src/s_llround.c"
        "${PROJECT_SRC}/src/s_llroundf.c"
        "${PROJECT_SRC}/src/s_nearbyint.c"

        # C99 complex functions
        "${PROJECT_SRC}/src/s_ccosh.c"
        "${PROJECT_SRC}/src/s_ccoshf.c"
        "${PROJECT_SRC}/src/s_cexp.c"
        "${PROJECT_SRC}/src/s_cexpf.c"
	    "${PROJECT_SRC}/src/s_cimag.c"
        "${PROJECT_SRC}/src/s_cimagf.c"
        "${PROJECT_SRC}/src/s_conj.c"
        "${PROJECT_SRC}/src/s_conjf.c"
	    "${PROJECT_SRC}/src/s_cproj.c"
        "${PROJECT_SRC}/src/s_cprojf.c"
        "${PROJECT_SRC}/src/s_creal.c"
        "${PROJECT_SRC}/src/s_crealf.c"
	    "${PROJECT_SRC}/src/s_csinh.c"
        "${PROJECT_SRC}/src/s_csinhf.c"
        "${PROJECT_SRC}/src/s_ctanh.c"
        "${PROJECT_SRC}/src/s_ctanhf.c"
        "${PROJECT_SRC}/src/s_cacos.c"
        "${PROJECT_SRC}/src/s_cacosf.c"
        "${PROJECT_SRC}/src/s_cacosh.c"
        "${PROJECT_SRC}/src/s_cacoshf.c"
        "${PROJECT_SRC}/src/s_casin.c"
        "${PROJECT_SRC}/src/s_casinf.c"
        "${PROJECT_SRC}/src/s_casinh.c"
        "${PROJECT_SRC}/src/s_casinhf.c"
        "${PROJECT_SRC}/src/s_catan.c"
        "${PROJECT_SRC}/src/s_catanf.c"
        "${PROJECT_SRC}/src/s_catanh.c"
        "${PROJECT_SRC}/src/s_catanhf.c"
        "${PROJECT_SRC}/src/s_clog.c"
        "${PROJECT_SRC}/src/s_clogf.c"

        # bsdsrc
		"${PROJECT_SRC}/bsdsrc/b_exp.c"
		"${PROJECT_SRC}/bsdsrc/b_log.c"
		"${PROJECT_SRC}/bsdsrc/b_tgamma.c"
)

if(NOT ${CMAKE_SYSTEM_NAME} STREQUAL "Windows")
    list(APPEND OPENLIBM_C_SOURCE
            "${PROJECT_SRC}/src/s_nan.c"
    )
endif()

# Determine if long double and double are the same size
include(CheckCSourceCompiles)
check_c_source_compiles("
#include <float.h>
#if (LDBL_MANT_DIG == DBL_MANT_DIG)
#error \"long double and double are the same size\"
#endif
int main(void ) { return 0; }
" LONG_DOUBLE_NOT_DOUBLE)

# Add in long double functions for x86, x64 and aarch64
if(LONG_DOUBLE_NOT_DOUBLE)
    list(APPEND OPENLIBM_C_SOURCE
            "${PROJECT_SRC}/src/s_copysignl.c"
            "${PROJECT_SRC}/src/s_fabsl.c"
            "${PROJECT_SRC}/src/s_llrintl.c"
            "${PROJECT_SRC}/src/s_lrintl.c"
            "${PROJECT_SRC}/src/s_modfl.c"

            "${PROJECT_SRC}/src/e_acosl.c"
            "${PROJECT_SRC}/src/e_asinl.c"
            "${PROJECT_SRC}/src/e_atan2l.c"
            "${PROJECT_SRC}/src/e_fmodl.c"
	        "${PROJECT_SRC}/src/s_fmaxl.c"
            "${PROJECT_SRC}/src/s_fminl.c"
            "${PROJECT_SRC}/src/s_ilogbl.c"
	        "${PROJECT_SRC}/src/e_hypotl.c"
            "${PROJECT_SRC}/src/e_lgammal.c"
            "${PROJECT_SRC}/src/e_remainderl.c"
            "${PROJECT_SRC}/src/e_sqrtl.c"
	        "${PROJECT_SRC}/src/s_atanl.c"
            "${PROJECT_SRC}/src/s_ceill.c"
            "${PROJECT_SRC}/src/s_cosl.c"
            "${PROJECT_SRC}/src/s_cprojl.c"
	        "${PROJECT_SRC}/src/s_csqrtl.c"
            "${PROJECT_SRC}/src/s_floorl.c"
            "${PROJECT_SRC}/src/s_fmal.c"
	        "${PROJECT_SRC}/src/s_frexpl.c"
            "${PROJECT_SRC}/src/s_logbl.c"
            "${PROJECT_SRC}/src/s_nexttoward.c"
	        "${PROJECT_SRC}/src/s_remquol.c"
            "${PROJECT_SRC}/src/s_roundl.c"
            "${PROJECT_SRC}/src/s_lroundl.c"
            "${PROJECT_SRC}/src/s_llroundl.c"
	        "${PROJECT_SRC}/src/s_cpowl.c"
            "${PROJECT_SRC}/src/s_cargl.c"
	        "${PROJECT_SRC}/src/s_sinl.c"
            "${PROJECT_SRC}/src/s_sincosl.c"
            "${PROJECT_SRC}/src/s_tanl.c"
            "${PROJECT_SRC}/src/s_truncl.c"
            "${PROJECT_SRC}/src/w_cabsl.c"
	        "${PROJECT_SRC}/src/s_nextafterl.c"
            "${PROJECT_SRC}/src/s_rintl.c"
            "${PROJECT_SRC}/src/s_scalbnl.c"
            "${PROJECT_SRC}/src/polevll.c"
	        "${PROJECT_SRC}/src/s_casinl.c"
            "${PROJECT_SRC}/src/s_ctanl.c"
	        "${PROJECT_SRC}/src/s_cimagl.c"
            "${PROJECT_SRC}/src/s_conjl.c"
            "${PROJECT_SRC}/src/s_creall.c"
            "${PROJECT_SRC}/src/s_cacoshl.c"
            "${PROJECT_SRC}/src/s_catanhl.c"
            "${PROJECT_SRC}/src/s_casinhl.c"
	        "${PROJECT_SRC}/src/s_catanl.c"
            "${PROJECT_SRC}/src/s_csinl.c"
            "${PROJECT_SRC}/src/s_cacosl.c"
            "${PROJECT_SRC}/src/s_cexpl.c"
            "${PROJECT_SRC}/src/s_csinhl.c"
            "${PROJECT_SRC}/src/s_ccoshl.c"
	        "${PROJECT_SRC}/src/s_clogl.c"
            "${PROJECT_SRC}/src/s_ctanhl.c"
            "${PROJECT_SRC}/src/s_ccosl.c"
            "${PROJECT_SRC}/src/s_cbrtl.c"
    )
endif()

if (LONG_DOUBLE_NOT_DOUBLE)
    if (${OPENLIBM_ARCH_FOLDER} STREQUAL "i387" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "amd64")
        list(APPEND OPENLIBM_C_SOURCE
            # ld80
            "${PROJECT_SRC}/ld80/invtrig.c"
            "${PROJECT_SRC}/ld80/e_acoshl.c"
            "${PROJECT_SRC}/ld80/e_powl.c"
            "${PROJECT_SRC}/ld80/k_tanl.c"
            "${PROJECT_SRC}/ld80/s_exp2l.c"
            "${PROJECT_SRC}/ld80/e_atanhl.c"
            "${PROJECT_SRC}/ld80/e_lgammal_r.c"
            "${PROJECT_SRC}/ld80/e_sinhl.c"
            "${PROJECT_SRC}/ld80/s_asinhl.c"
            "${PROJECT_SRC}/ld80/s_expm1l.c"
            "${PROJECT_SRC}/ld80/e_coshl.c"
            "${PROJECT_SRC}/ld80/e_log10l.c"
            "${PROJECT_SRC}/ld80/e_tgammal.c"
            "${PROJECT_SRC}/ld80/e_expl.c"
            "${PROJECT_SRC}/ld80/e_log2l.c"
            "${PROJECT_SRC}/ld80/k_cosl.c"
            "${PROJECT_SRC}/ld80/s_log1pl.c"
            "${PROJECT_SRC}/ld80/s_tanhl.c"
            "${PROJECT_SRC}/ld80/e_logl.c"
            "${PROJECT_SRC}/ld80/k_sinl.c"
            "${PROJECT_SRC}/ld80/s_erfl.c"
        )

        if(NOT ${CMAKE_SYSTEM_NAME} STREQUAL "Windows")
            list(APPEND OPENLIBM_C_SOURCE
                "${PROJECT_SRC}/ld80/s_nanl.c"
            )
        endif()
    else()
        if(${OPENLIBM_ARCH_FOLDER} STREQUAL "aarch64")
            list(APPEND OPENLIBM_C_SOURCE
                # ld128
                "${PROJECT_SRC}/ld128/invtrig.c"
                "${PROJECT_SRC}/ld128/e_acoshl.c"
                "${PROJECT_SRC}/ld128/e_powl.c"
                "${PROJECT_SRC}/ld128/k_tanl.c"
                "${PROJECT_SRC}/ld128/s_exp2l.c"
                "${PROJECT_SRC}/ld128/e_atanhl.c"
                "${PROJECT_SRC}/ld128/e_lgammal_r.c"
                "${PROJECT_SRC}/ld128/e_sinhl.c"
                "${PROJECT_SRC}/ld128/s_asinhl.c"
                "${PROJECT_SRC}/ld128/s_expm1l.c"
                "${PROJECT_SRC}/ld128/e_coshl.c"
                "${PROJECT_SRC}/ld128/e_log10l.c"
                "${PROJECT_SRC}/ld128/e_tgammal.c"
                "${PROJECT_SRC}/ld128/e_expl.c"
                "${PROJECT_SRC}/ld128/e_log2l.c"
                "${PROJECT_SRC}/ld128/k_cosl.c"
                "${PROJECT_SRC}/ld128/s_log1pl.c"
                "${PROJECT_SRC}/ld128/s_tanhl.c"
                "${PROJECT_SRC}/ld128/e_logl.c"
                "${PROJECT_SRC}/ld128/k_sinl.c"
                "${PROJECT_SRC}/ld128/s_erfl.c"
            )

            if(NOT ${CMAKE_SYSTEM_NAME} STREQUAL "Windows")
                list(APPEND OPENLIBM_C_SOURCE
                    "${PROJECT_SRC}/ld128/s_nanl.c"
                )
            endif()
        endif()
    endif()
endif()

# Architecture-specific sources
if (${OPENLIBM_ARCH_FOLDER} STREQUAL "amd64")
    list(APPEND OPENLIBM_C_SOURCE
        "${PROJECT_SRC}/amd64/fenv.c"
    )

    list(APPEND OPENLIBM_ASM_SOURCE
        "${PROJECT_SRC}/amd64/e_remainder.S"
        "${PROJECT_SRC}/amd64/e_remainderf.S"
        "${PROJECT_SRC}/amd64/e_remainderl.S"
        "${PROJECT_SRC}/amd64/e_sqrt.S"
        "${PROJECT_SRC}/amd64/e_sqrtf.S"
        "${PROJECT_SRC}/amd64/e_sqrtl.S"
        "${PROJECT_SRC}/amd64/s_llrint.S"
        "${PROJECT_SRC}/amd64/s_llrintf.S"
        "${PROJECT_SRC}/amd64/s_llrintl.S"
        "${PROJECT_SRC}/amd64/s_logbl.S"
        "${PROJECT_SRC}/amd64/s_lrint.S"
        "${PROJECT_SRC}/amd64/s_lrintf.S"
        "${PROJECT_SRC}/amd64/s_lrintl.S"
        "${PROJECT_SRC}/amd64/s_remquo.S"
        "${PROJECT_SRC}/amd64/s_remquof.S"
        "${PROJECT_SRC}/amd64/s_remquol.S"
        "${PROJECT_SRC}/amd64/s_rintl.S"
        "${PROJECT_SRC}/amd64/s_scalbn.S"
        "${PROJECT_SRC}/amd64/s_scalbnf.S"
        "${PROJECT_SRC}/amd64/s_scalbnl.S"
        "${PROJECT_SRC}/amd64/e_fmod.S"
        "${PROJECT_SRC}/amd64/e_fmodf.S"
        "${PROJECT_SRC}/amd64/e_fmodl.S"
    )

elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "aarch64")
    list(APPEND OPENLIBM_C_SOURCE
            "${PROJECT_SRC}/aarch64/fenv.c"
    )

elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "arm")
    list(APPEND OPENLIBM_C_SOURCE
            "${PROJECT_SRC}/${OPENLIBM_ARCH_FOLDER}/fenv.c"
    )

elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "i387")
    list(APPEND OPENLIBM_C_SOURCE
            "${PROJECT_SRC}/i387/fenv.c"
    )

    list(APPEND OPENLIBM_ASM_SOURCE
            "${PROJECT_SRC}/i387/e_exp.S"
            "${PROJECT_SRC}/i387/e_fmod.S"
            "${PROJECT_SRC}/i387/e_log.S"
            "${PROJECT_SRC}/i387/e_log10.S"
            "${PROJECT_SRC}/i387/e_remainder.S"
            "${PROJECT_SRC}/i387/e_sqrt.S"
            "${PROJECT_SRC}/i387/s_ceil.S"
            "${PROJECT_SRC}/i387/s_copysign.S"
            "${PROJECT_SRC}/i387/s_floor.S"
            "${PROJECT_SRC}/i387/s_llrint.S"
            "${PROJECT_SRC}/i387/s_logb.S"
            "${PROJECT_SRC}/i387/s_lrint.S"
            "${PROJECT_SRC}/i387/s_remquo.S"
            "${PROJECT_SRC}/i387/s_rint.S"
            "${PROJECT_SRC}/i387/s_tan.S"
            "${PROJECT_SRC}/i387/s_trunc.S"

            # float counterparts
            "${PROJECT_SRC}/i387/e_log10f.S"
            "${PROJECT_SRC}/i387/e_logf.S"
            "${PROJECT_SRC}/i387/e_remainderf.S"
            "${PROJECT_SRC}/i387/e_sqrtf.S"
            "${PROJECT_SRC}/i387/s_ceilf.S"
            "${PROJECT_SRC}/i387/s_copysignf.S"
            "${PROJECT_SRC}/i387/s_floorf.S"
            "${PROJECT_SRC}/i387/s_llrintf.S"
            "${PROJECT_SRC}/i387/s_logbf.S"
            "${PROJECT_SRC}/i387/s_lrintf.S"
            "${PROJECT_SRC}/i387/s_remquof.S"
            "${PROJECT_SRC}/i387/s_rintf.S"
            "${PROJECT_SRC}/i387/s_truncf.S"

            # long double counterparts
            "${PROJECT_SRC}/i387/e_remainderl.S"
            "${PROJECT_SRC}/i387/e_sqrtl.S"
            "${PROJECT_SRC}/i387/s_ceill.S"
            "${PROJECT_SRC}/i387/s_copysignl.S"
            "${PROJECT_SRC}/i387/s_floorl.S"
            "${PROJECT_SRC}/i387/s_llrintl.S"
            "${PROJECT_SRC}/i387/s_logbl.S"
            "${PROJECT_SRC}/i387/s_lrintl.S"
            "${PROJECT_SRC}/i387/s_remquol.S"
            "${PROJECT_SRC}/i387/s_rintl.S"
            "${PROJECT_SRC}/i387/s_truncl.S"
    )

    if(NOT ${CMAKE_SYSTEM_NAME} STREQUAL "Windows")
        list(APPEND OPENLIBM_ASM_SOURCE
                "${PROJECT_SRC}/i387/s_scalbn.S"
                "${PROJECT_SRC}/i387/s_scalbnf.S"
                "${PROJECT_SRC}/i387/s_scalbnl.S"
        )
    endif()

elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "powerpc")
        list(APPEND OPENLIBM_C_SOURCE
                "${PROJECT_SRC}/powerpc/fenv.c"
        )
elseif(${OPENLIBM_ARCH_FOLDER} STREQUAL "riscv64")
        list(APPEND OPENLIBM_C_SOURCE
                "${PROJECT_SRC}/riscv64/fenv.c")
else()
	message(FATAL_ERROR "${PROJECT_NAME} CMake build is not set up for ${OPENLIBM_ARCH_FOLDER}")
endif()


# Filter out C implementation from compilation list if a native implementation exists
foreach(FILE_TO_REMOVE ${OPENLIBM_ASM_SOURCE})
    # Get filename and strip out extension
    cmake_path(GET FILE_TO_REMOVE FILENAME FILENAME_TO_REMOVE)
    cmake_path(REMOVE_EXTENSION FILENAME_TO_REMOVE OUTPUT_VARIABLE FILENAME_TO_REMOVE)
    message(DEBUG "Filename to remove: ${FILENAME_TO_REMOVE}")

    # Go through files and remove one with the same name
    foreach(CUR_FILE ${OPENLIBM_C_SOURCE})
        cmake_path(GET CUR_FILE FILENAME CUR_FILENAME)
        cmake_path(REMOVE_EXTENSION CUR_FILENAME OUTPUT_VARIABLE CUR_FILENAME)

        if(${CUR_FILENAME} STREQUAL ${FILENAME_TO_REMOVE})
            list(REMOVE_ITEM OPENLIBM_C_SOURCE ${CUR_FILE})
            message(DEBUG "Removed source file from compilation list: ${CUR_FILE}")
            break()
        endif()
    endforeach()
endforeach()


# Add sources
target_sources("${PROJECT_NAME}" PRIVATE ${OPENLIBM_C_SOURCE}
                                         ${OPENLIBM_ASM_SOURCE}
)


# Include directories
list(APPEND OPENLIBM_INCLUDE_DIRS
        "${PROJECT_SRC}"
        "${PROJECT_SRC}/include"
        "${PROJECT_SRC}/${OPENLIBM_ARCH_FOLDER}"
        "${PROJECT_SRC}/src"
)

if(${OPENLIBM_ARCH_FOLDER} STREQUAL "i387" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "amd64" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "powerpc")
    list(APPEND OPENLIBM_INCLUDE_DIRS "${PROJECT_SRC}/ld80")
else()
    if(${OPENLIBM_ARCH_FOLDER} STREQUAL "aarch64" OR ${OPENLIBM_ARCH_FOLDER} STREQUAL "riscv64")
        list(APPEND OPENLIBM_INCLUDE_DIRS "${PROJECT_SRC}/ld128")
    endif()
endif()

target_include_directories("${PROJECT_NAME}" PUBLIC ${OPENLIBM_INCLUDE_DIRS})

file(GLOB PUBLIC_HEADERS "*.h" "include/*.h" "${OPENLIBM_ARCH_FOLDER}/*.h" "src/*.h")
set_target_properties("${PROJECT_NAME}" PROPERTIES PUBLIC_HEADER "${PUBLIC_HEADERS}")
install (TARGETS "${PROJECT_NAME}")

# Can't use configure_file because openlibm.pc.in uses $var instead of CMake configure @var's
# Would rather string replace variables here instead of editing .pc.in, because editing .pc.in
# might build break autotools build.
file(READ "${PROJECT_SRC}/openlibm.pc.in" PC_FILE)
string(REPLACE "\${version}" "${CMAKE_PROJECT_VERSION}" PC_FILE ${PC_FILE})
string(PREPEND PC_FILE "prefix=${CMAKE_INSTALL_PREFIX}
includedir=\${prefix}/${CMAKE_INSTALL_INCLUDEDIR}
libdir=\${prefix}/${CMAKE_INSTALL_LIBDIR}\n
")
file(WRITE "${CMAKE_BINARY_DIR}/${PROJECT_NAME}.pc" ${PC_FILE})
install(FILES "${CMAKE_BINARY_DIR}/${PROJECT_NAME}.pc"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")
