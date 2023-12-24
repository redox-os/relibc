/* Copyright (C) 1997, 1998, 1999, 2000, 2001 Free Software Foundation, Inc.
   This file is part of the GNU C Library.
   Contributed by Andreas Jaeger <aj@arthur.rhein-neckar.de>, 1997.

   The GNU C Library is free software; you can redistribute it and/or
   modify it under the terms of the GNU Lesser General Public
   License as published by the Free Software Foundation; either
   version 2.1 of the License, or (at your option) any later version.

   The GNU C Library is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public
   License along with the GNU C Library; if not, write to the Free
   Software Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA
   02111-1307 USA.  */

/* Part of testsuite for libm.

   This file is processed by a perl script.  The resulting file has to
   be included by a master file that defines:

   Makros:
   FUNC(function): converts general function name (like cos) to
   name with correct suffix (e.g. cosl or cosf)
   MATHCONST(x):   like FUNC but for constants (e.g convert 0.0 to 0.0L)
   FLOAT:	   floating point type to test
   - TEST_MSG:	   informal message to be displayed
   CHOOSE(Clongdouble,Cdouble,Cfloat,Cinlinelongdouble,Cinlinedouble,Cinlinefloat):
   chooses one of the parameters as delta for testing
   equality
   PRINTF_EXPR	   Floating point conversion specification to print a variable
   of type FLOAT with printf.  PRINTF_EXPR just contains
   the specifier, not the percent and width arguments,
   e.g. "f".
   PRINTF_XEXPR	   Like PRINTF_EXPR, but print in hexadecimal format.
   PRINTF_NEXPR Like PRINTF_EXPR, but print nice.  */

#include <complex.h>
#include <float.h>
#include <math.h>
#include <fenv.h>
#include <limits.h>
#include <errno.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#define BUILD_COMPLEX(real, imag) \
 __extension__ ({ __complex__ FLOAT __retval;					     \
    __real__ __retval = (real);					     \
    __imag__ __retval = (imag);					     \
    __retval; })

#define MANT_DIG CHOOSE((LDBL_MANT_DIG - 1), (DBL_MANT_DIG - 1), (FLT_MANT_DIG - 1), \
  (LDBL_MANT_DIG - 1), (DBL_MANT_DIG - 1), (FLT_MANT_DIG - 1))

/* Possible exceptions */
#define NO_EXCEPTION 0x0
#define INVALID_EXCEPTION 0x1
#define DIVIDE_BY_ZERO_EXCEPTION 0x2
/* The next flags signals that those exceptions are allowed but not required.   */
#define INVALID_EXCEPTION_OK 0x4
#define DIVIDE_BY_ZERO_EXCEPTION_OK 0x8
#define EXCEPTIONS_OK INVALID_EXCEPTION_OK + DIVIDE_BY_ZERO_EXCEPTION_OK
/* Some special test flags, passed togther with exceptions.  */
#define IGNORE_ZERO_INF_SIGN 0x10

/* Various constants (we must supply them precalculated for accuracy).  */
#define M_PI_6l			.52359877559829887307710723054658383L
#define M_E2l			7.389056098930650227230427460575008L
#define M_E3l			20.085536923187667740928529654581719L
#define M_2_SQRT_PIl		3.5449077018110320545963349666822903L	/* 2 sqrt (M_PIl)  */
#define M_SQRT_PIl		1.7724538509055160272981674833411451L	/* sqrt (M_PIl)  */
#define M_LOG_SQRT_PIl		0.57236494292470008707171367567652933L	/* log(sqrt(M_PIl))  */
#define M_LOG_2_SQRT_PIl	1.265512123484645396488945797134706L	/* log(2*sqrt(M_PIl))  */
#define M_PI_34l		(M_PIl - M_PI_4l)		/* 3*pi/4 */
#define M_PI_34_LOG10El		(M_PIl - M_PI_4l) * M_LOG10El
#define M_PI2_LOG10El		M_PI_2l * M_LOG10El
#define M_PI4_LOG10El		M_PI_4l * M_LOG10El
#define M_PI_LOG10El		M_PIl * M_LOG10El

#define M_El		2.7182818284590452353602874713526625L  /* e */
#define M_LOG2El	1.4426950408889634073599246810018922L  /* log_2 e */
#define M_LOG10El	0.4342944819032518276511289189166051L  /* log_10 e */
#define M_LN2l		0.6931471805599453094172321214581766L  /* log_e 2 */
#define M_LN10l	2.3025850929940456840179914546843642L  /* log_e 10 */
#define M_PIl		3.1415926535897932384626433832795029L  /* pi */
#define M_PI_2l	1.5707963267948966192313216916397514L  /* pi/2 */
#define M_PI_4l	0.7853981633974483096156608458198757L  /* pi/4 */
#define M_1_PIl	0.3183098861837906715377675267450287L  /* 1/pi */
#define M_2_PIl	0.6366197723675813430755350534900574L  /* 2/pi */
#define M_2_SQRTPIl	1.1283791670955125738961589031215452L  /* 2/sqrt(pi) */
#define M_SQRT2l	1.4142135623730950488016887242096981L  /* sqrt(2) */
#define M_SQRT1_2l	0.7071067811865475244008443621048490L  /* 1/sqrt(2) */

#define MAX_ULP 13

#define LOG 0

static int noErrors; /* number of errors */
static int noTests; /* number of tests (without testing exceptions) */
static int noExcTests; /* number of tests for exception flags */
static int noXFails; /* number of expected failures.  */
static int noXPasses; /* number of unexpected passes.  */

// static int ignore_max_ulp; /* Should we ignore max_ulp?  */

static FLOAT minus_zero, plus_zero;
static FLOAT plus_infty, minus_infty, nan_value;

static FLOAT max_error, real_max_error, imag_max_error;

/* Test whether a given exception was raised.  */
static int
test_single_exception(int exception,
  int exc_flag,
  int fe_flag) {
  int ok = 1;
  if (exception & exc_flag) {
    if (fetestexcept(fe_flag)) {
    } else {
      ok = 0;
    }
  } else {
    if (fetestexcept(fe_flag)) {
      ok = 0;
    }
  }
  if (!ok)
    ++noErrors;
  return ok;
}

static void
initialize(void) {
  plus_zero = 0.0;
  nan_value = plus_zero / plus_zero; /* Suppress GCC warning */

  minus_zero = FUNC(copysign)(0.0, -1.0);
  plus_infty = CHOOSE(HUGE_VALL, HUGE_VAL, HUGE_VALF,
    HUGE_VALL, HUGE_VAL, HUGE_VALF);
  minus_infty = CHOOSE(-HUGE_VALL, -HUGE_VAL, -HUGE_VALF,
    -HUGE_VALL, -HUGE_VAL, -HUGE_VALF);

  (void) & plus_zero;
  (void) & nan_value;
  (void) & minus_zero;
  (void) & plus_infty;
  (void) & minus_infty;

  /* Clear all exceptions.  From now on we must not get random exceptions.  */
  feclearexcept(FE_ALL_EXCEPT);
}

static void
init_max_error(void) {
  max_error = 0;
  real_max_error = 0;
  imag_max_error = 0;
  feclearexcept(FE_ALL_EXCEPT);
}

static void
set_max_error(FLOAT current, FLOAT * curr_max_error) {
  if (current > * curr_max_error)
    *
    curr_max_error = current;
}

/* Update statistic counters.  */
static void
update_stats (int ok, int xfail)
{
  ++noTests;
  if (ok && xfail)
    ++noXPasses;
  else if (!ok && xfail)
    ++noXFails;
  else if (!ok && !xfail)
    ++noErrors;
}

/* Test whether exceptions given by EXCEPTION are raised.  Ignore thereby
   allowed but not required exceptions.
*/
static void
test_exceptions(const char * test_name, int exception) {
  ++noExcTests;
  int err = 0;
  #ifdef FE_DIVBYZERO
  if ((exception & DIVIDE_BY_ZERO_EXCEPTION_OK) == 0)
    err = test_single_exception(exception, DIVIDE_BY_ZERO_EXCEPTION, FE_DIVBYZERO);
  #endif
  #ifdef FE_INVALID
  if ((exception & INVALID_EXCEPTION_OK) == 0)
    err = test_single_exception(exception, INVALID_EXCEPTION, FE_INVALID);
  #endif
  if (!err && LOG)
    printf("%s\n", test_name); 
  feclearexcept(FE_ALL_EXCEPT);
}

static void
check_float_internal(const char * test_name, FLOAT computed, FLOAT expected,
  FLOAT max_ulp, int xfail, int exceptions,
  FLOAT * curr_max_error) {
  int ok = 0;
  FLOAT diff = 0;
  FLOAT ulp = 0;

  test_exceptions(test_name, exceptions);
  if (isnan(computed) && isnan(expected))
    ok = 1;
  else if (isinf(computed) && isinf(expected)) {
    /* Test for sign of infinities.  */
    if ((exceptions & IGNORE_ZERO_INF_SIGN) == 0 &&
      signbit(computed) != signbit(expected)) {
      ok = 0;
      if (LOG)
        printf("infinity has wrong sign.\n");
    } else
      ok = 1;
  }
  /* Don't calc ulp for NaNs or infinities.  */
  else if (isinf(computed) || isnan(computed) || isinf(expected) || isnan(expected))
    ok = 0;
  else {
    diff = FUNC(fabs)(computed - expected);
    /* ilogb (0) isn't allowed.  */
    if (expected == 0.0)
      ulp = diff / FUNC(ldexp)(1.0, -MANT_DIG);
    else
      ulp = diff / FUNC(ldexp)(1.0, FUNC(ilogb)(expected) - MANT_DIG);
    set_max_error(ulp, curr_max_error);
    if ((exceptions & IGNORE_ZERO_INF_SIGN) == 0 &&
      computed == 0.0 && expected == 0.0 && signbit(computed) != signbit(expected))
      ok = 0;
    else if (ulp == 0.0 || (ulp <= max_ulp))
      ok = 1;
    else {
      ok = 0;
    }

  }

  if (!ok && LOG) {

	  printf ("Failure: %d %d\n ", (exceptions & IGNORE_ZERO_INF_SIGN) == 0, signbit(computed) != signbit(expected));
    printf ("Test: %s\n", test_name);
    printf ("Result:\n");
    printf (" is:         %f\n", computed);
    printf (" should be:  %f\n", expected);
    printf (" difference: %f\n", diff);
    printf (" ulp       : % .4" PRINTF_NEXPR "\n", ulp);
    printf (" max.ulp   : % .4" PRINTF_NEXPR "\n", max_ulp);
  }
  update_stats(ok, xfail);
}

static void
check_float(const char * test_name, FLOAT computed, FLOAT expected,
  FLOAT max_ulp, int xfail, int exceptions) {
  check_float_internal(test_name, computed, expected, max_ulp, xfail,
    exceptions, & max_error);
}

__extension__ static void
check_complex(const char * test_name,
  __complex__ FLOAT computed,
  __complex__ FLOAT expected,
  __complex__ FLOAT max_ulp, __complex__ int xfail,
  int exception) {
  FLOAT part_comp, part_exp, part_max_ulp;
  int part_xfail;

  part_comp = __real__ computed;
  part_exp = __real__ expected;
  part_max_ulp = __real__ max_ulp;
  part_xfail = __real__ xfail;

  check_float_internal(test_name, part_comp, part_exp, part_max_ulp, part_xfail,
    exception, & real_max_error);

  part_comp = __imag__ computed;
  part_exp = __imag__ expected;
  part_max_ulp = __imag__ max_ulp;
  part_xfail = __imag__ xfail;

  /* Don't check again for exceptions, just pass through the
     zero/inf sign test.  */
  check_float_internal(test_name, part_comp, part_exp, part_max_ulp, part_xfail,
    exception & IGNORE_ZERO_INF_SIGN, &
    imag_max_error);
}

/* Check that computed and expected values are equal (int values).  */
static void
check_int(const char * test_name, int computed, int expected, int max_ulp,
  int xfail, int exceptions) {
  int diff = computed - expected;
  int ok = 0;

  test_exceptions(test_name, exceptions);
  noTests++;
  if (abs(diff) <= max_ulp)
    ok = 1;
  update_stats(ok, xfail);
}

/* Check that computed and expected values are equal (long int values).  */
// static void
// check_long(const char * test_name, long int computed, long int expected,
//   long int max_ulp, int xfail, int exceptions) {
//   long int diff = computed - expected;
//   int ok = 0;

//   test_exceptions(test_name, exceptions);
//   noTests++;
//   if (labs(diff) <= max_ulp)
//     ok = 1;
//   update_stats(ok, xfail);
// }

// /* Check that computed value is true/false.  */
static void
check_bool(const char * test_name, int computed, int expected,
  int xfail, int exceptions) {
  int ok = 0;

  test_exceptions(test_name, exceptions);
  noTests++;
  if ((computed == 0) == (expected == 0))
    ok = 1;
  update_stats(ok, xfail);
}

// /* check that computed and expected values are equal (long int values) */
// static void
// check_longlong(const char * test_name, long long int computed,
//   long long int expected,
//   long long int max_ulp, int xfail,
//   int exceptions) {
//   long long int diff = computed - expected;
//   int ok = 0;

//   test_exceptions(test_name, exceptions);
//   noTests++;
//   if (llabs(diff) <= max_ulp)
//     ok = 1;
//   update_stats(ok, xfail);
// }

static void
fpclassify_test(void) {
  init_max_error();

  check_int("fpclassify (NaN) == FP_NAN", fpclassify(nan_value), FP_NAN, MAX_ULP, 0, 0);
  check_int("fpclassify (inf) == FP_INFINITE", fpclassify(plus_infty), FP_INFINITE, MAX_ULP, 0, 0);
  check_int("fpclassify (-inf) == FP_INFINITE", fpclassify(minus_infty), FP_INFINITE, MAX_ULP, 0, 0);
  check_int("fpclassify (+0) == FP_ZERO", fpclassify(plus_zero), FP_ZERO, MAX_ULP, 0, 0);
  check_int("fpclassify (-0) == FP_ZERO", fpclassify(minus_zero), FP_ZERO, MAX_ULP, 0, 0);
  check_int("fpclassify (1000) == FP_NORMAL", fpclassify(1000.0), FP_NORMAL, MAX_ULP, 0, 0);
}

static void
isfinite_test(void) {
  init_max_error();

  check_bool("isfinite (0) == true", isfinite(0.0), 1, 0, 0);
  check_bool("isfinite (-0) == true", isfinite(minus_zero), 1, 0, 0);
  check_bool("isfinite (10) == true", isfinite(10.0), 1, 0, 0);
  check_bool("isfinite (inf) == false", isfinite(plus_infty), MAX_ULP, 0, 0);
  check_bool("isfinite (-inf) == false", isfinite(minus_infty), MAX_ULP, 0, 0);
  check_bool("isfinite (NaN) == false", isfinite(nan_value), MAX_ULP, 0, 0);
}

static void
isnormal_test(void) {
  init_max_error();

  check_bool("isnormal (0) == false", isnormal(0.0), MAX_ULP, 0, 0);
  check_bool("isnormal (-0) == false", isnormal(minus_zero), MAX_ULP, 0, 0);
  check_bool("isnormal (10) == true", isnormal(10.0), 1, 0, 0);
  check_bool("isnormal (inf) == false", isnormal(plus_infty), MAX_ULP, 0, 0);
  check_bool("isnormal (-inf) == false", isnormal(minus_infty), MAX_ULP, 0, 0);
  check_bool("isnormal (NaN) == false", isnormal(nan_value), MAX_ULP, 0, 0);
}

static void
signbit_test(void) {

  init_max_error();

  check_bool("signbit (0) == false", signbit(0.0), MAX_ULP, 0, 0);
  check_bool("signbit (-0) == true", signbit(minus_zero), 1, 0, 0);
  check_bool("signbit (inf) == false", signbit(plus_infty), MAX_ULP, 0, 0);
  check_bool("signbit (-inf) == true", signbit(minus_infty), 1, 0, 0);

  /* signbit (x) != 0 for x < 0.  */
  check_bool("signbit (-1) == true", signbit(-1.0), 1, 0, 0);
  /* signbit (x) == 0 for x >= 0.  */
  check_bool("signbit (1) == false", signbit(1.0), MAX_ULP, 0, 0);
}

static void
cabs_test (void)
{
  errno = 0;
  FUNC(cabs) (BUILD_COMPLEX (0.7L, 12.4L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  /* cabs (x + iy) is specified as hypot (x,y) */

  /* cabs (+inf + i x) == +inf.  */
  check_float ("cabs (inf + 1.0 i) == inf",  FUNC(cabs) (BUILD_COMPLEX (plus_infty, 1.0)), plus_infty, MAX_ULP, 0, 0);
  /* cabs (-inf + i x) == +inf.  */
  check_float ("cabs (-inf + 1.0 i) == inf",  FUNC(cabs) (BUILD_COMPLEX (minus_infty, 1.0)), plus_infty, MAX_ULP, 0, 0);

  check_float ("cabs (-inf + NaN i) == inf",  FUNC(cabs) (BUILD_COMPLEX (minus_infty, nan_value)), plus_infty, MAX_ULP, 0, 0);
  check_float ("cabs (-inf + NaN i) == inf",  FUNC(cabs) (BUILD_COMPLEX (minus_infty, nan_value)), plus_infty, MAX_ULP, 0, 0);

  check_float ("cabs (NaN + NaN i) == NaN",  FUNC(cabs) (BUILD_COMPLEX (nan_value, nan_value)), nan_value, MAX_ULP, 0, 0);

  /* cabs (x,y) == cabs (y,x).  */
  check_float ("cabs (0.7 + 12.4 i) == 12.419742348374220601176836866763271",  FUNC(cabs) (BUILD_COMPLEX (0.7L, 12.4L)), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  /* cabs (x,y) == cabs (-x,y).  */
  check_float ("cabs (-12.4 + 0.7 i) == 12.419742348374220601176836866763271",  FUNC(cabs) (BUILD_COMPLEX (-12.4L, 0.7L)), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  /* cabs (x,y) == cabs (-y,x).  */
  check_float ("cabs (-0.7 + 12.4 i) == 12.419742348374220601176836866763271",  FUNC(cabs) (BUILD_COMPLEX (-0.7L, 12.4L)), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  /* cabs (x,y) == cabs (-x,-y).  */
  check_float ("cabs (-12.4 - 0.7 i) == 12.419742348374220601176836866763271",  FUNC(cabs) (BUILD_COMPLEX (-12.4L, -0.7L)), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  /* cabs (x,y) == cabs (-y,-x).  */
  check_float ("cabs (-0.7 - 12.4 i) == 12.419742348374220601176836866763271",  FUNC(cabs) (BUILD_COMPLEX (-0.7L, -12.4L)), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  /* cabs (x,0) == fabs (x).  */
  check_float ("cabs (-0.7 + 0 i) == 0.7",  FUNC(cabs) (BUILD_COMPLEX (-0.7L, 0)), 0.7L, MAX_ULP, 0, 0);
  check_float ("cabs (0.7 + 0 i) == 0.7",  FUNC(cabs) (BUILD_COMPLEX (0.7L, 0)), 0.7L, MAX_ULP, 0, 0);
  check_float ("cabs (-1.0 + 0 i) == 1.0",  FUNC(cabs) (BUILD_COMPLEX (-1.0L, 0)), 1.0L, MAX_ULP, 0, 0);
  check_float ("cabs (1.0 + 0 i) == 1.0",  FUNC(cabs) (BUILD_COMPLEX (1.0L, 0)), 1.0L, MAX_ULP, 0, 0);
  check_float ("cabs (-5.7e7 + 0 i) == 5.7e7",  FUNC(cabs) (BUILD_COMPLEX (-5.7e7L, 0)), 5.7e7L, MAX_ULP, 0, 0);
  check_float ("cabs (5.7e7 + 0 i) == 5.7e7",  FUNC(cabs) (BUILD_COMPLEX (5.7e7L, 0)), 5.7e7L, MAX_ULP, 0, 0);

  check_float ("cabs (0.7 + 1.2 i) == 1.3892443989449804508432547041028554",  FUNC(cabs) (BUILD_COMPLEX (0.7L, 1.2L)), 1.3892443989449804508432547041028554L, MAX_ULP, 0, 0);
}

static void
cacos_test (void)
{
  errno = 0;
  FUNC(cacos) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();


  check_complex ("cacos (0 + 0 i) == pi/2 - 0 i",  FUNC(cacos) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-0 + 0 i) == pi/2 - 0 i",  FUNC(cacos) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-0 - 0 i) == pi/2 + 0.0 i",  FUNC(cacos) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (M_PI_2l, 0.0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (0 - 0 i) == pi/2 + 0.0 i",  FUNC(cacos) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (M_PI_2l, 0.0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (-inf + inf i) == 3/4 pi - inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (M_PI_34l, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-inf - inf i) == 3/4 pi + inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (M_PI_34l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (inf + inf i) == pi/4 - inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (M_PI_4l, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (inf - inf i) == pi/4 + inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (M_PI_4l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (-10.0 + inf i) == pi/2 - inf i",  FUNC(cacos) (BUILD_COMPLEX (-10.0, plus_infty)), BUILD_COMPLEX (M_PI_2l, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-10.0 - inf i) == pi/2 + inf i",  FUNC(cacos) (BUILD_COMPLEX (-10.0, minus_infty)), BUILD_COMPLEX (M_PI_2l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (0 + inf i) == pi/2 - inf i",  FUNC(cacos) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (M_PI_2l, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (0 - inf i) == pi/2 + inf i",  FUNC(cacos) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (M_PI_2l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (0.1 + inf i) == pi/2 - inf i",  FUNC(cacos) (BUILD_COMPLEX (0.1L, plus_infty)), BUILD_COMPLEX (M_PI_2l, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (0.1 - inf i) == pi/2 + inf i",  FUNC(cacos) (BUILD_COMPLEX (0.1L, minus_infty)), BUILD_COMPLEX (M_PI_2l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (-inf + 0 i) == pi - inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (M_PIl, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-inf - 0 i) == pi + inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (M_PIl, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-inf + 100 i) == pi - inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, 100)), BUILD_COMPLEX (M_PIl, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-inf - 100 i) == pi + inf i",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, -100)), BUILD_COMPLEX (M_PIl, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (inf + 0 i) == 0.0 - inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (inf - 0 i) == 0.0 + inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (inf + 0.5 i) == 0.0 - inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, 0.5)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (inf - 0.5 i) == 0.0 + inf i",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, -0.5)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (inf + NaN i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(cacos) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-inf + NaN i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(cacos) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (0 + NaN i) == pi/2 + NaN i",  FUNC(cacos) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (M_PI_2l, nan_value), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-0 + NaN i) == pi/2 + NaN i",  FUNC(cacos) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (M_PI_2l, nan_value), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (NaN + inf i) == NaN - inf i",  FUNC(cacos) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, minus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (NaN - inf i) == NaN + inf i",  FUNC(cacos) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacos) (BUILD_COMPLEX (10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("cacos (-10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacos) (BUILD_COMPLEX (-10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("cacos (NaN + 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacos) (BUILD_COMPLEX (nan_value, 0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("cacos (NaN - 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacos) (BUILD_COMPLEX (nan_value, -0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("cacos (NaN + NaN i) == NaN + NaN i",  FUNC(cacos) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cacos (0.7 + 1.2 i) == 1.1351827477151551088992008271819053 - 1.0927647857577371459105272080819308 i",  FUNC(cacos) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.1351827477151551088992008271819053L, -1.0927647857577371459105272080819308L), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacos (-2 - 3 i) == 2.1414491111159960199416055713254211 + 1.9833870299165354323470769028940395 i",  FUNC(cacos) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (2.1414491111159960199416055713254211L, 1.9833870299165354323470769028940395L), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
}

static void
cacosh_test (void)
{
  errno = 0;
  FUNC(cacosh) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();


  check_complex ("cacosh (0 + 0 i) == 0.0 + pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (-0 + 0 i) == 0.0 + pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (0 - 0 i) == 0.0 - pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacosh (-0 - 0 i) == 0.0 - pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacosh (-inf + inf i) == inf + 3/4 pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_34l), MAX_ULP, 0, 0);
  check_complex ("cacosh (-inf - inf i) == inf - 3/4 pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_34l), MAX_ULP, 0, 0);

  check_complex ("cacosh (inf + inf i) == inf + pi/4 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_4l), MAX_ULP, 0, 0);
  check_complex ("cacosh (inf - inf i) == inf - pi/4 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_4l), MAX_ULP, 0, 0);

  check_complex ("cacosh (-10.0 + inf i) == inf + pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (-10.0, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (-10.0 - inf i) == inf - pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (-10.0, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (0 + inf i) == inf + pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (0 - inf i) == inf - pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (0.1 + inf i) == inf + pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0.1L, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("cacosh (0.1 - inf i) == inf - pi/2 i",  FUNC(cacosh) (BUILD_COMPLEX (0.1L, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("cacosh (-inf + 0 i) == inf + pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (plus_infty, M_PIl), MAX_ULP, 0, 0);
  check_complex ("cacosh (-inf - 0 i) == inf - pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, -M_PIl), MAX_ULP, 0, 0);
  check_complex ("cacosh (-inf + 100 i) == inf + pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, 100)), BUILD_COMPLEX (plus_infty, M_PIl), MAX_ULP, 0, 0);
  check_complex ("cacosh (-inf - 100 i) == inf - pi i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, -100)), BUILD_COMPLEX (plus_infty, -M_PIl), MAX_ULP, 0, 0);

  check_complex ("cacosh (inf + 0 i) == inf + 0.0 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("cacosh (inf - 0 i) == inf - 0 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cacosh (inf + 0.5 i) == inf + 0.0 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, 0.5)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cacosh (inf - 0.5 i) == inf - 0 i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, -0.5)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("cacosh (inf + NaN i) == inf + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("cacosh (-inf + NaN i) == inf + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("cacosh (0 + NaN i) == NaN + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
  check_complex ("cacosh (-0 + NaN i) == NaN + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("cacosh (NaN + inf i) == inf + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("cacosh (NaN - inf i) == inf + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("cacosh (10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacosh) (BUILD_COMPLEX (10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("cacosh (-10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacosh) (BUILD_COMPLEX (-10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("cacosh (NaN + 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacosh) (BUILD_COMPLEX (nan_value, 0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("cacosh (NaN - 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cacosh) (BUILD_COMPLEX (nan_value, -0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("cacosh (NaN + NaN i) == NaN + NaN i",  FUNC(cacosh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("cacosh (0.7 + 1.2 i) == 1.0927647857577371459105272080819308 + 1.1351827477151551088992008271819053 i",  FUNC(cacosh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.0927647857577371459105272080819308L, 1.1351827477151551088992008271819053L), MAX_ULP, 0, 0);
  check_complex ("cacosh (-2 - 3 i) == -1.9833870299165354323470769028940395 + 2.1414491111159960199416055713254211 i",  FUNC(cacosh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-1.9833870299165354323470769028940395L, 2.1414491111159960199416055713254211L), MAX_ULP, 0, 0);
}

static void
carg_test (void)
{
  init_max_error ();

  /* carg (x + iy) is specified as atan2 (y, x) */

  /* carg (x + i 0) == 0 for x > 0.  */
  check_float ("carg (2.0 + 0 i) == 0",  FUNC(carg) (BUILD_COMPLEX (2.0, 0)), 0, 0, 0, 0);
  /* carg (x - i 0) == -0 for x > 0.  */
  check_float ("carg (2.0 - 0 i) == -0",  FUNC(carg) (BUILD_COMPLEX (2.0, minus_zero)), minus_zero, 0, 0, 0);

  check_float ("carg (0 + 0 i) == 0",  FUNC(carg) (BUILD_COMPLEX (0, 0)), 0, 0, 0, 0);
  check_float ("carg (0 - 0 i) == -0",  FUNC(carg) (BUILD_COMPLEX (0, minus_zero)), minus_zero, 0, 0, 0);

  /* carg (x + i 0) == +pi for x < 0.  */
  check_float ("carg (-2.0 + 0 i) == pi",  FUNC(carg) (BUILD_COMPLEX (-2.0, 0)), M_PIl, 0, 0, 0);

  /* carg (x - i 0) == -pi for x < 0.  */
  check_float ("carg (-2.0 - 0 i) == -pi",  FUNC(carg) (BUILD_COMPLEX (-2.0, minus_zero)), -M_PIl, 0, 0, 0);

  check_float ("carg (-0 + 0 i) == pi",  FUNC(carg) (BUILD_COMPLEX (minus_zero, 0)), M_PIl, 0, 0, 0);
  check_float ("carg (-0 - 0 i) == -pi",  FUNC(carg) (BUILD_COMPLEX (minus_zero, minus_zero)), -M_PIl, 0, 0, 0);

  /* carg (+0 + i y) == pi/2 for y > 0.  */
  check_float ("carg (0 + 2.0 i) == pi/2",  FUNC(carg) (BUILD_COMPLEX (0, 2.0)), M_PI_2l, 0, 0, 0);

  /* carg (-0 + i y) == pi/2 for y > 0.  */
  check_float ("carg (-0 + 2.0 i) == pi/2",  FUNC(carg) (BUILD_COMPLEX (minus_zero, 2.0)), M_PI_2l, 0, 0, 0);

  /* carg (+0 + i y) == -pi/2 for y < 0.  */
  check_float ("carg (0 - 2.0 i) == -pi/2",  FUNC(carg) (BUILD_COMPLEX (0, -2.0)), -M_PI_2l, 0, 0, 0);

  /* carg (-0 + i y) == -pi/2 for y < 0.  */
  check_float ("carg (-0 - 2.0 i) == -pi/2",  FUNC(carg) (BUILD_COMPLEX (minus_zero, -2.0)), -M_PI_2l, 0, 0, 0);

  /* carg (inf + i y) == +0 for finite y > 0.  */
  check_float ("carg (inf + 2.0 i) == 0",  FUNC(carg) (BUILD_COMPLEX (plus_infty, 2.0)), 0, 0, 0, 0);

  /* carg (inf + i y) == -0 for finite y < 0.  */
  check_float ("carg (inf - 2.0 i) == -0",  FUNC(carg) (BUILD_COMPLEX (plus_infty, -2.0)), minus_zero, 0, 0, 0);

  /* carg(x + i inf) == pi/2 for finite x.  */
  check_float ("carg (10.0 + inf i) == pi/2",  FUNC(carg) (BUILD_COMPLEX (10.0, plus_infty)), M_PI_2l, 0, 0, 0);

  /* carg(x - i inf) == -pi/2 for finite x.  */
  check_float ("carg (10.0 - inf i) == -pi/2",  FUNC(carg) (BUILD_COMPLEX (10.0, minus_infty)), -M_PI_2l, 0, 0, 0);

  /* carg (-inf + i y) == +pi for finite y > 0.  */
  check_float ("carg (-inf + 10.0 i) == pi",  FUNC(carg) (BUILD_COMPLEX (minus_infty, 10.0)), M_PIl, 0, 0, 0);

  /* carg (-inf + i y) == -pi for finite y < 0.  */
  check_float ("carg (-inf - 10.0 i) == -pi",  FUNC(carg) (BUILD_COMPLEX (minus_infty, -10.0)), -M_PIl, 0, 0, 0);

  check_float ("carg (inf + inf i) == pi/4",  FUNC(carg) (BUILD_COMPLEX (plus_infty, plus_infty)), M_PI_4l, 0, 0, 0);

  check_float ("carg (inf - inf i) == -pi/4",  FUNC(carg) (BUILD_COMPLEX (plus_infty, minus_infty)), -M_PI_4l, 0, 0, 0);

  check_float ("carg (-inf + inf i) == 3 * M_PI_4l",  FUNC(carg) (BUILD_COMPLEX (minus_infty, plus_infty)), 3 * M_PI_4l, 0, 0, 0);

  check_float ("carg (-inf - inf i) == -3 * M_PI_4l",  FUNC(carg) (BUILD_COMPLEX (minus_infty, minus_infty)), -3 * M_PI_4l, 0, 0, 0);

  check_float ("carg (NaN + NaN i) == NaN",  FUNC(carg) (BUILD_COMPLEX (nan_value, nan_value)), nan_value, 0, 0, 0);
}

static void
casin_test (void)
{
  errno = 0;
  FUNC(casin) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("casin (0 + 0 i) == 0.0 + 0.0 i",  FUNC(casin) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casin (-0 + 0 i) == -0 + 0.0 i",  FUNC(casin) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casin (0 - 0 i) == 0.0 - 0 i",  FUNC(casin) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("casin (-0 - 0 i) == -0 - 0 i",  FUNC(casin) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);

  check_complex ("casin (inf + inf i) == pi/4 + inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (M_PI_4l, plus_infty), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casin (inf - inf i) == pi/4 - inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (M_PI_4l, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-inf + inf i) == -pi/4 + inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (-M_PI_4l, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-inf - inf i) == -pi/4 - inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (-M_PI_4l, minus_infty), MAX_ULP, 0, 0);

  check_complex ("casin (-10.0 + inf i) == -0 + inf i",  FUNC(casin) (BUILD_COMPLEX (-10.0, plus_infty)), BUILD_COMPLEX (minus_zero, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-10.0 - inf i) == -0 - inf i",  FUNC(casin) (BUILD_COMPLEX (-10.0, minus_infty)), BUILD_COMPLEX (minus_zero, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (0 + inf i) == 0.0 + inf i",  FUNC(casin) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (0 - inf i) == 0.0 - inf i",  FUNC(casin) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-0 + inf i) == -0 + inf i",  FUNC(casin) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (minus_zero, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-0 - inf i) == -0 - inf i",  FUNC(casin) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (minus_zero, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (0.1 + inf i) == 0.0 + inf i",  FUNC(casin) (BUILD_COMPLEX (0.1L, plus_infty)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (0.1 - inf i) == 0.0 - inf i",  FUNC(casin) (BUILD_COMPLEX (0.1L, minus_infty)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, 0);

  check_complex ("casin (-inf + 0 i) == -pi/2 + inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (-M_PI_2l, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-inf - 0 i) == -pi/2 - inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (-M_PI_2l, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-inf + 100 i) == -pi/2 + inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, 100)), BUILD_COMPLEX (-M_PI_2l, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (-inf - 100 i) == -pi/2 - inf i",  FUNC(casin) (BUILD_COMPLEX (minus_infty, -100)), BUILD_COMPLEX (-M_PI_2l, minus_infty), MAX_ULP, 0, 0);

  check_complex ("casin (inf + 0 i) == pi/2 + inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (M_PI_2l, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (inf - 0 i) == pi/2 - inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (M_PI_2l, minus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (inf + 0.5 i) == pi/2 + inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, 0.5)), BUILD_COMPLEX (M_PI_2l, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (inf - 0.5 i) == pi/2 - inf i",  FUNC(casin) (BUILD_COMPLEX (plus_infty, -0.5)), BUILD_COMPLEX (M_PI_2l, minus_infty), MAX_ULP, 0, 0);

  check_complex ("casin (NaN + inf i) == NaN + inf i",  FUNC(casin) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, plus_infty), MAX_ULP, 0, 0);
  check_complex ("casin (NaN - inf i) == NaN - inf i",  FUNC(casin) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, minus_infty), MAX_ULP, 0, 0);

  check_complex ("casin (0.0 + NaN i) == 0.0 + NaN i",  FUNC(casin) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (0.0, nan_value), MAX_ULP, 0, 0);
  check_complex ("casin (-0 + NaN i) == -0 + NaN i",  FUNC(casin) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (minus_zero, nan_value), MAX_ULP, 0, 0);

  check_complex ("casin (inf + NaN i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(casin) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casin (-inf + NaN i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(casin) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("casin (NaN + 10.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(casin) (BUILD_COMPLEX (nan_value, 10.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("casin (NaN - 10.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(casin) (BUILD_COMPLEX (nan_value, -10.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("casin (0.75 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(casin) (BUILD_COMPLEX (0.75, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("casin (-0.75 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(casin) (BUILD_COMPLEX (-0.75, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("casin (NaN + NaN i) == NaN + NaN i",  FUNC(casin) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("casin (0.7 + 1.2 i) == 0.4356135790797415103321208644578462 + 1.0927647857577371459105272080819308 i",  FUNC(casin) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.4356135790797415103321208644578462L, 1.0927647857577371459105272080819308L), MAX_ULP, 0, 0);
  check_complex ("casin (-2 - 3 i) == -0.57065278432109940071028387968566963 - 1.9833870299165354323470769028940395 i",  FUNC(casin) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-0.57065278432109940071028387968566963L, -1.9833870299165354323470769028940395L), MAX_ULP, 0, 0);
}


static void
casinh_test (void)
{
  errno = 0;
  FUNC(casinh) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("casinh (0 + 0 i) == 0.0 + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (-0 + 0 i) == -0 + 0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0), MAX_ULP, 0, 0);
  check_complex ("casinh (0 - 0 i) == 0.0 - 0 i",  FUNC(casinh) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casinh (-0 - 0 i) == -0 - 0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("casinh (inf + inf i) == inf + pi/4 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_4l), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casinh (inf - inf i) == inf - pi/4 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_4l), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf + inf i) == -inf + pi/4 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (minus_infty, M_PI_4l), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf - inf i) == -inf - pi/4 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (minus_infty, -M_PI_4l), MAX_ULP, 0, 0);

  check_complex ("casinh (-10.0 + inf i) == -inf + pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (-10.0, plus_infty)), BUILD_COMPLEX (minus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (-10.0 - inf i) == -inf - pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (-10.0, minus_infty)), BUILD_COMPLEX (minus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (0 + inf i) == inf + pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (0 - inf i) == inf - pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (-0 + inf i) == -inf + pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (minus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (-0 - inf i) == -inf - pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (minus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (0.1 + inf i) == inf + pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (0.1L, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("casinh (0.1 - inf i) == inf - pi/2 i",  FUNC(casinh) (BUILD_COMPLEX (0.1L, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("casinh (-inf + 0 i) == -inf + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (minus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf - 0 i) == -inf - 0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (minus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf + 100 i) == -inf + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, 100)), BUILD_COMPLEX (minus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf - 100 i) == -inf - 0 i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, -100)), BUILD_COMPLEX (minus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("casinh (inf + 0 i) == inf + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (inf - 0 i) == inf - 0 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("casinh (inf + 0.5 i) == inf + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, 0.5)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (inf - 0.5 i) == inf - 0 i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, -0.5)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("casinh (inf + NaN i) == inf + NaN i",  FUNC(casinh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("casinh (-inf + NaN i) == -inf + NaN i",  FUNC(casinh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (minus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("casinh (NaN + 0 i) == NaN + 0.0 i",  FUNC(casinh) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, 0.0), MAX_ULP, 0, 0);
  check_complex ("casinh (NaN - 0 i) == NaN - 0 i",  FUNC(casinh) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, minus_zero), MAX_ULP, 0, 0);

  check_complex ("casinh (NaN + inf i) == inf + NaN i plus sign of zero/inf not specified",  FUNC(casinh) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("casinh (NaN - inf i) == inf + NaN i plus sign of zero/inf not specified",  FUNC(casinh) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("casinh (10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(casinh) (BUILD_COMPLEX (10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("casinh (-10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(casinh) (BUILD_COMPLEX (-10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("casinh (NaN + 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(casinh) (BUILD_COMPLEX (nan_value, 0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("casinh (-0.75 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(casinh) (BUILD_COMPLEX (-0.75, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("casinh (NaN + NaN i) == NaN + NaN i",  FUNC(casinh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("casinh (0.7 + 1.2 i) == 0.97865459559367387689317593222160964 + 0.91135418953156011567903546856170941 i",  FUNC(casinh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.97865459559367387689317593222160964L, 0.91135418953156011567903546856170941L), MAX_ULP, 0, 0);
  check_complex ("casinh (-2 - 3 i) == -1.9686379257930962917886650952454982 - 0.96465850440760279204541105949953237 i",  FUNC(casinh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-1.9686379257930962917886650952454982L, -0.96465850440760279204541105949953237L), MAX_ULP, 0, 0);
}


static void
catan_test (void)
{
  errno = 0;
  FUNC(catan) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("catan (0 + 0 i) == 0 + 0 i",  FUNC(catan) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-0 + 0 i) == -0 + 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0), MAX_ULP, 0, 0);
  check_complex ("catan (0 - 0 i) == 0 - 0 i",  FUNC(catan) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catan (-0 - 0 i) == -0 - 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("catan (inf + inf i) == pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (M_PI_2l, 0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catan (inf - inf i) == pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (-inf + inf i) == -pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (-M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-inf - inf i) == -pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (-M_PI_2l, minus_zero), MAX_ULP, 0, 0);


  check_complex ("catan (inf - 10.0 i) == pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, -10.0)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (-inf - 10.0 i) == -pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, -10.0)), BUILD_COMPLEX (-M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (inf - 0 i) == pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (-inf - 0 i) == -pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (-M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (inf + 0.0 i) == pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, 0.0)), BUILD_COMPLEX (M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-inf + 0.0 i) == -pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, 0.0)), BUILD_COMPLEX (-M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (inf + 0.1 i) == pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (plus_infty, 0.1L)), BUILD_COMPLEX (M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-inf + 0.1 i) == -pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_infty, 0.1L)), BUILD_COMPLEX (-M_PI_2l, 0), MAX_ULP, 0, 0);

  check_complex ("catan (0.0 - inf i) == pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (-0 - inf i) == -pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (-M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (100.0 - inf i) == pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (100.0, minus_infty)), BUILD_COMPLEX (M_PI_2l, minus_zero), MAX_ULP, 0, 0);
  check_complex ("catan (-100.0 - inf i) == -pi/2 - 0 i",  FUNC(catan) (BUILD_COMPLEX (-100.0, minus_infty)), BUILD_COMPLEX (-M_PI_2l, minus_zero), MAX_ULP, 0, 0);

  check_complex ("catan (0.0 + inf i) == pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-0 + inf i) == -pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (-M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (0.5 + inf i) == pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (0.5, plus_infty)), BUILD_COMPLEX (M_PI_2l, 0), MAX_ULP, 0, 0);
  check_complex ("catan (-0.5 + inf i) == -pi/2 + 0 i",  FUNC(catan) (BUILD_COMPLEX (-0.5, plus_infty)), BUILD_COMPLEX (-M_PI_2l, 0), MAX_ULP, 0, 0);

  check_complex ("catan (NaN + 0.0 i) == NaN + 0 i",  FUNC(catan) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0), MAX_ULP, 0, 0);
  check_complex ("catan (NaN - 0 i) == NaN - 0 i",  FUNC(catan) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, minus_zero), MAX_ULP, 0, 0);

  check_complex ("catan (NaN + inf i) == NaN + 0 i",  FUNC(catan) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, 0), MAX_ULP, 0, 0);
  check_complex ("catan (NaN - inf i) == NaN - 0 i",  FUNC(catan) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, minus_zero), MAX_ULP, 0, 0);

  check_complex ("catan (0.0 + NaN i) == NaN + NaN i",  FUNC(catan) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
  check_complex ("catan (-0 + NaN i) == NaN + NaN i",  FUNC(catan) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("catan (inf + NaN i) == pi/2 + 0 i plus sign of zero/inf not specified",  FUNC(catan) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (M_PI_2l, 0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catan (-inf + NaN i) == -pi/2 + 0 i plus sign of zero/inf not specified",  FUNC(catan) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (-M_PI_2l, 0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("catan (NaN + 10.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(catan) (BUILD_COMPLEX (nan_value, 10.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("catan (NaN - 10.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(catan) (BUILD_COMPLEX (nan_value, -10.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("catan (0.75 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(catan) (BUILD_COMPLEX (0.75, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("catan (-0.75 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(catan) (BUILD_COMPLEX (-0.75, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("catan (NaN + NaN i) == NaN + NaN i",  FUNC(catan) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("catan (0.7 + 1.2 i) == 1.0785743834118921877443707996386368 + 0.57705737765343067644394541889341712 i",  FUNC(catan) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.0785743834118921877443707996386368L, 0.57705737765343067644394541889341712L), MAX_ULP, 0, 0);

  check_complex ("catan (-2 - 3 i) == -1.4099210495965755225306193844604208 - 0.22907268296853876629588180294200276 i",  FUNC(catan) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-1.4099210495965755225306193844604208L, -0.22907268296853876629588180294200276L), MAX_ULP, 0, 0);
}

static void
catanh_test (void)
{
  errno = 0;
  FUNC(catanh) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("catanh (0 + 0 i) == 0.0 + 0.0 i",  FUNC(catanh) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("catanh (-0 + 0 i) == -0 + 0.0 i",  FUNC(catanh) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("catanh (0 - 0 i) == 0.0 - 0 i",  FUNC(catanh) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catanh (-0 - 0 i) == -0 - 0 i",  FUNC(catanh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("catanh (inf + inf i) == 0.0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catanh (inf - inf i) == 0.0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf + inf i) == -0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (minus_zero, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf - inf i) == -0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (minus_zero, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("catanh (-10.0 + inf i) == -0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (-10.0, plus_infty)), BUILD_COMPLEX (minus_zero, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-10.0 - inf i) == -0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (-10.0, minus_infty)), BUILD_COMPLEX (minus_zero, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-0 + inf i) == -0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (minus_zero, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-0 - inf i) == -0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (minus_zero, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (0 + inf i) == 0.0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (0 - inf i) == 0.0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (0.1 + inf i) == 0.0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (0.1L, plus_infty)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (0.1 - inf i) == 0.0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (0.1L, minus_infty)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("catanh (-inf + 0 i) == -0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (minus_zero, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf - 0 i) == -0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (minus_zero, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf + 100 i) == -0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, 100)), BUILD_COMPLEX (minus_zero, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf - 100 i) == -0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, -100)), BUILD_COMPLEX (minus_zero, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("catanh (inf + 0 i) == 0.0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (inf - 0 i) == 0.0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (inf + 0.5 i) == 0.0 + pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, 0.5)), BUILD_COMPLEX (0.0, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("catanh (inf - 0.5 i) == 0.0 - pi/2 i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, -0.5)), BUILD_COMPLEX (0.0, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("catanh (0 + NaN i) == 0.0 + NaN i",  FUNC(catanh) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (0.0, nan_value), MAX_ULP, 0, 0);
  check_complex ("catanh (-0 + NaN i) == -0 + NaN i",  FUNC(catanh) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (minus_zero, nan_value), MAX_ULP, 0, 0);

  check_complex ("catanh (inf + NaN i) == 0.0 + NaN i",  FUNC(catanh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (0.0, nan_value), MAX_ULP, 0, 0);
  check_complex ("catanh (-inf + NaN i) == -0 + NaN i",  FUNC(catanh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (minus_zero, nan_value), MAX_ULP, 0, 0);

  check_complex ("catanh (NaN + 0 i) == NaN + NaN i",  FUNC(catanh) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
  check_complex ("catanh (NaN - 0 i) == NaN + NaN i",  FUNC(catanh) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("catanh (NaN + inf i) == 0.0 + pi/2 i plus sign of zero/inf not specified",  FUNC(catanh) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (0.0, M_PI_2l), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("catanh (NaN - inf i) == 0.0 - pi/2 i plus sign of zero/inf not specified",  FUNC(catanh) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (0.0, -M_PI_2l), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("catanh (10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(catanh) (BUILD_COMPLEX (10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("catanh (-10.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(catanh) (BUILD_COMPLEX (-10.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("catanh (NaN + 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(catanh) (BUILD_COMPLEX (nan_value, 0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("catanh (NaN - 0.75 i) == NaN + NaN i plus invalid exception allowed",  FUNC(catanh) (BUILD_COMPLEX (nan_value, -0.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("catanh (NaN + NaN i) == NaN + NaN i",  FUNC(catanh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("catanh (0.7 + 1.2 i) == 0.2600749516525135959200648705635915 + 0.97024030779509898497385130162655963 i",  FUNC(catanh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.2600749516525135959200648705635915L, 0.97024030779509898497385130162655963L), MAX_ULP, 0, 0);
  check_complex ("catanh (-2 - 3 i) == -0.14694666622552975204743278515471595 - 1.3389725222944935611241935759091443 i",  FUNC(catanh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-0.14694666622552975204743278515471595L, -1.3389725222944935611241935759091443L), MAX_ULP, 0, 0);
}

static void
ccos_test (void)
{
  errno = 0;
  FUNC(ccos) (BUILD_COMPLEX (0, 0));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("ccos (0.0 + 0.0 i) == 1.0 - 0 i",  FUNC(ccos) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccos (-0 + 0.0 i) == 1.0 + 0.0 i",  FUNC(ccos) (BUILD_COMPLEX (minus_zero, 0.0)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccos (0.0 - 0 i) == 1.0 + 0.0 i",  FUNC(ccos) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccos (-0 - 0 i) == 1.0 - 0 i",  FUNC(ccos) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ccos (inf + 0.0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccos (inf - 0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccos (-inf + 0.0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccos (-inf - 0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("ccos (0.0 + inf i) == inf - 0 i",  FUNC(ccos) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccos (0.0 - inf i) == inf + 0.0 i",  FUNC(ccos) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccos (-0 + inf i) == inf + 0.0 i",  FUNC(ccos) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccos (-0 - inf i) == inf - 0 i",  FUNC(ccos) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ccos (inf + inf i) == inf + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (-inf + inf i) == inf + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (inf - inf i) == inf + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (-inf - inf i) == inf + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ccos (4.625 + inf i) == -inf + inf i",  FUNC(ccos) (BUILD_COMPLEX (4.625, plus_infty)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("ccos (4.625 - inf i) == -inf - inf i",  FUNC(ccos) (BUILD_COMPLEX (4.625, minus_infty)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("ccos (-4.625 + inf i) == -inf - inf i",  FUNC(ccos) (BUILD_COMPLEX (-4.625, plus_infty)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("ccos (-4.625 - inf i) == -inf + inf i",  FUNC(ccos) (BUILD_COMPLEX (-4.625, minus_infty)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);

  check_complex ("ccos (inf + 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, 6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (inf - 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, -6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (-inf + 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, 6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccos (-inf - 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, -6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ccos (NaN + 0.0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ccos (NaN - 0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ccos (NaN + inf i) == inf + NaN i",  FUNC(ccos) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("ccos (NaN - inf i) == inf + NaN i",  FUNC(ccos) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("ccos (NaN + 9.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (nan_value, 9.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccos (NaN - 9.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (nan_value, -9.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccos (0.0 + NaN i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ccos (-0 + NaN i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccos) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ccos (10.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (10.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccos (-10.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (-10.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccos (inf + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccos (-inf + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccos) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccos (NaN + NaN i) == NaN + NaN i",  FUNC(ccos) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("ccos (0.7 + 1.2 i) == 1.3848657645312111080 - 0.97242170335830028619 i",  FUNC(ccos) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.3848657645312111080L, -0.97242170335830028619L), MAX_ULP, 0, 0);

  check_complex ("ccos (-2 - 3 i) == -4.1896256909688072301 - 9.1092278937553365979 i",  FUNC(ccos) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-4.1896256909688072301L, -9.1092278937553365979L), MAX_ULP, 0, 0);
}


static void
ccosh_test (void)
{
  errno = 0;
  FUNC(ccosh) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("ccosh (0.0 + 0.0 i) == 1.0 + 0.0 i",  FUNC(ccosh) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccosh (-0 + 0.0 i) == 1.0 - 0 i",  FUNC(ccosh) (BUILD_COMPLEX (minus_zero, 0.0)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccosh (0.0 - 0 i) == 1.0 - 0 i",  FUNC(ccosh) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccosh (-0 - 0 i) == 1.0 + 0.0 i",  FUNC(ccosh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);

  check_complex ("ccosh (0.0 + inf i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (nan_value, 0.0), 0, MAX_ULP, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccosh (-0 + inf i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (nan_value, 0.0), 0, MAX_ULP, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccosh (0.0 - inf i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (nan_value, 0.0), 0, MAX_ULP, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("ccosh (-0 - inf i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (nan_value, 0.0), 0, MAX_ULP, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("ccosh (inf + 0.0 i) == inf + 0.0 i",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, 0.0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("ccosh (-inf + 0.0 i) == inf - 0 i",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, 0.0)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccosh (inf - 0 i) == inf - 0 i",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ccosh (-inf - 0 i) == inf + 0.0 i",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);

  check_complex ("ccosh (inf + inf i) == inf + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (-inf + inf i) == inf + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (inf - inf i) == inf + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (-inf - inf i) == inf + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ccosh (inf + 4.625 i) == -inf - inf i",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, 4.625)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("ccosh (-inf + 4.625 i) == -inf + inf i",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, 4.625)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("ccosh (inf - 4.625 i) == -inf + inf i",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, -4.625)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("ccosh (-inf - 4.625 i) == -inf - inf i",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, -4.625)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);

  check_complex ("ccosh (6.75 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (6.75, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (-6.75 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (-6.75, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (6.75 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (6.75, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ccosh (-6.75 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ccosh) (BUILD_COMPLEX (-6.75, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ccosh (0.0 + NaN i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ccosh (-0 + NaN i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ccosh (inf + NaN i) == inf + NaN i",  FUNC(ccosh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("ccosh (-inf + NaN i) == inf + NaN i",  FUNC(ccosh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("ccosh (9.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (9.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccosh (-9.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (-9.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccosh (NaN + 0.0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ccosh (NaN - 0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ccosh (NaN + 10.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, 10.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccosh (NaN - 10.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, -10.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccosh (NaN + inf i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ccosh (NaN - inf i) == NaN + NaN i plus invalid exception allowed",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ccosh (NaN + NaN i) == NaN + NaN i",  FUNC(ccosh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("ccosh (0.7 + 1.2 i) == 0.4548202223691477654 + 0.7070296600921537682 i",  FUNC(ccosh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.4548202223691477654L, 0.7070296600921537682L), MAX_ULP, 0, 0);

  check_complex ("ccosh (-2 - 3 i) == -3.7245455049153225654 + 0.5118225699873846088 i",  FUNC(ccosh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-3.7245455049153225654L, 0.5118225699873846088L), MAX_ULP, 0, 0);
}

static void
cexp_test (void)
{
  errno = 0;
  FUNC(cexp) (BUILD_COMPLEX (0, 0));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("cexp (+0 + +0 i) == 1 + 0.0 i",  FUNC(cexp) (BUILD_COMPLEX (plus_zero, plus_zero)), BUILD_COMPLEX (1, 0.0), MAX_ULP, 0, 0);
  check_complex ("cexp (-0 + +0 i) == 1 + 0.0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_zero, plus_zero)), BUILD_COMPLEX (1, 0.0), MAX_ULP, 0, 0);
  check_complex ("cexp (+0 - 0 i) == 1 - 0 i",  FUNC(cexp) (BUILD_COMPLEX (plus_zero, minus_zero)), BUILD_COMPLEX (1, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cexp (-0 - 0 i) == 1 - 0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (1, minus_zero), MAX_ULP, 0, 0);

  check_complex ("cexp (inf + +0 i) == inf + 0.0 i",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, plus_zero)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("cexp (inf - 0 i) == inf - 0 i",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("cexp (-inf + +0 i) == 0.0 + 0.0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, plus_zero)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("cexp (-inf - 0 i) == 0.0 - 0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);

  check_complex ("cexp (0.0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("cexp (-0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("cexp (0.0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("cexp (-0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("cexp (100.0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (100.0, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("cexp (-100.0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (-100.0, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("cexp (100.0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (100.0, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("cexp (-100.0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(cexp) (BUILD_COMPLEX (-100.0, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("cexp (-inf + 2.0 i) == -0 + 0.0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, 2.0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("cexp (-inf + 4.0 i) == -0 - 0 i",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, 4.0)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cexp (inf + 2.0 i) == -inf + inf i",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, 2.0)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("cexp (inf + 4.0 i) == -inf - inf i",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, 4.0)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);

  check_complex ("cexp (inf + inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("cexp (inf - inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("cexp (-inf + inf i) == 0.0 + 0.0 i plus sign of zero/inf not specified",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (0.0, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("cexp (-inf - inf i) == 0.0 - 0 i plus sign of zero/inf not specified",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cexp (-inf + NaN i) == 0 + 0 i plus sign of zero/inf not specified",  FUNC(cexp) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (0, 0), MAX_ULP, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("cexp (inf + NaN i) == inf + NaN i",  FUNC(cexp) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("cexp (NaN + 0.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cexp) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0.0), MAX_ULP, 0, INVALID_EXCEPTION_OK);
  check_complex ("cexp (NaN + 1.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(cexp) (BUILD_COMPLEX (nan_value, 1.0)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, INVALID_EXCEPTION_OK);

  check_complex ("cexp (NaN + inf i) == NaN + NaN i plus invalid exception allowed",  FUNC(cexp) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, INVALID_EXCEPTION_OK);
  check_complex ("cexp (0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cexp) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, INVALID_EXCEPTION_OK);
  check_complex ("cexp (1 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(cexp) (BUILD_COMPLEX (1, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, INVALID_EXCEPTION_OK);
  check_complex ("cexp (NaN + NaN i) == NaN + NaN i",  FUNC(cexp) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("cexp (0.7 + 1.2 i) == 0.7296989091503238 + 1.8768962328348102 i",  FUNC(cexp) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.7296989091503238, 1.8768962328348102), MAX_ULP, 0, 0);
  check_complex ("cexp (-2.0 - 3.0 i) == -0.13398091492954262 - 0.019098516261135196 i",  FUNC(cexp) (BUILD_COMPLEX (-2.0, -3.0)), BUILD_COMPLEX (-0.13398091492954262, -0.019098516261135196), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, 0);
}

static void
cimag_test (void)
{
  init_max_error ();
  check_float ("cimag (1.0 + 0.0 i) == 0.0",  FUNC(cimag) (BUILD_COMPLEX (1.0, 0.0)), 0.0, MAX_ULP, 0, 0);
  check_float ("cimag (1.0 - 0 i) == -0",  FUNC(cimag) (BUILD_COMPLEX (1.0, minus_zero)), minus_zero, MAX_ULP, 0, 0);
  check_float ("cimag (1.0 + NaN i) == NaN",  FUNC(cimag) (BUILD_COMPLEX (1.0, nan_value)), nan_value, MAX_ULP, 0, 0);
  check_float ("cimag (NaN + NaN i) == NaN",  FUNC(cimag) (BUILD_COMPLEX (nan_value, nan_value)), nan_value, MAX_ULP, 0, 0);
  check_float ("cimag (1.0 + inf i) == inf",  FUNC(cimag) (BUILD_COMPLEX (1.0, plus_infty)), plus_infty, MAX_ULP, 0, 0);
  check_float ("cimag (1.0 - inf i) == -inf",  FUNC(cimag) (BUILD_COMPLEX (1.0, minus_infty)), minus_infty, MAX_ULP, 0, 0);
  check_float ("cimag (2.0 + 3.0 i) == 3.0",  FUNC(cimag) (BUILD_COMPLEX (2.0, 3.0)), 3.0, MAX_ULP, 0, 0);
}

static void
clog_test (void)
{
  errno = 0;
  FUNC(clog) (BUILD_COMPLEX (-2, -3));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("clog (-0 + 0 i) == -inf + pi i plus division by zero exception",  FUNC(clog) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_infty, M_PIl), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_complex ("clog (-0 - 0 i) == -inf - pi i plus division by zero exception",  FUNC(clog) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_infty, -M_PIl), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_complex ("clog (0 + 0 i) == -inf + 0.0 i plus division by zero exception",  FUNC(clog) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (minus_infty, 0.0), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_complex ("clog (0 - 0 i) == -inf - 0 i plus division by zero exception",  FUNC(clog) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (minus_infty, minus_zero), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_complex ("clog (-inf + inf i) == inf + 3/4 pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_34l), MAX_ULP, 0, 0);
  check_complex ("clog (-inf - inf i) == inf - 3/4 pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_34l), MAX_ULP, 0, 0);

  check_complex ("clog (inf + inf i) == inf + pi/4 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_4l), MAX_ULP, 0, 0);
  check_complex ("clog (inf - inf i) == inf - pi/4 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_4l), MAX_ULP, 0, 0);

  check_complex ("clog (0 + inf i) == inf + pi/2 i",  FUNC(clog) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (3 + inf i) == inf + pi/2 i",  FUNC(clog) (BUILD_COMPLEX (3, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (-0 + inf i) == inf + pi/2 i",  FUNC(clog) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (-3 + inf i) == inf + pi/2 i",  FUNC(clog) (BUILD_COMPLEX (-3, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (0 - inf i) == inf - pi/2 i",  FUNC(clog) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (3 - inf i) == inf - pi/2 i",  FUNC(clog) (BUILD_COMPLEX (3, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (-0 - inf i) == inf - pi/2 i",  FUNC(clog) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);
  check_complex ("clog (-3 - inf i) == inf - pi/2 i",  FUNC(clog) (BUILD_COMPLEX (-3, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI_2l), MAX_ULP, 0, 0);

  check_complex ("clog (-inf + 0 i) == inf + pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (plus_infty, M_PIl), MAX_ULP, 0, 0);
  check_complex ("clog (-inf + 1 i) == inf + pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, 1)), BUILD_COMPLEX (plus_infty, M_PIl), MAX_ULP, 0, 0);
  check_complex ("clog (-inf - 0 i) == inf - pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, -M_PIl), MAX_ULP, 0, 0);
  check_complex ("clog (-inf - 1 i) == inf - pi i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, -1)), BUILD_COMPLEX (plus_infty, -M_PIl), MAX_ULP, 0, 0);

  check_complex ("clog (inf + 0 i) == inf + 0.0 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("clog (inf + 1 i) == inf + 0.0 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, 1)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("clog (inf - 0 i) == inf - 0 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("clog (inf - 1 i) == inf - 0 i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, -1)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("clog (inf + NaN i) == inf + NaN i",  FUNC(clog) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("clog (-inf + NaN i) == inf + NaN i",  FUNC(clog) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("clog (NaN + inf i) == inf + NaN i",  FUNC(clog) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("clog (NaN - inf i) == inf + NaN i",  FUNC(clog) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("clog (0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (3 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (3, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (-0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (-3 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (-3, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("clog (NaN + 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (NaN + 5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (nan_value, 5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (NaN - 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog (NaN - 5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog) (BUILD_COMPLEX (nan_value, -5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("clog (NaN + NaN i) == NaN + NaN i",  FUNC(clog) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
  check_complex ("clog (-2 - 3 i) == 1.2824746787307683680267437207826593 - 2.1587989303424641704769327722648368 i",  FUNC(clog) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (1.2824746787307683680267437207826593L, -2.1587989303424641704769327722648368L), MAX_ULP, 0, 0);
}


static void
clog10_test (void)
{
  errno = 0;
  FUNC(clog10) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("clog10 (-0 + 0 i) == -inf + pi i plus division by zero exception",  FUNC(clog10) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_infty, 1.364376), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_complex ("clog10 (-0 - 0 i) == -inf - pi i plus division by zero exception",  FUNC(clog10) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_infty, -M_PIl), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_complex ("clog10 (0 + 0 i) == -inf + 0.0 i plus division by zero exception",  FUNC(clog10) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (minus_infty, 0.0), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_complex ("clog10 (0 - 0 i) == -inf - 0 i plus division by zero exception",  FUNC(clog10) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (minus_infty, minus_zero), 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_complex ("clog10 (-inf + inf i) == inf + 3/4 pi*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI_34_LOG10El), MAX_ULP, 0, 0);

  check_complex ("clog10 (inf + inf i) == inf + pi/4*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI4_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (inf - inf i) == inf - pi/4*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI4_LOG10El), MAX_ULP, 0, 0);

  check_complex ("clog10 (0 + inf i) == inf + pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (3 + inf i) == inf + pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (3, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-0 + inf i) == inf + pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-3 + inf i) == inf + pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (-3, plus_infty)), BUILD_COMPLEX (plus_infty, M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (0 - inf i) == inf - pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (3 - inf i) == inf - pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (3, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-0 - inf i) == inf - pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI2_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-3 - inf i) == inf - pi/2*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (-3, minus_infty)), BUILD_COMPLEX (plus_infty, -M_PI2_LOG10El), MAX_ULP, 0, 0);

  check_complex ("clog10 (-inf + 0 i) == inf + pi*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (plus_infty, M_PI_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-inf + 1 i) == inf + pi*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, 1)), BUILD_COMPLEX (plus_infty, M_PI_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-inf - 0 i) == inf - pi*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, -M_PI_LOG10El), MAX_ULP, 0, 0);
  check_complex ("clog10 (-inf - 1 i) == inf - pi*log10(e) i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, -1)), BUILD_COMPLEX (plus_infty, -M_PI_LOG10El), MAX_ULP, 0, 0);

  check_complex ("clog10 (inf + 0 i) == inf + 0.0 i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("clog10 (inf + 1 i) == inf + 0.0 i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, 1)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("clog10 (inf - 0 i) == inf - 0 i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("clog10 (inf - 1 i) == inf - 0 i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, -1)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("clog10 (inf + NaN i) == inf + NaN i",  FUNC(clog10) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("clog10 (-inf + NaN i) == inf + NaN i",  FUNC(clog10) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("clog10 (NaN + inf i) == inf + NaN i",  FUNC(clog10) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);
  check_complex ("clog10 (NaN - inf i) == inf + NaN i",  FUNC(clog10) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("clog10 (0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (3 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (3, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (-0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (-3 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (-3, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("clog10 (NaN + 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (NaN + 5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (nan_value, 5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (NaN - 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("clog10 (NaN - 5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(clog10) (BUILD_COMPLEX (nan_value, -5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("clog10 (NaN + NaN i) == NaN + NaN i",  FUNC(clog10) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("clog10 (0.7 + 1.2 i) == 0.1427786545038868803 + 0.4528483579352493248 i",  FUNC(clog10) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.1427786545038868803L, 0.4528483579352493248L), MAX_ULP, 0, 0);
  check_complex ("clog10 (-2 - 3 i) == 0.5569716761534183846 - 0.9375544629863747085 i",  FUNC(clog10) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (0.5569716761534183846L, -0.9375544629863747085L), MAX_ULP, 0, 0);
}

static void
conj_test (void)
{
  init_max_error ();
  check_complex ("conj (0.0 + 0.0 i) == 0.0 - 0 i",  FUNC(conj) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("conj (0.0 - 0 i) == 0.0 + 0.0 i",  FUNC(conj) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("conj (NaN + NaN i) == NaN + NaN i",  FUNC(conj) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
  check_complex ("conj (inf - inf i) == inf + inf i",  FUNC(conj) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("conj (inf + inf i) == inf - inf i",  FUNC(conj) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("conj (1.0 + 2.0 i) == 1.0 - 2.0 i",  FUNC(conj) (BUILD_COMPLEX (1.0, 2.0)), BUILD_COMPLEX (1.0, -2.0), MAX_ULP, 0, 0);
  check_complex ("conj (3.0 - 4.0 i) == 3.0 + 4.0 i",  FUNC(conj) (BUILD_COMPLEX (3.0, -4.0)), BUILD_COMPLEX (3.0, 4.0), MAX_ULP, 0, 0);
}

static void
cpow_test (void)
{
  errno = 0;
  FUNC(cpow) (BUILD_COMPLEX (1, 0), BUILD_COMPLEX (0, 0));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("cpow (1 + 0 i, 0 + 0 i) == 1.0 + 0.0 i",  FUNC(cpow) (BUILD_COMPLEX (1, 0), BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("cpow (2 + 0 i, 10 + 0 i) == 1024.0 + 0.0 i",  FUNC(cpow) (BUILD_COMPLEX (2, 0), BUILD_COMPLEX (10, 0)), BUILD_COMPLEX (1024.0, 0.0), MAX_ULP, 0, 0);

  check_complex ("cpow (e + 0 i, 0 + 2 * M_PIl i) == 1.0 + 0.0 i",  FUNC(cpow) (BUILD_COMPLEX (M_El, 0), BUILD_COMPLEX (0, 2 * M_PIl)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("cpow (2 + 3 i, 4 + 0 i) == -119.0 - 120.0 i",  FUNC(cpow) (BUILD_COMPLEX (2, 3), BUILD_COMPLEX (4, 0)), BUILD_COMPLEX (-119.0, -120.0), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, 0);

  check_complex ("cpow (NaN + NaN i, NaN + NaN i) == NaN + NaN i",  FUNC(cpow) (BUILD_COMPLEX (nan_value, nan_value), BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);
}

static void
cproj_test (void)
{
  init_max_error ();
  check_complex ("cproj (0.0 + 0.0 i) == 0.0 + 0.0 i",  FUNC(cproj) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("cproj (-0 - 0 i) == -0 - 0 i",  FUNC(cproj) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cproj (0.0 - 0 i) == 0.0 - 0 i",  FUNC(cproj) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cproj (-0 + 0.0 i) == -0 + 0.0 i",  FUNC(cproj) (BUILD_COMPLEX (minus_zero, 0.0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);

  check_complex ("cproj (NaN + NaN i) == NaN + NaN i",  FUNC(cproj) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("cproj (inf + inf i) == inf + 0.0 i",  FUNC(cproj) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("cproj (inf - inf i) == inf - 0 i",  FUNC(cproj) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("cproj (-inf + inf i) == inf + 0.0 i",  FUNC(cproj) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("cproj (-inf - inf i) == inf - 0 i",  FUNC(cproj) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("cproj (1.0 + 0.0 i) == 1.0 + 0.0 i",  FUNC(cproj) (BUILD_COMPLEX (1.0, 0.0)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("cproj (2.0 + 3.0 i) == 2 + 3 i",  FUNC(cproj) (BUILD_COMPLEX (2.0, 3.0)), BUILD_COMPLEX (2, 3), 0, 0, 0);
}

static void
creal_test (void)
{
  init_max_error ();
  check_float ("creal (0.0 + 1.0 i) == 0.0",  FUNC(creal) (BUILD_COMPLEX (0.0, 1.0)), 0.0, MAX_ULP, 0, 0);
  check_float ("creal (-0 + 1.0 i) == -0",  FUNC(creal) (BUILD_COMPLEX (minus_zero, 1.0)), minus_zero, MAX_ULP, 0, 0);
  check_float ("creal (NaN + 1.0 i) == NaN",  FUNC(creal) (BUILD_COMPLEX (nan_value, 1.0)), nan_value, MAX_ULP, 0, 0);
  check_float ("creal (NaN + NaN i) == NaN",  FUNC(creal) (BUILD_COMPLEX (nan_value, nan_value)), nan_value, MAX_ULP, 0, 0);
  check_float ("creal (inf + 1.0 i) == inf",  FUNC(creal) (BUILD_COMPLEX (plus_infty, 1.0)), plus_infty, MAX_ULP, 0, 0);
  check_float ("creal (-inf + 1.0 i) == -inf",  FUNC(creal) (BUILD_COMPLEX (minus_infty, 1.0)), minus_infty, MAX_ULP, 0, 0);
  check_float ("creal (2.0 + 3.0 i) == 2.0",  FUNC(creal) (BUILD_COMPLEX (2.0, 3.0)), 2.0, MAX_ULP, 0, 0);
}

static void
csin_test (void)
{
  errno = 0;
  FUNC(csin) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("csin (0.0 + 0.0 i) == 0.0 + 0.0 i",  FUNC(csin) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("csin (-0 + 0.0 i) == -0 + 0.0 i",  FUNC(csin) (BUILD_COMPLEX (minus_zero, 0.0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("csin (0.0 - 0 i) == 0 - 0 i",  FUNC(csin) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("csin (-0 - 0 i) == -0 - 0 i",  FUNC(csin) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csin (0.0 + inf i) == 0.0 + inf i",  FUNC(csin) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (-0 + inf i) == -0 + inf i",  FUNC(csin) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (minus_zero, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (0.0 - inf i) == 0.0 - inf i",  FUNC(csin) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (-0 - inf i) == -0 - inf i",  FUNC(csin) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (minus_zero, minus_infty), MAX_ULP, 0, 0);

  check_complex ("csin (inf + 0.0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (plus_infty, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (-inf + 0.0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (minus_infty, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (inf - 0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (-inf - 0 i) == NaN + 0.0 i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("csin (inf + inf i) == NaN + inf i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (-inf + inf i) == NaN + inf i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (inf - inf i) == NaN + inf i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (-inf - inf i) == NaN + inf i plus invalid exception and sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("csin (inf + 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(csin) (BUILD_COMPLEX (plus_infty, 6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csin (inf - 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(csin) (BUILD_COMPLEX (plus_infty, -6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csin (-inf + 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(csin) (BUILD_COMPLEX (minus_infty, 6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csin (-inf - 6.75 i) == NaN + NaN i plus invalid exception",  FUNC(csin) (BUILD_COMPLEX (minus_infty, -6.75)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("csin (4.625 + inf i) == -inf - inf i",  FUNC(csin) (BUILD_COMPLEX (4.625, plus_infty)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (4.625 - inf i) == -inf + inf i",  FUNC(csin) (BUILD_COMPLEX (4.625, minus_infty)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (-4.625 + inf i) == inf - inf i",  FUNC(csin) (BUILD_COMPLEX (-4.625, plus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csin (-4.625 - inf i) == inf + inf i",  FUNC(csin) (BUILD_COMPLEX (-4.625, minus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);

  check_complex ("csin (NaN + 0.0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (NaN - 0 i) == NaN + 0.0 i plus sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("csin (NaN + inf i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("csin (NaN - inf i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(csin) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("csin (NaN + 9.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (nan_value, 9.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csin (NaN - 9.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (nan_value, -9.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csin (0.0 + NaN i) == 0.0 + NaN i",  FUNC(csin) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (0.0, nan_value), MAX_ULP, 0, 0);
  check_complex ("csin (-0 + NaN i) == -0 + NaN i",  FUNC(csin) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (minus_zero, nan_value), MAX_ULP, 0, 0);

  check_complex ("csin (10.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (10.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csin (NaN - 10.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (nan_value, -10.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csin (inf + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csin (-inf + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csin) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csin (NaN + NaN i) == NaN + NaN i",  FUNC(csin) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("csin (0.7 + 1.2 i) == 1.1664563419657581376 + 1.1544997246948547371 i",  FUNC(csin) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.1664563419657581376L, 1.1544997246948547371L), MAX_ULP, 0, 0);

  check_complex ("csin (-2 - 3 i) == -9.1544991469114295734 + 4.1689069599665643507 i",  FUNC(csin) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-9.1544991469114295734L, 4.1689069599665643507L), MAX_ULP, 0, 0);
}


static void
csinh_test (void)
{
  errno = 0;
  FUNC(csinh) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("csinh (0.0 + 0.0 i) == 0.0 + 0.0 i",  FUNC(csinh) (BUILD_COMPLEX (0.0, 0.0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("csinh (-0 + 0.0 i) == -0 + 0.0 i",  FUNC(csinh) (BUILD_COMPLEX (minus_zero, 0.0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("csinh (0.0 - 0 i) == 0.0 - 0 i",  FUNC(csinh) (BUILD_COMPLEX (0.0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("csinh (-0 - 0 i) == -0 - 0 i",  FUNC(csinh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csinh (0.0 + inf i) == 0.0 + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (0.0, plus_infty)), BUILD_COMPLEX (0.0, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-0 + inf i) == 0.0 + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (0.0, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (0.0 - inf i) == 0.0 + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (0.0, minus_infty)), BUILD_COMPLEX (0.0, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-0 - inf i) == 0.0 + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (0.0, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("csinh (inf + 0.0 i) == inf + 0.0 i",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, 0.0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("csinh (-inf + 0.0 i) == -inf + 0.0 i",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, 0.0)), BUILD_COMPLEX (minus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("csinh (inf - 0 i) == inf - 0 i",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("csinh (-inf - 0 i) == -inf - 0 i",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (minus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csinh (inf + inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-inf + inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (inf - inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-inf - inf i) == inf + NaN i plus invalid exception and sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, INVALID_EXCEPTION|IGNORE_ZERO_INF_SIGN);

  check_complex ("csinh (inf + 4.625 i) == -inf - inf i",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, 4.625)), BUILD_COMPLEX (minus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csinh (-inf + 4.625 i) == inf - inf i",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, 4.625)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csinh (inf - 4.625 i) == -inf + inf i",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, -4.625)), BUILD_COMPLEX (minus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csinh (-inf - 4.625 i) == inf + inf i",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, -4.625)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);

  check_complex ("csinh (6.75 + inf i) == NaN + NaN i plus invalid exception",  FUNC(csinh) (BUILD_COMPLEX (6.75, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csinh (-6.75 + inf i) == NaN + NaN i plus invalid exception",  FUNC(csinh) (BUILD_COMPLEX (-6.75, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csinh (6.75 - inf i) == NaN + NaN i plus invalid exception",  FUNC(csinh) (BUILD_COMPLEX (6.75, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("csinh (-6.75 - inf i) == NaN + NaN i plus invalid exception",  FUNC(csinh) (BUILD_COMPLEX (-6.75, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("csinh (0.0 + NaN i) == 0.0 + NaN i plus sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (0.0, nan_value)), BUILD_COMPLEX (0.0, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-0 + NaN i) == 0.0 + NaN i plus sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (0.0, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("csinh (inf + NaN i) == inf + NaN i plus sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("csinh (-inf + NaN i) == inf + NaN i plus sign of zero/inf not specified",  FUNC(csinh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("csinh (9.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (9.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csinh (-9.0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (-9.0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csinh (NaN + 0.0 i) == NaN + 0.0 i",  FUNC(csinh) (BUILD_COMPLEX (nan_value, 0.0)), BUILD_COMPLEX (nan_value, 0.0), MAX_ULP, 0, 0);
  check_complex ("csinh (NaN - 0 i) == NaN - 0 i",  FUNC(csinh) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csinh (NaN + 10.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (nan_value, 10.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csinh (NaN - 10.0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (nan_value, -10.0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csinh (NaN + inf i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csinh (NaN - inf i) == NaN + NaN i plus invalid exception allowed",  FUNC(csinh) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csinh (NaN + NaN i) == NaN + NaN i",  FUNC(csinh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("csinh (0.7 + 1.2 i) == 0.27487868678117583582 + 1.1698665727426565139 i",  FUNC(csinh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.27487868678117583582L, 1.1698665727426565139L), MAX_ULP, 0, 0);
  check_complex ("csinh (-2 - 3 i) == 3.5905645899857799520 - 0.5309210862485198052 i",  FUNC(csinh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (3.5905645899857799520L, -0.5309210862485198052L), MAX_ULP, 0, 0);
}

static void
csqrt_test (void)
{
  errno = 0;
  FUNC(csqrt) (BUILD_COMPLEX (-1, 0));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("csqrt (0 + 0 i) == 0.0 + 0.0 i",  FUNC(csqrt) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (0 - 0 i) == 0 - 0 i",  FUNC(csqrt) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("csqrt (-0 + 0 i) == 0.0 + 0.0 i",  FUNC(csqrt) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (-0 - 0 i) == 0.0 - 0 i",  FUNC(csqrt) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csqrt (-inf + 0 i) == 0.0 + inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-inf + 6 i) == 0.0 + inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, 6)), BUILD_COMPLEX (0.0, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-inf - 0 i) == 0.0 - inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-inf - 6 i) == 0.0 - inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, -6)), BUILD_COMPLEX (0.0, minus_infty), MAX_ULP, 0, 0);

  check_complex ("csqrt (inf + 0 i) == inf + 0.0 i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (inf + 6 i) == inf + 0.0 i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, 6)), BUILD_COMPLEX (plus_infty, 0.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (inf - 0 i) == inf - 0 i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);
  check_complex ("csqrt (inf - 6 i) == inf - 0 i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, -6)), BUILD_COMPLEX (plus_infty, minus_zero), MAX_ULP, 0, 0);

  check_complex ("csqrt (0 + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (4 + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (4, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (inf + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-0 + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-4 + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (-4, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-inf + inf i) == inf + inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, plus_infty)), BUILD_COMPLEX (plus_infty, plus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (0 - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (4 - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (4, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (inf - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-0 - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-4 - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (-4, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);
  check_complex ("csqrt (-inf - inf i) == inf - inf i",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, minus_infty)), BUILD_COMPLEX (plus_infty, minus_infty), MAX_ULP, 0, 0);

  check_complex ("csqrt (-inf + NaN i) == NaN + inf i plus sign of zero/inf not specified",  FUNC(csqrt) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (nan_value, plus_infty), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("csqrt (inf + NaN i) == inf + NaN i",  FUNC(csqrt) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (plus_infty, nan_value), MAX_ULP, 0, 0);

  check_complex ("csqrt (0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (1 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (1, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (-0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (-1 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (-1, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csqrt (NaN + 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (NaN + 8 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (nan_value, 8)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (NaN - 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("csqrt (NaN - 8 i) == NaN + NaN i plus invalid exception allowed",  FUNC(csqrt) (BUILD_COMPLEX (nan_value, -8)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("csqrt (NaN + NaN i) == NaN + NaN i",  FUNC(csqrt) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("csqrt (16.0 - 30.0 i) == 5.0 - 3.0 i",  FUNC(csqrt) (BUILD_COMPLEX (16.0, -30.0)), BUILD_COMPLEX (5.0, -3.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (-1 + 0 i) == 0.0 + 1.0 i",  FUNC(csqrt) (BUILD_COMPLEX (-1, 0)), BUILD_COMPLEX (0.0, 1.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (0 + 2 i) == 1.0 + 1.0 i",  FUNC(csqrt) (BUILD_COMPLEX (0, 2)), BUILD_COMPLEX (1.0, 1.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (119 + 120 i) == 12.0 + 5.0 i",  FUNC(csqrt) (BUILD_COMPLEX (119, 120)), BUILD_COMPLEX (12.0, 5.0), MAX_ULP, 0, 0);
  check_complex ("csqrt (0.7 + 1.2 i) == 1.0220676100300263 + 0.5870453129635652 i",  FUNC(csqrt) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.0220676100300263, 0.5870453129635652), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, 0);
  check_complex ("csqrt (-2 - 3 i) == 0.8959774761298382 - 1.6741492280355400404480393008490519 i",  FUNC(csqrt) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (0.8959774761298382, -1.67414922803554), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, 0);
  check_complex ("csqrt (-2 + 3 i) == 0.8959774761298382 + 1.6741492280355400404480393008490519 i",  FUNC(csqrt) (BUILD_COMPLEX (-2, 3)), BUILD_COMPLEX (0.8959774761298382, 1.67414922803554), BUILD_COMPLEX(MAX_ULP, MAX_ULP), 0, 0);
}

static void
ctan_test (void)
{
  errno = 0;
  FUNC(ctan) (BUILD_COMPLEX (0.7L, 1.2L));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("ctan (0 + 0 i) == 0.0 + 0.0 i",  FUNC(ctan) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctan (0 - 0 i) == 0.0 - 0 i",  FUNC(ctan) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ctan (-0 + 0 i) == -0 + 0.0 i",  FUNC(ctan) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctan (-0 - 0 i) == -0 - 0 i",  FUNC(ctan) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ctan (0 + inf i) == 0.0 + 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (0.0, 1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (1 + inf i) == 0.0 + 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (1, plus_infty)), BUILD_COMPLEX (0.0, 1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (-0 + inf i) == -0 + 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (minus_zero, 1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (-1 + inf i) == -0 + 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (-1, plus_infty)), BUILD_COMPLEX (minus_zero, 1.0), MAX_ULP, 0, 0);

  check_complex ("ctan (0 - inf i) == 0.0 - 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (0.0, -1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (1 - inf i) == 0.0 - 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (1, minus_infty)), BUILD_COMPLEX (0.0, -1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (-0 - inf i) == -0 - 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (minus_zero, -1.0), MAX_ULP, 0, 0);
  check_complex ("ctan (-1 - inf i) == -0 - 1.0 i",  FUNC(ctan) (BUILD_COMPLEX (-1, minus_infty)), BUILD_COMPLEX (minus_zero, -1.0), MAX_ULP, 0, 0);

  check_complex ("ctan (inf + 0 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (inf + 2 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (plus_infty, 2)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (-inf + 0 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (-inf + 2 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (minus_infty, 2)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (inf - 0 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (inf - 2 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (plus_infty, -2)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (-inf - 0 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctan (-inf - 2 i) == NaN + NaN i plus invalid exception",  FUNC(ctan) (BUILD_COMPLEX (minus_infty, -2)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ctan (NaN + inf i) == 0.0 + 1.0 i plus sign of zero/inf not specified",  FUNC(ctan) (BUILD_COMPLEX (nan_value, plus_infty)), BUILD_COMPLEX (0.0, 1.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ctan (NaN - inf i) == 0.0 - 1.0 i plus sign of zero/inf not specified",  FUNC(ctan) (BUILD_COMPLEX (nan_value, minus_infty)), BUILD_COMPLEX (0.0, -1.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ctan (0 + NaN i) == 0.0 + NaN i",  FUNC(ctan) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (0.0, nan_value), MAX_ULP, 0, 0);
  check_complex ("ctan (-0 + NaN i) == -0 + NaN i",  FUNC(ctan) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (minus_zero, nan_value), MAX_ULP, 0, 0);

  check_complex ("ctan (0.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (0.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctan (-4.5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (-4.5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ctan (NaN + 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctan (NaN + 5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (nan_value, 5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctan (NaN - 0 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctan (NaN - 0.25 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctan) (BUILD_COMPLEX (nan_value, -0.25)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ctan (NaN + NaN i) == NaN + NaN i",  FUNC(ctan) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("ctan (0.7 + 1.2 i) == 0.17207341 + 0.95448065 i",  FUNC(ctan) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (0.17207341, 0.95448065), MAX_ULP, 0, 0);
  check_complex ("ctan (-2 - 3 i) == 0.0037640256415042482 - 1.0032386273536098014 i",  FUNC(ctan) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (0.0037640256415042482L, -1.0032386273536098014L), MAX_ULP, 0, 0);
}


static void
ctanh_test (void)
{
  errno = 0;
  FUNC(ctanh) (BUILD_COMPLEX (0, 0));
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_complex ("ctanh (0 + 0 i) == 0.0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (0, 0)), BUILD_COMPLEX (0.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (0 - 0 i) == 0.0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (0, minus_zero)), BUILD_COMPLEX (0.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ctanh (-0 + 0 i) == -0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_zero, 0)), BUILD_COMPLEX (minus_zero, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (-0 - 0 i) == -0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_zero, minus_zero)), BUILD_COMPLEX (minus_zero, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ctanh (inf + 0 i) == 1.0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (plus_infty, 0)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (inf + 1 i) == 1.0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (plus_infty, 1)), BUILD_COMPLEX (1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (inf - 0 i) == 1.0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (plus_infty, minus_zero)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ctanh (inf - 1 i) == 1.0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (plus_infty, -1)), BUILD_COMPLEX (1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ctanh (-inf + 0 i) == -1.0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_infty, 0)), BUILD_COMPLEX (-1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (-inf + 1 i) == -1.0 + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_infty, 1)), BUILD_COMPLEX (-1.0, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (-inf - 0 i) == -1.0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_infty, minus_zero)), BUILD_COMPLEX (-1.0, minus_zero), MAX_ULP, 0, 0);
  check_complex ("ctanh (-inf - 1 i) == -1.0 - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (minus_infty, -1)), BUILD_COMPLEX (-1.0, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ctanh (0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (0, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (2 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (2, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (0, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (2 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (2, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (-0 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (minus_zero, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (-2 + inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (-2, plus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (-0 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (minus_zero, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);
  check_complex ("ctanh (-2 - inf i) == NaN + NaN i plus invalid exception",  FUNC(ctanh) (BUILD_COMPLEX (-2, minus_infty)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION);

  check_complex ("ctanh (inf + NaN i) == 1.0 + 0.0 i plus sign of zero/inf not specified",  FUNC(ctanh) (BUILD_COMPLEX (plus_infty, nan_value)), BUILD_COMPLEX (1.0, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);
  check_complex ("ctanh (-inf + NaN i) == -1.0 + 0.0 i plus sign of zero/inf not specified",  FUNC(ctanh) (BUILD_COMPLEX (minus_infty, nan_value)), BUILD_COMPLEX (-1.0, 0.0), 0, 0, IGNORE_ZERO_INF_SIGN);

  check_complex ("ctanh (NaN + 0 i) == NaN + 0.0 i",  FUNC(ctanh) (BUILD_COMPLEX (nan_value, 0)), BUILD_COMPLEX (nan_value, 0.0), MAX_ULP, 0, 0);
  check_complex ("ctanh (NaN - 0 i) == NaN - 0 i",  FUNC(ctanh) (BUILD_COMPLEX (nan_value, minus_zero)), BUILD_COMPLEX (nan_value, minus_zero), MAX_ULP, 0, 0);

  check_complex ("ctanh (NaN + 0.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (nan_value, 0.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctanh (NaN - 4.5 i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (nan_value, -4.5)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ctanh (0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (0, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctanh (5 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (5, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctanh (-0 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (minus_zero, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);
  check_complex ("ctanh (-0.25 + NaN i) == NaN + NaN i plus invalid exception allowed",  FUNC(ctanh) (BUILD_COMPLEX (-0.25, nan_value)), BUILD_COMPLEX (nan_value, nan_value), 0, 0, INVALID_EXCEPTION_OK);

  check_complex ("ctanh (NaN + NaN i) == NaN + NaN i",  FUNC(ctanh) (BUILD_COMPLEX (nan_value, nan_value)), BUILD_COMPLEX (nan_value, nan_value), MAX_ULP, 0, 0);

  check_complex ("ctanh (0 + pi/4 i) == 0.0 + 1.0 i",  FUNC(ctanh) (BUILD_COMPLEX (0, M_PI_4l)), BUILD_COMPLEX (0.0, 1.0), MAX_ULP, 0, 0);

  check_complex ("ctanh (0.7 + 1.2 i) == 1.3472197399061191630 + 0.4778641038326365540 i",  FUNC(ctanh) (BUILD_COMPLEX (0.7L, 1.2L)), BUILD_COMPLEX (1.3472197399061191630L, 0.4778641038326365540L), MAX_ULP, 0, 0);
  check_complex ("ctanh (-2 - 3 i) == -0.965386 + 0.009884375 i",  FUNC(ctanh) (BUILD_COMPLEX (-2, -3)), BUILD_COMPLEX (-0.965386, 0.009884375), MAX_ULP, 0, 0);
}

int
main() {
  initialize();
  /* Keep the tests a wee bit ordered (according to ISO C99).  */
  /* Classification macros:  */
  fpclassify_test();
  isfinite_test();
  isnormal_test();
  signbit_test();

  /* Complex functions:  */
  cabs_test();
  cacos_test();
  cacosh_test();
  carg_test();
  casin_test();
  casinh_test();
  catan_test();
  catanh_test();
  ccos_test();
  ccosh_test();
  cexp_test();
  cimag_test();
  clog10_test();
  clog_test();
  conj_test();
  cpow_test();
  cproj_test();
  creal_test();
  csin_test();
  csinh_test();
  csqrt_test();
  ctan_test();
  ctanh_test();

  printf("\nTest suite completed:\n");
  printf("  %d test cases plus %d tests for exception flags executed.\n",
    noTests, noExcTests);
  if (noXFails)
    printf("  %d expected failures occurred.\n", noXFails);
  if (noXPasses)
    printf("  %d unexpected passes occurred.\n", noXPasses);
  if (noErrors) {
    printf("  %d errors occurred.\n", noErrors);
  } 
  else {
    printf("  All tests passed successfully.\n");
  }
  return 0;
}