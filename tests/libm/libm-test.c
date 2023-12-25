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

// Helps debugging
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
  int fe_flag,
  const char *flag_name) {
  int ok = 1;
  if (exception & exc_flag) {
    if (fetestexcept(fe_flag)) {
      if (LOG)
        printf ("Pass Exception \"%s\" set\n", flag_name);
    } else {
      ok = 0;
      if (LOG)
        printf ("Failure Exception \"%s\" set\n", flag_name);
    }
  } else {
    if (fetestexcept(fe_flag)) {
      ok = 0;
      if (LOG)
        printf ("Failure Exception \"%s\" set\n", flag_name);
    } else {
      if (LOG)
        printf ("Exception \"%s\" not set, ok = %d\n", flag_name, ok);
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
  if (LOG)
    printf("%s\n", test_name); 
  #ifdef FE_DIVBYZERO
  if ((exception & DIVIDE_BY_ZERO_EXCEPTION_OK) == 0)
    test_single_exception(exception, DIVIDE_BY_ZERO_EXCEPTION, FE_DIVBYZERO, "Divide by zero");
  #endif
  #ifdef FE_INVALID
  if ((exception & INVALID_EXCEPTION_OK) == 0)
    test_single_exception(exception, INVALID_EXCEPTION, FE_INVALID, "Invalid operation");
  #endif
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
static void
check_long(const char * test_name, long int computed, long int expected,
  long int max_ulp, int xfail, int exceptions) {
  long int diff = computed - expected;
  int ok = 0;

  test_exceptions(test_name, exceptions);
  noTests++;
  if (labs(diff) <= max_ulp)
    ok = 1;
  update_stats(ok, xfail);
}

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
acos_test (void)
{
  errno = 0;
  FUNC(acos) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("acos (inf) == NaN plus invalid exception",  FUNC(acos) (plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("acos (-inf) == NaN plus invalid exception",  FUNC(acos) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("acos (NaN) == NaN",  FUNC(acos) (nan_value), nan_value, 0, 0, 0);

  /* |x| > 1: */
  check_float ("acos (1.1) == NaN plus invalid exception",  FUNC(acos) (1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("acos (-1.1) == NaN plus invalid exception",  FUNC(acos) (-1.1L), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("acos (0) == pi/2",  FUNC(acos) (0), M_PI_2l, MAX_ULP, 0, 0);
  check_float ("acos (-0) == pi/2",  FUNC(acos) (minus_zero), M_PI_2l, MAX_ULP, 0, 0);
  check_float ("acos (1) == 0",  FUNC(acos) (1), 0, 0, 0, 0);
  check_float ("acos (-1) == pi",  FUNC(acos) (-1), M_PIl, MAX_ULP, 0, 0);
  check_float ("acos (0.5) == M_PI_6l*2.0",  FUNC(acos) (0.5), M_PI_6l*2.0, MAX_ULP, 0, 0);
  check_float ("acos (-0.5) == M_PI_6l*4.0",  FUNC(acos) (-0.5), M_PI_6l*4.0, MAX_ULP, 0, 0);
  check_float ("acos (0.7) == 0.7953988301841436",  FUNC(acos) (0.7L), 0.7953988301841436, MAX_ULP, 0, 0);
}

static void
asin_test (void)
{
  errno = 0;
  FUNC(asin) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("asin (inf) == NaN plus invalid exception",  FUNC(asin) (plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("asin (-inf) == NaN plus invalid exception",  FUNC(asin) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("asin (NaN) == NaN",  FUNC(asin) (nan_value), nan_value, 0, 0, 0);

  /* asin x == NaN plus invalid exception for |x| > 1.  */
  check_float ("asin (1.1) == NaN plus invalid exception",  FUNC(asin) (1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("asin (-1.1) == NaN plus invalid exception",  FUNC(asin) (-1.1L), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("asin (0) == 0",  FUNC(asin) (0), 0, 0, 0, 0);
  check_float ("asin (-0) == -0",  FUNC(asin) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("asin (0.5) == pi/6",  FUNC(asin) (0.5), M_PI_6l, MAX_ULP, 0, 0);
  check_float ("asin (-0.5) == -pi/6",  FUNC(asin) (-0.5), -M_PI_6l, MAX_ULP, 0, 0);
  check_float ("asin (1.0) == pi/2",  FUNC(asin) (1.0), M_PI_2l, MAX_ULP, 0, 0);
  check_float ("asin (-1.0) == -pi/2",  FUNC(asin) (-1.0), -M_PI_2l, MAX_ULP, 0, 0);
  check_float ("asin (0.7) == 0.77539749661075306374035335271498708",  FUNC(asin) (0.7L), 0.77539749661075306374035335271498708L, MAX_ULP, 0, 0);
}

static void
atan_test (void)
{
  errno = 0;
  FUNC(atan) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("atan (0) == 0",  FUNC(atan) (0), 0, 0, 0, 0);
  check_float ("atan (-0) == -0",  FUNC(atan) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("atan (inf) == pi/2",  FUNC(atan) (plus_infty), M_PI_2l, MAX_ULP, 0, 0);
  check_float ("atan (-inf) == -pi/2",  FUNC(atan) (minus_infty), -M_PI_2l, MAX_ULP, 0, 0);
  check_float ("atan (NaN) == NaN",  FUNC(atan) (nan_value), nan_value, 0, 0, 0);

  check_float ("atan (1) == pi/4",  FUNC(atan) (1), M_PI_4l, 0, 0, 0);
  check_float ("atan (-1) == -pi/4",  FUNC(atan) (-1), -M_PI_4l, 0, 0, 0);

  check_float ("atan (0.7) == 0.61072596438920861654375887649023613",  FUNC(atan) (0.7L), 0.61072596438920861654375887649023613L, MAX_ULP, 0, 0);
}

static void
atan2_test (void)
{
  errno = 0;
  FUNC(atan2) (-0, 1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  /* atan2 (0,x) == 0 for x > 0.  */
  check_float ("atan2 (0, 1) == 0",  FUNC(atan2) (0, 1), 0, 0, 0, 0);

  /* atan2 (-0,x) == -0 for x > 0.  */
  check_float ("atan2 (-0, 1) == -0",  FUNC(atan2) (minus_zero, 1), minus_zero, 0, 0, 0);

  check_float ("atan2 (0, 0) == 0",  FUNC(atan2) (0, 0), 0, 0, 0, 0);
  check_float ("atan2 (-0, 0) == -0",  FUNC(atan2) (minus_zero, 0), minus_zero, 0, 0, 0);

  /* atan2 (+0,x) == +pi for x < 0.  */
  check_float ("atan2 (0, -1) == pi",  FUNC(atan2) (0, -1), M_PIl, 0, 0, 0);

  /* atan2 (-0,x) == -pi for x < 0.  */
  check_float ("atan2 (-0, -1) == -pi",  FUNC(atan2) (minus_zero, -1), -M_PIl, 0, 0, 0);

  check_float ("atan2 (0, -0) == pi",  FUNC(atan2) (0, minus_zero), M_PIl, 0, 0, 0);
  check_float ("atan2 (-0, -0) == -pi",  FUNC(atan2) (minus_zero, minus_zero), -M_PIl, 0, 0, 0);

  /* atan2 (y,+0) == pi/2 for y > 0.  */
  check_float ("atan2 (1, 0) == pi/2",  FUNC(atan2) (1, 0), M_PI_2l, 0, 0, 0);

  /* atan2 (y,-0) == pi/2 for y > 0.  */
  check_float ("atan2 (1, -0) == pi/2",  FUNC(atan2) (1, minus_zero), M_PI_2l, 0, 0, 0);

  /* atan2 (y,+0) == -pi/2 for y < 0.  */
  check_float ("atan2 (-1, 0) == -pi/2",  FUNC(atan2) (-1, 0), -M_PI_2l, 0, 0, 0);

  /* atan2 (y,-0) == -pi/2 for y < 0.  */
  check_float ("atan2 (-1, -0) == -pi/2",  FUNC(atan2) (-1, minus_zero), -M_PI_2l, 0, 0, 0);

  /* atan2 (y,inf) == +0 for finite y > 0.  */
  check_float ("atan2 (1, inf) == 0",  FUNC(atan2) (1, plus_infty), 0, 0, 0, 0);

  /* atan2 (y,inf) == -0 for finite y < 0.  */
  check_float ("atan2 (-1, inf) == -0",  FUNC(atan2) (-1, plus_infty), minus_zero, 0, 0, 0);

  /* atan2(+inf, x) == pi/2 for finite x.  */
  check_float ("atan2 (inf, -1) == pi/2",  FUNC(atan2) (plus_infty, -1), M_PI_2l, 0, 0, 0);

  /* atan2(-inf, x) == -pi/2 for finite x.  */
  check_float ("atan2 (-inf, 1) == -pi/2",  FUNC(atan2) (minus_infty, 1), -M_PI_2l, MAX_ULP, 0, 0);

  /* atan2 (y,-inf) == +pi for finite y > 0.  */
  check_float ("atan2 (1, -inf) == pi",  FUNC(atan2) (1, minus_infty), M_PIl, 0, 0, 0);

  /* atan2 (y,-inf) == -pi for finite y < 0.  */
  check_float ("atan2 (-1, -inf) == -pi",  FUNC(atan2) (-1, minus_infty), -M_PIl, 0, 0, 0);

  check_float ("atan2 (inf, inf) == pi/4",  FUNC(atan2) (plus_infty, plus_infty), M_PI_4l, 0, 0, 0);
  check_float ("atan2 (-inf, inf) == -pi/4",  FUNC(atan2) (minus_infty, plus_infty), -M_PI_4l, 0, 0, 0);
  check_float ("atan2 (inf, -inf) == 3/4 pi",  FUNC(atan2) (plus_infty, minus_infty), M_PI_34l, 0, 0, 0);
  check_float ("atan2 (-inf, -inf) == -3/4 pi",  FUNC(atan2) (minus_infty, minus_infty), -M_PI_34l, 0, 0, 0);
  check_float ("atan2 (NaN, NaN) == NaN",  FUNC(atan2) (nan_value, nan_value), nan_value, 0, 0, 0);

  check_float ("atan2 (0.7, 1) == 0.61072596438920861654375887649023613",  FUNC(atan2) (0.7L, 1), 0.61072596438920861654375887649023613L, MAX_ULP, 0, 0);
  check_float ("atan2 (-0.7, 1.0) == -0.61072596438920861654375887649023613",  FUNC(atan2) (-0.7L, 1.0L), -0.61072596438920861654375887649023613L, 0, 0, 0);
  check_float ("atan2 (0.7, -1.0) == 2.530866689200584621918884506789267",  FUNC(atan2) (0.7L, -1.0L), 2.530866689200584621918884506789267L, 0, 0, 0);
  check_float ("atan2 (-0.7, -1.0) == -2.530866689200584621918884506789267",  FUNC(atan2) (-0.7L, -1.0L), -2.530866689200584621918884506789267L, 0, 0, 0);
  check_float ("atan2 (0.4, 0.0003) == 1.5700463269355215717704032607580829",  FUNC(atan2) (0.4L, 0.0003L), 1.5700463269355215717704032607580829L, MAX_ULP, 0, 0);
  check_float ("atan2 (1.4, -0.93) == 2.1571487668237843754887415992772736",  FUNC(atan2) (1.4L, -0.93L), 2.1571487668237843754887415992772736L, 0, 0, 0);
}

static void
cos_test (void)
{
  errno = 0;
  FUNC(cos) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("cos (0) == 1",  FUNC(cos) (0), 1, 0, 0, 0);
  check_float ("cos (-0) == 1",  FUNC(cos) (minus_zero), 1, 0, 0, 0);
  check_float ("cos (inf) == NaN plus invalid exception",  FUNC(cos) (plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("cos (-inf) == NaN plus invalid exception",  FUNC(cos) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("cos (NaN) == NaN",  FUNC(cos) (nan_value), nan_value, 0, 0, 0);

  check_float ("cos (M_PI_6l * 2.0) == 0.5",  FUNC(cos) (M_PI_6l * 2.0), 0.5, MAX_ULP, 0, 0);
  check_float ("cos (M_PI_6l * 4.0) == -0.5",  FUNC(cos) (M_PI_6l * 4.0), -0.5, MAX_ULP, 0, 0);
  check_float ("cos (pi/2) == 0",  FUNC(cos) (M_PI_2l), 0, MAX_ULP, 0, 0);

  check_float ("cos (0.7) == 0.76484218728448842625585999019186495",  FUNC(cos) (0.7L), 0.76484218728448842625585999019186495L, MAX_ULP, 0, 0);
}

static void
sin_test (void)
{
  errno = 0;
  FUNC(sin) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("sin (0) == 0",  FUNC(sin) (0), 0, 0, 0, 0);
  check_float ("sin (-0) == -0",  FUNC(sin) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("sin (inf) == NaN plus invalid exception",  FUNC(sin) (plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sin (-inf) == NaN plus invalid exception",  FUNC(sin) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sin (NaN) == NaN",  FUNC(sin) (nan_value), nan_value, 0, 0, 0);

  check_float ("sin (pi/6) == 0.5",  FUNC(sin) (M_PI_6l), 0.5, 0, 0, 0);
  check_float ("sin (-pi/6) == -0.5",  FUNC(sin) (-M_PI_6l), -0.5, 0, 0, 0);
  check_float ("sin (pi/2) == 1",  FUNC(sin) (M_PI_2l), 1, 0, 0, 0);
  check_float ("sin (-pi/2) == -1",  FUNC(sin) (-M_PI_2l), -1, 0, 0, 0);
  check_float ("sin (0.7) == 0.64421768723769105367261435139872014",  FUNC(sin) (0.7L), 0.64421768723769105367261435139872014L, MAX_ULP, 0, 0);
}

static void
sincos_test (void)
{
  FLOAT sin_res, cos_res;

  errno = 0;
  FUNC(sincos) (0, &sin_res, &cos_res);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  /* sincos is treated differently because it returns void.  */
  FUNC (sincos) (0, &sin_res, &cos_res);
  check_float ("sincos (0, &sin_res, &cos_res) puts 0 in sin_res", sin_res, 0, 0, 0, 0);
  check_float ("sincos (0, &sin_res, &cos_res) puts 1 in cos_res", cos_res, 1, 0, 0, 0);

  FUNC (sincos) (minus_zero, &sin_res, &cos_res);
  check_float ("sincos (-0, &sin_res, &cos_res) puts -0 in sin_res", sin_res, minus_zero, 0, 0, 0);
  check_float ("sincos (-0, &sin_res, &cos_res) puts 1 in cos_res", cos_res, 1, 0, 0, 0);
  FUNC (sincos) (plus_infty, &sin_res, &cos_res);
  check_float ("sincos (inf, &sin_res, &cos_res) puts NaN in sin_res plus invalid exception", sin_res, nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sincos (inf, &sin_res, &cos_res) puts NaN in cos_res", cos_res, nan_value, 0, 0, 0);
  FUNC (sincos) (minus_infty, &sin_res, &cos_res);
  check_float ("sincos (-inf, &sin_res, &cos_res) puts NaN in sin_res plus invalid exception", sin_res, nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sincos (-inf, &sin_res, &cos_res) puts NaN in cos_res", cos_res, nan_value, 0, 0, 0);
  FUNC (sincos) (nan_value, &sin_res, &cos_res);
  check_float ("sincos (NaN, &sin_res, &cos_res) puts NaN in sin_res", sin_res, nan_value, 0, 0, 0);
  check_float ("sincos (NaN, &sin_res, &cos_res) puts NaN in cos_res", cos_res, nan_value, 0, 0, 0);

  FUNC (sincos) (M_PI_2l, &sin_res, &cos_res);
  check_float ("sincos (pi/2, &sin_res, &cos_res) puts 1 in sin_res", sin_res, 1, 0, 0, 0);
  check_float ("sincos (pi/2, &sin_res, &cos_res) puts 0 in cos_res", cos_res, 0, MAX_ULP, 0, 0);
  FUNC (sincos) (M_PI_6l, &sin_res, &cos_res);
  check_float ("sincos (pi/6, &sin_res, &cos_res) puts 0.5 in sin_res", sin_res, 0.5, 0, 0, 0);
  check_float ("sincos (pi/6, &sin_res, &cos_res) puts 0.86602540378443864676372317075293616 in cos_res", cos_res, 0.86602540378443864676372317075293616L, 0, 0, 0);
  FUNC (sincos) (M_PI_6l*2.0, &sin_res, &cos_res);
  check_float ("sincos (M_PI_6l*2.0, &sin_res, &cos_res) puts 0.86602540378443864676372317075293616 in sin_res", sin_res, 0.86602540378443864676372317075293616L, MAX_ULP, 0, 0);
  check_float ("sincos (M_PI_6l*2.0, &sin_res, &cos_res) puts 0.5 in cos_res", cos_res, 0.5, MAX_ULP, 0, 0);
  FUNC (sincos) (0.7L, &sin_res, &cos_res);
  check_float ("sincos (0.7, &sin_res, &cos_res) puts 0.64421768723769105367261435139872014 in sin_res", sin_res, 0.64421768723769105367261435139872014L, MAX_ULP, 0, 0);
  check_float ("sincos (0.7, &sin_res, &cos_res) puts 0.76484218728448842625585999019186495 in cos_res", cos_res, 0.76484218728448842625585999019186495L, MAX_ULP, 0, 0);
}

static void
tan_test (void)
{
  errno = 0;
  FUNC(tan) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("tan (0) == 0",  FUNC(tan) (0), 0, 0, 0, 0);
  check_float ("tan (-0) == -0",  FUNC(tan) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("tan (inf) == NaN plus invalid exception",  FUNC(tan) (plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("tan (-inf) == NaN plus invalid exception",  FUNC(tan) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("tan (NaN) == NaN",  FUNC(tan) (nan_value), nan_value, 0, 0, 0);

  check_float ("tan (pi/4) == 1",  FUNC(tan) (M_PI_4l), 1, MAX_ULP, 0, 0);
  check_float ("tan (0.7) == 0.84228838046307944812813500221293775",  FUNC(tan) (0.7L), 0.84228838046307944812813500221293775L, MAX_ULP, 0, 0);
}

static void
acosh_test (void)
{
  errno = 0;
  FUNC(acosh) (7);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("acosh (inf) == inf",  FUNC(acosh) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("acosh (-inf) == NaN plus invalid exception",  FUNC(acosh) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);

  /* x < 1:  */
  check_float ("acosh (-1.1) == NaN plus invalid exception",  FUNC(acosh) (-1.1L), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("acosh (1) == 0",  FUNC(acosh) (1), 0, 0, 0, 0);
  check_float ("acosh (7) == 2.633915793849633417250092694615937",  FUNC(acosh) (7), 2.633915793849633417250092694615937L, MAX_ULP, 0, 0);
}

static void
asinh_test (void)
{
  errno = 0;
  FUNC(asinh) (0.7L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("asinh (0) == 0",  FUNC(asinh) (0), 0, 0, 0, 0);
  check_float ("asinh (-0) == -0",  FUNC(asinh) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("asinh (inf) == inf",  FUNC(asinh) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("asinh (-inf) == -inf",  FUNC(asinh) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("asinh (NaN) == NaN",  FUNC(asinh) (nan_value), nan_value, 0, 0, 0);
  check_float ("asinh (0.7) == 0.652666566082355786",  FUNC(asinh) (0.7L), 0.652666566082355786L, MAX_ULP, 0, 0);
}

static void
atanh_test (void)
{
  errno = 0;
  FUNC(atanh) (0.7L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();


  check_float ("atanh (0) == 0",  FUNC(atanh) (0), 0, 0, 0, 0);
  check_float ("atanh (-0) == -0",  FUNC(atanh) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("atanh (1) == inf plus division by zero exception",  FUNC(atanh) (1), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("atanh (-1) == -inf plus division by zero exception",  FUNC(atanh) (-1), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("atanh (NaN) == NaN",  FUNC(atanh) (nan_value), nan_value, 0, 0, 0);

  /* atanh (x) == NaN plus invalid exception if |x| > 1.  */
  check_float ("atanh (1.1) == NaN plus invalid exception",  FUNC(atanh) (1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("atanh (-1.1) == NaN plus invalid exception",  FUNC(atanh) (-1.1L), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("atanh (0.7) == 0.8673005276940531944",  FUNC(atanh) (0.7L), 0.8673005276940531944L, MAX_ULP, 0, 0);
}

static void
cosh_test (void)
{
  errno = 0;
  FUNC(cosh) (0.7L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();
  check_float ("cosh (0) == 1",  FUNC(cosh) (0), 1, 0, 0, 0);
  check_float ("cosh (-0) == 1",  FUNC(cosh) (minus_zero), 1, 0, 0, 0);

  check_float ("cosh (inf) == inf",  FUNC(cosh) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("cosh (-inf) == inf",  FUNC(cosh) (minus_infty), plus_infty, 0, 0, 0);
  check_float ("cosh (NaN) == NaN",  FUNC(cosh) (nan_value), nan_value, 0, 0, 0);

  check_float ("cosh (0.7) == 1.255169005630943018",  FUNC(cosh) (0.7L), 1.255169005630943018L, MAX_ULP, 0, 0);
}

static void
sinh_test (void)
{
  errno = 0;
  FUNC(sinh) (0.7L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();
  check_float ("sinh (0) == 0",  FUNC(sinh) (0), 0, 0, 0, 0);
  check_float ("sinh (-0) == -0",  FUNC(sinh) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("sinh (inf) == inf",  FUNC(sinh) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("sinh (-inf) == -inf",  FUNC(sinh) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("sinh (NaN) == NaN",  FUNC(sinh) (nan_value), nan_value, 0, 0, 0);

  check_float ("sinh (0.7) == 0.75858370183953350346",  FUNC(sinh) (0.7L), 0.75858370183953350346L, MAX_ULP, 0, 0);
  check_float ("sinh (0x8p-32) == 1.86264514923095703232705808926175479e-9",  FUNC(sinh) (0x8p-32L), 1.86264514923095703232705808926175479e-9L, 0, 0, 0);
}

static void
tanh_test (void)
{
  errno = 0;
  FUNC(tanh) (0.7L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("tanh (0) == 0",  FUNC(tanh) (0), 0, 0, 0, 0);
  check_float ("tanh (-0) == -0",  FUNC(tanh) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("tanh (inf) == 1",  FUNC(tanh) (plus_infty), 1, 0, 0, 0);
  check_float ("tanh (-inf) == -1",  FUNC(tanh) (minus_infty), -1, 0, 0, 0);
  check_float ("tanh (NaN) == NaN",  FUNC(tanh) (nan_value), nan_value, 0, 0, 0);

  check_float ("tanh (0.7) == 0.60436777711716349631",  FUNC(tanh) (0.7L), 0.60436777711716349631L, MAX_ULP, 0, 0);
  check_float ("tanh (-0.7) == -0.60436777711716349631",  FUNC(tanh) (-0.7L), -0.60436777711716349631L, MAX_ULP, 0, 0);

  check_float ("tanh (1.0) == 0.7615941559557648881194582826047935904",  FUNC(tanh) (1.0L), 0.7615941559557648881194582826047935904L, 0, 0, 0);
  check_float ("tanh (-1.0) == -0.7615941559557648881194582826047935904",  FUNC(tanh) (-1.0L), -0.7615941559557648881194582826047935904L, 0, 0, 0);

  /* 2^-57  */
  check_float ("tanh (6.938893903907228377647697925567626953125e-18) == 6.938893903907228377647697925567626953125e-18",  FUNC(tanh) (6.938893903907228377647697925567626953125e-18L), 6.938893903907228377647697925567626953125e-18L, 0, 0, 0);
}

static void
exp_test (void)
{
  errno = 0;
  FUNC(exp) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("exp (0) == 1",  FUNC(exp) (0), 1, 0, 0, 0);
  check_float ("exp (-0) == 1",  FUNC(exp) (minus_zero), 1, 0, 0, 0);

  check_float ("exp (inf) == inf",  FUNC(exp) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("exp (-inf) == 0",  FUNC(exp) (minus_infty), 0, 0, 0, 0);
  check_float ("exp (NaN) == NaN",  FUNC(exp) (nan_value), nan_value, 0, 0, 0);
  check_float ("exp (1) == e",  FUNC(exp) (1), M_El, 0, 0, 0);

  check_float ("exp (2) == e^2",  FUNC(exp) (2), M_E2l, 0, 0, 0);
  check_float ("exp (3) == e^3",  FUNC(exp) (3), M_E3l, 0, 0, 0);
  check_float ("exp (0.7) == 2.0137527074704765216",  FUNC(exp) (0.7L), 2.0137527074704765216L, MAX_ULP, 0, 0);
  check_float ("exp (50.0) == 5184705528587072464087.45332293348538",  FUNC(exp) (50.0L), 5184705528587072464087.45332293348538L, MAX_ULP, 0, 0);
}

static void
exp10_test (void)
{
  errno = 0;
  FUNC(exp10) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("exp10 (0) == 1",  FUNC(exp10) (0), 1, 0, 0, 0);
  check_float ("exp10 (-0) == 1",  FUNC(exp10) (minus_zero), 1, 0, 0, 0);

  check_float ("exp10 (inf) == inf",  FUNC(exp10) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("exp10 (-inf) == 0",  FUNC(exp10) (minus_infty), 0, 0, 0, 0);
  check_float ("exp10 (NaN) == NaN",  FUNC(exp10) (nan_value), nan_value, 0, 0, 0);
  check_float ("exp10 (3) == 1000",  FUNC(exp10) (3), 1000, MAX_ULP, 0, 0);
  check_float ("exp10 (-1) == 0.1",  FUNC(exp10) (-1), 0.1L, MAX_ULP, 0, 0);
  check_float ("exp10 (1e6) == inf",  FUNC(exp10) (1e6), plus_infty, 0, 0, 0);
  check_float ("exp10 (-1e6) == 0",  FUNC(exp10) (-1e6), 0, 0, 0, 0);
  check_float ("exp10 (0.7) == 5.0118723362727228500155418688494574",  FUNC(exp10) (0.7L), 5.0118723362727228500155418688494574L, MAX_ULP, 0, 0);
}

static void
exp2_test (void)
{
  errno = 0;
  FUNC(exp2) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("exp2 (0) == 1",  FUNC(exp2) (0), 1, 0, 0, 0);
  check_float ("exp2 (-0) == 1",  FUNC(exp2) (minus_zero), 1, 0, 0, 0);
  check_float ("exp2 (inf) == inf",  FUNC(exp2) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("exp2 (-inf) == 0",  FUNC(exp2) (minus_infty), 0, 0, 0, 0);
  check_float ("exp2 (NaN) == NaN",  FUNC(exp2) (nan_value), nan_value, 0, 0, 0);

  check_float ("exp2 (10) == 1024",  FUNC(exp2) (10), 1024, 0, 0, 0);
  check_float ("exp2 (-1) == 0.5",  FUNC(exp2) (-1), 0.5, 0, 0, 0);
  check_float ("exp2 (1e6) == inf",  FUNC(exp2) (1e6), plus_infty, 0, 0, 0);
  check_float ("exp2 (-1e6) == 0",  FUNC(exp2) (-1e6), 0, 0, 0, 0);
  check_float ("exp2 (0.7) == 1.6245047927124710452",  FUNC(exp2) (0.7L), 1.6245047927124710452L, MAX_ULP, 0, 0);
}

static void
expm1_test (void)
{
  errno = 0;
  FUNC(expm1) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("expm1 (0) == 0",  FUNC(expm1) (0), 0, 0, 0, 0);
  check_float ("expm1 (-0) == -0",  FUNC(expm1) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("expm1 (inf) == inf",  FUNC(expm1) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("expm1 (-inf) == -1",  FUNC(expm1) (minus_infty), -1, 0, 0, 0);
  check_float ("expm1 (NaN) == NaN",  FUNC(expm1) (nan_value), nan_value, 0, 0, 0);

  check_float ("expm1 (1) == M_El - 1.0",  FUNC(expm1) (1), M_El - 1.0, 1, 0, 0);
  check_float ("expm1 (0.7) == 1.0137527074704765216",  FUNC(expm1) (0.7L), 1.0137527074704765216L, MAX_ULP, 0, 0);
}

static void
frexp_test (void)
{
  int x;

  init_max_error ();

  check_float ("frexp (inf, &x) == inf",  FUNC(frexp) (plus_infty, &x), plus_infty, 0, 0, 0);
  check_float ("frexp (-inf, &x) == -inf",  FUNC(frexp) (minus_infty, &x), minus_infty, 0, 0, 0);
  check_float ("frexp (NaN, &x) == NaN",  FUNC(frexp) (nan_value, &x), nan_value, 0, 0, 0);

  check_float ("frexp (0.0, &x) == 0.0",  FUNC(frexp) (0.0, &x), 0.0, 0, 0, 0);
  check_int ("frexp (0.0, &x) sets x to 0.0", x, 0.0, 0, 0, 0);
  check_float ("frexp (-0, &x) == -0",  FUNC(frexp) (minus_zero, &x), minus_zero, 0, 0, 0);
  check_int ("frexp (-0, &x) sets x to 0.0", x, 0.0, 0, 0, 0);

  check_float ("frexp (12.8, &x) == 0.8",  FUNC(frexp) (12.8L, &x), 0.8L, 0, 0, 0);
  check_int ("frexp (12.8, &x) sets x to 4", x, 4, 0, 0, 0);
  check_float ("frexp (-27.34, &x) == -0.854375",  FUNC(frexp) (-27.34L, &x), -0.854375L, 0, 0, 0);
  check_int ("frexp (-27.34, &x) sets x to 5", x, 5, 0, 0, 0);
}

static void
ldexp_test (void)
{
  check_float ("ldexp (0, 0) == 0",  FUNC(ldexp) (0, 0), 0, 0, 0, 0);
  check_float ("ldexp (-0, 0) == -0",  FUNC(ldexp) (minus_zero, 0), minus_zero, 0, 0, 0);

  check_float ("ldexp (inf, 1) == inf",  FUNC(ldexp) (plus_infty, 1), plus_infty, 0, 0, 0);
  check_float ("ldexp (-inf, 1) == -inf",  FUNC(ldexp) (minus_infty, 1), minus_infty, 0, 0, 0);
  check_float ("ldexp (NaN, 1) == NaN",  FUNC(ldexp) (nan_value, 1), nan_value, 0, 0, 0);

  check_float ("ldexp (0.8, 4) == 12.8",  FUNC(ldexp) (0.8L, 4), 12.8L, 0, 0, 0);
  check_float ("ldexp (-0.854375, 5) == -27.34",  FUNC(ldexp) (-0.854375L, 5), -27.34L, 0, 0, 0);

  /* ldexp (x, 0) == x.  */
  check_float ("ldexp (1.0, 0) == 1.0",  FUNC(ldexp) (1.0L, 0L), 1.0L, 0, 0, 0);
}

static void
log_test (void)
{
  errno = 0;
  FUNC(log) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;
  init_max_error ();

  check_float ("log (0) == -inf plus division by zero exception",  FUNC(log) (0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("log (-0) == -inf plus division by zero exception",  FUNC(log) (minus_zero), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_float ("log (1) == 0",  FUNC(log) (1), 0, 0, 0, 0);

  check_float ("log (-1) == NaN plus invalid exception",  FUNC(log) (-1), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("log (inf) == inf",  FUNC(log) (plus_infty), plus_infty, 0, 0, 0);

  check_float ("log (e) == 1",  FUNC(log) (M_El), 1, MAX_ULP, 0, 0);
  check_float ("log (1.0 / M_El) == -1",  FUNC(log) (1.0 / M_El), -1, MAX_ULP, 0, 0);
  check_float ("log (2) == M_LN2l",  FUNC(log) (2), M_LN2l, 0, 0, 0);
  check_float ("log (10) == M_LN10l",  FUNC(log) (10), M_LN10l, 0, 0, 0);
  check_float ("log (0.7) == -0.35667494393873237891263871124118447",  FUNC(log) (0.7L), -0.35667494393873237891263871124118447L, MAX_ULP, 0, 0);
}

static void
log10_test (void)
{
  errno = 0;
  FUNC(log10) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("log10 (0) == -inf plus division by zero exception",  FUNC(log10) (0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("log10 (-0) == -inf plus division by zero exception",  FUNC(log10) (minus_zero), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_float ("log10 (1) == 0",  FUNC(log10) (1), 0, 0, 0, 0);

  /* log10 (x) == NaN plus invalid exception if x < 0.  */
  check_float ("log10 (-1) == NaN plus invalid exception",  FUNC(log10) (-1), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("log10 (inf) == inf",  FUNC(log10) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("log10 (NaN) == NaN",  FUNC(log10) (nan_value), nan_value, 0, 0, 0);

  check_float ("log10 (0.1) == -1",  FUNC(log10) (0.1L), -1, 0, 0, 0);
  check_float ("log10 (10.0) == 1",  FUNC(log10) (10.0), 1, 0, 0, 0);
  check_float ("log10 (100.0) == 2",  FUNC(log10) (100.0), 2, 0, 0, 0);
  check_float ("log10 (10000.0) == 4",  FUNC(log10) (10000.0), 4, 0, 0, 0);
  check_float ("log10 (e) == log10(e)",  FUNC(log10) (M_El), M_LOG10El, MAX_ULP, 0, 0);
  check_float ("log10 (0.7) == -0.15490195998574316929",  FUNC(log10) (0.7L), -0.15490195998574316929L, MAX_ULP, 0, 0);
}

static void
log1p_test (void)
{
  errno = 0;
  FUNC(log1p) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("log1p (0) == 0",  FUNC(log1p) (0), 0, 0, 0, 0);
  check_float ("log1p (-0) == -0",  FUNC(log1p) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("log1p (-1) == -inf plus division by zero exception",  FUNC(log1p) (-1), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("log1p (-2) == NaN plus invalid exception",  FUNC(log1p) (-2), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("log1p (inf) == inf",  FUNC(log1p) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("log1p (NaN) == NaN",  FUNC(log1p) (nan_value), nan_value, 0, 0, 0);

  check_float ("log1p (M_El - 1.0) == 1",  FUNC(log1p) (M_El - 1.0), 1, MAX_ULP, 0, 0);

  check_float ("log1p (-0.3) == -0.35667494393873237891263871124118447",  FUNC(log1p) (-0.3L), -0.35667494393873237891263871124118447L, MAX_ULP, 0, 0);
}

static void
log2_test (void)
{
  errno = 0;
  FUNC(log2) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("log2 (0) == -inf plus division by zero exception",  FUNC(log2) (0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("log2 (-0) == -inf plus division by zero exception",  FUNC(log2) (minus_zero), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_float ("log2 (1) == 0",  FUNC(log2) (1), 0, 0, 0, 0);

  check_float ("log2 (-1) == NaN plus invalid exception",  FUNC(log2) (-1), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("log2 (inf) == inf",  FUNC(log2) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("log2 (NaN) == NaN",  FUNC(log2) (nan_value), nan_value, 0, 0, 0);

  check_float ("log2 (e) == M_LOG2El",  FUNC(log2) (M_El), M_LOG2El, 0, 0, 0);
  check_float ("log2 (2.0) == 1",  FUNC(log2) (2.0), 1, 0, 0, 0);
  check_float ("log2 (16.0) == 4",  FUNC(log2) (16.0), 4, 0, 0, 0);
  check_float ("log2 (256.0) == 8",  FUNC(log2) (256.0), 8, 0, 0, 0);
  check_float ("log2 (0.7) == -0.51457317282975824043",  FUNC(log2) (0.7L), -0.51457317282975824043L, MAX_ULP, 0, 0);
}

static void
logb_test (void)
{
  init_max_error ();

  check_float ("logb (inf) == inf",  FUNC(logb) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("logb (-inf) == inf",  FUNC(logb) (minus_infty), plus_infty, 0, 0, 0);

  check_float ("logb (0) == -inf plus division by zero exception",  FUNC(logb) (0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_float ("logb (-0) == -inf plus division by zero exception",  FUNC(logb) (minus_zero), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("logb (NaN) == NaN",  FUNC(logb) (nan_value), nan_value, 0, 0, 0);

  check_float ("logb (1) == 0",  FUNC(logb) (1), 0, 0, 0, 0);
  check_float ("logb (e) == 1",  FUNC(logb) (M_El), 1, 0, 0, 0);
  check_float ("logb (1024) == 10",  FUNC(logb) (1024), 10, 0, 0, 0);
  check_float ("logb (-2000) == 10",  FUNC(logb) (-2000), 10, 0, 0, 0);
}

static void
modf_test (void)
{
  FLOAT x;

  init_max_error ();

  check_float ("modf (inf, &x) == 0",  FUNC(modf) (plus_infty, &x), 0, 0, 0, 0);
  check_float ("modf (inf, &x) sets x to plus_infty", x, plus_infty, 0, 0, 0);
  check_float ("modf (-inf, &x) == -0",  FUNC(modf) (minus_infty, &x), minus_zero, 0, 0, 0);
  check_float ("modf (-inf, &x) sets x to minus_infty", x, minus_infty, 0, 0, 0);
  check_float ("modf (NaN, &x) == NaN",  FUNC(modf) (nan_value, &x), nan_value, 0, 0, 0);
  check_float ("modf (NaN, &x) sets x to nan_value", x, nan_value, 0, 0, 0);
  check_float ("modf (0, &x) == 0",  FUNC(modf) (0, &x), 0, 0, 0, 0);
  check_float ("modf (0, &x) sets x to 0", x, 0, 0, 0, 0);
  check_float ("modf (1.5, &x) == 0.5",  FUNC(modf) (1.5, &x), 0.5, 0, 0, 0);
  check_float ("modf (1.5, &x) sets x to 1", x, 1, 0, 0, 0);
  check_float ("modf (2.5, &x) == 0.5",  FUNC(modf) (2.5, &x), 0.5, 0, 0, 0);
  check_float ("modf (2.5, &x) sets x to 2", x, 2, 0, 0, 0);
  check_float ("modf (-2.5, &x) == -0.5",  FUNC(modf) (-2.5, &x), -0.5, 0, 0, 0);
  check_float ("modf (-2.5, &x) sets x to -2", x, -2, 0, 0, 0);
  check_float ("modf (20, &x) == 0",  FUNC(modf) (20, &x), 0, 0, 0, 0);
  check_float ("modf (20, &x) sets x to 20", x, 20, 0, 0, 0);
  check_float ("modf (21, &x) == 0",  FUNC(modf) (21, &x), 0, 0, 0, 0);
  check_float ("modf (21, &x) sets x to 21", x, 21, 0, 0, 0);
  check_float ("modf (89.5, &x) == 0.5",  FUNC(modf) (89.5, &x), 0.5, 0, 0, 0);
  check_float ("modf (89.5, &x) sets x to 89", x, 89, 0, 0, 0);
}

static void
ilogb_test (void)
{
  init_max_error ();

  check_int ("ilogb (1) == 0",  FUNC(ilogb) (1), 0, 0, 0, 0);
  check_int ("ilogb (e) == 1",  FUNC(ilogb) (M_El), 1, 0, 0, 0);
  check_int ("ilogb (1024) == 10",  FUNC(ilogb) (1024), 10, 0, 0, 0);
  check_int ("ilogb (-2000) == 10",  FUNC(ilogb) (-2000), 10, 0, 0, 0);

  /* XXX We have a problem here: the standard does not tell us whether
     exceptions are allowed/required.  ignore them for now.  */

  check_int ("ilogb (0.0) == FP_ILOGB0 plus exceptions allowed",  FUNC(ilogb) (0.0), FP_ILOGB0, 0, 0, EXCEPTIONS_OK);
  check_int ("ilogb (NaN) == FP_ILOGBNAN plus exceptions allowed",  FUNC(ilogb) (nan_value), FP_ILOGBNAN, 0, 0, EXCEPTIONS_OK);
  check_int ("ilogb (inf) == INT_MAX plus exceptions allowed",  FUNC(ilogb) (plus_infty), INT_MAX, 0, 0, EXCEPTIONS_OK);
  check_int ("ilogb (-inf) == INT_MAX plus exceptions allowed",  FUNC(ilogb) (minus_infty), INT_MAX, 0, 0, EXCEPTIONS_OK);
}

static void
scalbn_test (void)
{

  init_max_error ();

  check_float ("scalbn (0, 0) == 0",  FUNC(scalbn) (0, 0), 0, 0, 0, 0);
  check_float ("scalbn (-0, 0) == -0",  FUNC(scalbn) (minus_zero, 0), minus_zero, 0, 0, 0);

  check_float ("scalbn (inf, 1) == inf",  FUNC(scalbn) (plus_infty, 1), plus_infty, 0, 0, 0);
  check_float ("scalbn (-inf, 1) == -inf",  FUNC(scalbn) (minus_infty, 1), minus_infty, 0, 0, 0);
  check_float ("scalbn (NaN, 1) == NaN",  FUNC(scalbn) (nan_value, 1), nan_value, 0, 0, 0);

  check_float ("scalbn (0.8, 4) == 12.8",  FUNC(scalbn) (0.8L, 4), 12.8L, 0, 0, 0);
  check_float ("scalbn (-0.854375, 5) == -27.34",  FUNC(scalbn) (-0.854375L, 5), -27.34L, 0, 0, 0);

  check_float ("scalbn (1, 0) == 1",  FUNC(scalbn) (1, 0L), 1, 0, 0, 0);
}

static void
scalbln_test (void)
{

  init_max_error ();

  check_float ("scalbln (0, 0) == 0",  FUNC(scalbln) (0, 0), 0, 0, 0, 0);
  check_float ("scalbln (-0, 0) == -0",  FUNC(scalbln) (minus_zero, 0), minus_zero, 0, 0, 0);

  check_float ("scalbln (inf, 1) == inf",  FUNC(scalbln) (plus_infty, 1), plus_infty, 0, 0, 0);
  check_float ("scalbln (-inf, 1) == -inf",  FUNC(scalbln) (minus_infty, 1), minus_infty, 0, 0, 0);
  check_float ("scalbln (NaN, 1) == NaN",  FUNC(scalbln) (nan_value, 1), nan_value, 0, 0, 0);

  check_float ("scalbln (0.8, 4) == 12.8",  FUNC(scalbln) (0.8L, 4), 12.8L, 0, 0, 0);
  check_float ("scalbln (-0.854375, 5) == -27.34",  FUNC(scalbln) (-0.854375L, 5), -27.34L, 0, 0, 0);

  check_float ("scalbln (1, 0) == 1",  FUNC(scalbln) (1, 0L), 1, 0, 0, 0);
}

static void
cbrt_test (void)
{
  errno = 0;
  FUNC(cbrt) (8);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("cbrt (0.0) == 0.0",  FUNC(cbrt) (0.0), 0.0, 0, 0, 0);
  check_float ("cbrt (-0) == -0",  FUNC(cbrt) (minus_zero), minus_zero, 0, 0, 0);

  check_float ("cbrt (inf) == inf",  FUNC(cbrt) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("cbrt (-inf) == -inf",  FUNC(cbrt) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("cbrt (NaN) == NaN",  FUNC(cbrt) (nan_value), nan_value, 0, 0, 0);

  check_float ("cbrt (-0.001) == -0.1",  FUNC(cbrt) (-0.001L), -0.1L, MAX_ULP, 0, 0);
  check_float ("cbrt (8) == 2",  FUNC(cbrt) (8), 2, 0, 0, 0);
  check_float ("cbrt (-27.0) == -3.0",  FUNC(cbrt) (-27.0), -3.0, MAX_ULP, 0, 0);
  check_float ("cbrt (0.970299) == 0.99",  FUNC(cbrt) (0.970299L), 0.99L, MAX_ULP, 0, 0);
  check_float ("cbrt (0.7) == 0.8879040017426007084",  FUNC(cbrt) (0.7L), 0.8879040017426007084L, MAX_ULP, 0, 0);
}

static void
fabs_test (void)
{
  init_max_error ();

  check_float ("fabs (0) == 0",  FUNC(fabs) ((FLOAT)0.0), 0, 0, 0, 0);
  check_float ("fabs (-0) == 0",  FUNC(fabs) (minus_zero), 0, 0, 0, 0);

  check_float ("fabs (inf) == inf",  FUNC(fabs) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("fabs (-inf) == inf",  FUNC(fabs) (minus_infty), plus_infty, 0, 0, 0);
  check_float ("fabs (NaN) == NaN",  FUNC(fabs) (nan_value), nan_value, 0, 0, 0);

  check_float ("fabs (38.0) == 38.0",  FUNC(fabs) ((FLOAT)38.0), 38.0, 0, 0, 0);
  check_float ("fabs (-e) == e",  FUNC(fabs) ((FLOAT)-M_El), M_El, 0, 0, 0);
}

static void
hypot_test (void)
{
  errno = 0;
  FUNC(hypot) (0.7L, 12.4L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("hypot (inf, 1) == inf plus sign of zero/inf not specified",  FUNC(hypot) (plus_infty, 1), plus_infty, 0, 0, IGNORE_ZERO_INF_SIGN);
  check_float ("hypot (-inf, 1) == inf plus sign of zero/inf not specified",  FUNC(hypot) (minus_infty, 1), plus_infty, 0, 0, IGNORE_ZERO_INF_SIGN);

  check_float ("hypot (inf, NaN) == inf",  FUNC(hypot) (plus_infty, nan_value), plus_infty, 0, 0, 0);
  check_float ("hypot (-inf, NaN) == inf",  FUNC(hypot) (minus_infty, nan_value), plus_infty, 0, 0, 0);
  check_float ("hypot (NaN, inf) == inf",  FUNC(hypot) (nan_value, plus_infty), plus_infty, 0, 0, 0);
  check_float ("hypot (NaN, -inf) == inf",  FUNC(hypot) (nan_value, minus_infty), plus_infty, 0, 0, 0);

  check_float ("hypot (NaN, NaN) == NaN",  FUNC(hypot) (nan_value, nan_value), nan_value, 0, 0, 0);

  /* hypot (x,y) == hypot (+-x, +-y)  */
  check_float ("hypot (0.7, 12.4) == 12.419742348374220601176836866763271",  FUNC(hypot) (0.7L, 12.4L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (-0.7, 12.4) == 12.419742348374220601176836866763271",  FUNC(hypot) (-0.7L, 12.4L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (0.7, -12.4) == 12.419742348374220601176836866763271",  FUNC(hypot) (0.7L, -12.4L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (-0.7, -12.4) == 12.419742348374220601176836866763271",  FUNC(hypot) (-0.7L, -12.4L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (12.4, 0.7) == 12.419742348374220601176836866763271",  FUNC(hypot) (12.4L, 0.7L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (-12.4, 0.7) == 12.419742348374220601176836866763271",  FUNC(hypot) (-12.4L, 0.7L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (12.4, -0.7) == 12.419742348374220601176836866763271",  FUNC(hypot) (12.4L, -0.7L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);
  check_float ("hypot (-12.4, -0.7) == 12.419742348374220601176836866763271",  FUNC(hypot) (-12.4L, -0.7L), 12.419742348374220601176836866763271L, MAX_ULP, 0, 0);

  /*  hypot (x,0) == fabs (x)  */
  check_float ("hypot (0.7, 0) == 0.7",  FUNC(hypot) (0.7L, 0), 0.7L, 0, 0, 0);
  check_float ("hypot (-0.7, 0) == 0.7",  FUNC(hypot) (-0.7L, 0), 0.7L, 0, 0, 0);
  check_float ("hypot (-5.7e7, 0) == 5.7e7",  FUNC(hypot) (-5.7e7, 0), 5.7e7L, 0, 0, 0);

  check_float ("hypot (0.7, 1.2) == 1.3892443989449804508432547041028554",  FUNC(hypot) (0.7L, 1.2L), 1.3892443989449804508432547041028554L, MAX_ULP, 0, 0);
}

static void
pow_test (void)
{

  errno = 0;
  FUNC(pow) (0, 0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("pow (0, 0) == 1",  FUNC(pow) (0, 0), 1, 0, 0, 0);
  check_float ("pow (0, -0) == 1",  FUNC(pow) (0, minus_zero), 1, 0, 0, 0);
  check_float ("pow (-0, 0) == 1",  FUNC(pow) (minus_zero, 0), 1, 0, 0, 0);
  check_float ("pow (-0, -0) == 1",  FUNC(pow) (minus_zero, minus_zero), 1, 0, 0, 0);

  check_float ("pow (10, 0) == 1",  FUNC(pow) (10, 0), 1, 0, 0, 0);
  check_float ("pow (10, -0) == 1",  FUNC(pow) (10, minus_zero), 1, 0, 0, 0);
  check_float ("pow (-10, 0) == 1",  FUNC(pow) (-10, 0), 1, 0, 0, 0);
  check_float ("pow (-10, -0) == 1",  FUNC(pow) (-10, minus_zero), 1, 0, 0, 0);

  check_float ("pow (NaN, 0) == 1",  FUNC(pow) (nan_value, 0), 1, 0, 0, 0);
  check_float ("pow (NaN, -0) == 1",  FUNC(pow) (nan_value, minus_zero), 1, 0, 0, 0);


  check_float ("pow (1.1, inf) == inf",  FUNC(pow) (1.1L, plus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (inf, inf) == inf",  FUNC(pow) (plus_infty, plus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (-1.1, inf) == inf",  FUNC(pow) (-1.1L, plus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, inf) == inf",  FUNC(pow) (minus_infty, plus_infty), plus_infty, 0, 0, 0);

  check_float ("pow (0.9, inf) == 0",  FUNC(pow) (0.9L, plus_infty), 0, 0, 0, 0);
  check_float ("pow (1e-7, inf) == 0",  FUNC(pow) (1e-7L, plus_infty), 0, 0, 0, 0);
  check_float ("pow (-0.9, inf) == 0",  FUNC(pow) (-0.9L, plus_infty), 0, 0, 0, 0);
  check_float ("pow (-1e-7, inf) == 0",  FUNC(pow) (-1e-7L, plus_infty), 0, 0, 0, 0);

  check_float ("pow (1.1, -inf) == 0",  FUNC(pow) (1.1L, minus_infty), 0, 0, 0, 0);
  check_float ("pow (inf, -inf) == 0",  FUNC(pow) (plus_infty, minus_infty), 0, 0, 0, 0);
  check_float ("pow (-1.1, -inf) == 0",  FUNC(pow) (-1.1L, minus_infty), 0, 0, 0, 0);
  check_float ("pow (-inf, -inf) == 0",  FUNC(pow) (minus_infty, minus_infty), 0, 0, 0, 0);

  check_float ("pow (0.9, -inf) == inf",  FUNC(pow) (0.9L, minus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (1e-7, -inf) == inf",  FUNC(pow) (1e-7L, minus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (-0.9, -inf) == inf",  FUNC(pow) (-0.9L, minus_infty), plus_infty, 0, 0, 0);
  check_float ("pow (-1e-7, -inf) == inf",  FUNC(pow) (-1e-7L, minus_infty), plus_infty, 0, 0, 0);

  check_float ("pow (inf, 1e-7) == inf",  FUNC(pow) (plus_infty, 1e-7L), plus_infty, 0, 0, 0);
  check_float ("pow (inf, 1) == inf",  FUNC(pow) (plus_infty, 1), plus_infty, 0, 0, 0);
  check_float ("pow (inf, 1e7) == inf",  FUNC(pow) (plus_infty, 1e7L), plus_infty, 0, 0, 0);

  check_float ("pow (inf, -1e-7) == 0",  FUNC(pow) (plus_infty, -1e-7L), 0, 0, 0, 0);
  check_float ("pow (inf, -1) == 0",  FUNC(pow) (plus_infty, -1), 0, 0, 0, 0);
  check_float ("pow (inf, -1e7) == 0",  FUNC(pow) (plus_infty, -1e7L), 0, 0, 0, 0);

  check_float ("pow (-inf, 1) == -inf",  FUNC(pow) (minus_infty, 1), minus_infty, 0, 0, 0);
  check_float ("pow (-inf, 11) == -inf",  FUNC(pow) (minus_infty, 11), minus_infty, 0, 0, 0);
  check_float ("pow (-inf, 1001) == -inf",  FUNC(pow) (minus_infty, 1001), minus_infty, 0, 0, 0);

  check_float ("pow (-inf, 2) == inf",  FUNC(pow) (minus_infty, 2), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 12) == inf",  FUNC(pow) (minus_infty, 12), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 1002) == inf",  FUNC(pow) (minus_infty, 1002), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 0.1) == inf",  FUNC(pow) (minus_infty, 0.1L), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 1.1) == inf",  FUNC(pow) (minus_infty, 1.1L), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 11.1) == inf",  FUNC(pow) (minus_infty, 11.1L), plus_infty, 0, 0, 0);
  check_float ("pow (-inf, 1001.1) == inf",  FUNC(pow) (minus_infty, 1001.1L), plus_infty, 0, 0, 0);

  check_float ("pow (-inf, -1) == -0",  FUNC(pow) (minus_infty, -1), minus_zero, 0, 0, 0);
  check_float ("pow (-inf, -11) == -0",  FUNC(pow) (minus_infty, -11), minus_zero, 0, 0, 0);
  check_float ("pow (-inf, -1001) == -0",  FUNC(pow) (minus_infty, -1001), minus_zero, 0, 0, 0);

  check_float ("pow (-inf, -2) == 0",  FUNC(pow) (minus_infty, -2), 0, 0, 0, 0);
  check_float ("pow (-inf, -12) == 0",  FUNC(pow) (minus_infty, -12), 0, 0, 0, 0);
  check_float ("pow (-inf, -1002) == 0",  FUNC(pow) (minus_infty, -1002), 0, 0, 0, 0);
  check_float ("pow (-inf, -0.1) == 0",  FUNC(pow) (minus_infty, -0.1L), 0, 0, 0, 0);
  check_float ("pow (-inf, -1.1) == 0",  FUNC(pow) (minus_infty, -1.1L), 0, 0, 0, 0);
  check_float ("pow (-inf, -11.1) == 0",  FUNC(pow) (minus_infty, -11.1L), 0, 0, 0, 0);
  check_float ("pow (-inf, -1001.1) == 0",  FUNC(pow) (minus_infty, -1001.1L), 0, 0, 0, 0);

  check_float ("pow (NaN, NaN) == NaN",  FUNC(pow) (nan_value, nan_value), nan_value, 0, 0, 0);
  check_float ("pow (0, NaN) == NaN",  FUNC(pow) (0, nan_value), nan_value, 0, 0, 0);
  check_float ("pow (1, NaN) == 1",  FUNC(pow) (1, nan_value), 1, 0, 0, 0);
  check_float ("pow (-1, NaN) == NaN",  FUNC(pow) (-1, nan_value), nan_value, 0, 0, 0);
  check_float ("pow (NaN, 1) == NaN",  FUNC(pow) (nan_value, 1), nan_value, 0, 0, 0);
  check_float ("pow (NaN, -1) == NaN",  FUNC(pow) (nan_value, -1), nan_value, 0, 0, 0);

  /* pow (x, NaN) == NaN.  */
  check_float ("pow (3.0, NaN) == NaN",  FUNC(pow) (3.0, nan_value), nan_value, 0, 0, 0);

  check_float ("pow (1, inf) == 1",  FUNC(pow) (1, plus_infty), 1, 0, 0, 0);
  check_float ("pow (-1, inf) == 1",  FUNC(pow) (-1, plus_infty), 1, 0, 0, 0);
  check_float ("pow (1, -inf) == 1",  FUNC(pow) (1, minus_infty), 1, 0, 0, 0);
  check_float ("pow (-1, -inf) == 1",  FUNC(pow) (-1, minus_infty), 1, 0, 0, 0);

  check_float ("pow (-0.1, 1.1) == NaN plus invalid exception",  FUNC(pow) (-0.1L, 1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("pow (-0.1, -1.1) == NaN plus invalid exception",  FUNC(pow) (-0.1L, -1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("pow (-10.1, 1.1) == NaN plus invalid exception",  FUNC(pow) (-10.1L, 1.1L), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("pow (-10.1, -1.1) == NaN plus invalid exception",  FUNC(pow) (-10.1L, -1.1L), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("pow (0, -1) == inf plus division by zero exception",  FUNC(pow) (0, -1), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (0, -11) == inf plus division by zero exception",  FUNC(pow) (0, -11), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (-0, -1) == -inf plus division by zero exception",  FUNC(pow) (minus_zero, -1), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (-0, -11) == -inf plus division by zero exception",  FUNC(pow) (minus_zero, -11), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);

  check_float ("pow (0, -2) == inf plus division by zero exception",  FUNC(pow) (0, -2), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (0, -11.1) == inf plus division by zero exception",  FUNC(pow) (0, -11.1L), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (-0, -2) == inf plus division by zero exception",  FUNC(pow) (minus_zero, -2), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("pow (-0, -11.1) == inf plus division by zero exception",  FUNC(pow) (minus_zero, -11.1L), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);


  check_float ("pow (0, 1) == 0",  FUNC(pow) (0, 1), 0, 0, 0, 0);
  check_float ("pow (0, 11) == 0",  FUNC(pow) (0, 11), 0, 0, 0, 0);

  check_float ("pow (-0, 1) == -0",  FUNC(pow) (minus_zero, 1), minus_zero, 0, 0, 0);
  check_float ("pow (-0, 11) == -0",  FUNC(pow) (minus_zero, 11), minus_zero, 0, 0, 0);


  check_float ("pow (0, 2) == 0",  FUNC(pow) (0, 2), 0, 0, 0, 0);
  check_float ("pow (0, 11.1) == 0",  FUNC(pow) (0, 11.1L), 0, 0, 0, 0);


  check_float ("pow (-0, 2) == 0",  FUNC(pow) (minus_zero, 2), 0, 0, 0, 0);
  check_float ("pow (-0, 11.1) == 0",  FUNC(pow) (minus_zero, 11.1L), 0, 0, 0, 0);

  /* pow (x, +inf) == +inf for |x| > 1.  */
  check_float ("pow (1.5, inf) == inf",  FUNC(pow) (1.5, plus_infty), plus_infty, 0, 0, 0);

  /* pow (x, +inf) == +0 for |x| < 1.  */
  check_float ("pow (0.5, inf) == 0.0",  FUNC(pow) (0.5, plus_infty), 0.0, 0, 0, 0);

  /* pow (x, -inf) == +0 for |x| > 1.  */
  check_float ("pow (1.5, -inf) == 0.0",  FUNC(pow) (1.5, minus_infty), 0.0, 0, 0, 0);

  /* pow (x, -inf) == +inf for |x| < 1.  */
  check_float ("pow (0.5, -inf) == inf",  FUNC(pow) (0.5, minus_infty), plus_infty, 0, 0, 0);

  /* pow (+inf, y) == +inf for y > 0.  */
  check_float ("pow (inf, 2) == inf",  FUNC(pow) (plus_infty, 2), plus_infty, 0, 0, 0);

  /* pow (+inf, y) == +0 for y < 0.  */
  check_float ("pow (inf, -1) == 0.0",  FUNC(pow) (plus_infty, -1), 0.0, 0, 0, 0);

  /* pow (-inf, y) == -inf for y an odd integer > 0.  */
  check_float ("pow (-inf, 27) == -inf",  FUNC(pow) (minus_infty, 27), minus_infty, 0, 0, 0);

  /* pow (-inf, y) == +inf for y > 0 and not an odd integer.  */
  check_float ("pow (-inf, 28) == inf",  FUNC(pow) (minus_infty, 28), plus_infty, 0, 0, 0);

  /* pow (-inf, y) == -0 for y an odd integer < 0. */
  check_float ("pow (-inf, -3) == -0",  FUNC(pow) (minus_infty, -3), minus_zero, 0, 0, 0);
  /* pow (-inf, y) == +0 for y < 0 and not an odd integer.  */
  check_float ("pow (-inf, -2.0) == 0.0",  FUNC(pow) (minus_infty, -2.0), 0.0, 0, 0, 0);

  /* pow (+0, y) == +0 for y an odd integer > 0.  */
  check_float ("pow (0.0, 27) == 0.0",  FUNC(pow) (0.0, 27), 0.0, 0, 0, 0);

  /* pow (-0, y) == -0 for y an odd integer > 0.  */
  check_float ("pow (-0, 27) == -0",  FUNC(pow) (minus_zero, 27), minus_zero, 0, 0, 0);

  /* pow (+0, y) == +0 for y > 0 and not an odd integer.  */
  check_float ("pow (0.0, 4) == 0.0",  FUNC(pow) (0.0, 4), 0.0, 0, 0, 0);

  /* pow (-0, y) == +0 for y > 0 and not an odd integer.  */
  check_float ("pow (-0, 4) == 0.0",  FUNC(pow) (minus_zero, 4), 0.0, 0, 0, 0);

  check_float ("pow (0.7, 1.2) == 0.65180494056638638188",  FUNC(pow) (0.7L, 1.2L), 0.65180494056638638188L, MAX_ULP, 0, 0);

#if defined TEST_DOUBLE || defined TEST_LDOUBLE
  check_float ("pow (-7.49321e+133, -9.80818e+16) == 0",  FUNC(pow) (-7.49321e+133, -9.80818e+16), 0, 0, 0, 0);
#endif
}

static void
sqrt_test (void)
{
  errno = 0;
  FUNC(sqrt) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("sqrt (0) == 0",  FUNC(sqrt) (0), 0, 0, 0, 0);
  check_float ("sqrt (NaN) == NaN",  FUNC(sqrt) (nan_value), nan_value, 0, 0, 0);
  check_float ("sqrt (inf) == inf",  FUNC(sqrt) (plus_infty), plus_infty, 0, 0, 0);

  check_float ("sqrt (-0) == -0",  FUNC(sqrt) (minus_zero), minus_zero, 0, 0, 0);

  /* sqrt (x) == NaN plus invalid exception for x < 0.  */
  check_float ("sqrt (-1) == NaN plus invalid exception",  FUNC(sqrt) (-1), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sqrt (-inf) == NaN plus invalid exception",  FUNC(sqrt) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("sqrt (NaN) == NaN",  FUNC(sqrt) (nan_value), nan_value, 0, 0, 0);

  check_float ("sqrt (2209) == 47",  FUNC(sqrt) (2209), 47, 0, 0, 0);
  check_float ("sqrt (4) == 2",  FUNC(sqrt) (4), 2, 0, 0, 0);
  check_float ("sqrt (2) == M_SQRT2l",  FUNC(sqrt) (2), M_SQRT2l, 0, 0, 0);
  check_float ("sqrt (0.25) == 0.5",  FUNC(sqrt) (0.25), 0.5, 0, 0, 0);
  check_float ("sqrt (6642.25) == 81.5",  FUNC(sqrt) (6642.25), 81.5, 0, 0, 0);
  check_float ("sqrt (15239.9025) == 123.45",  FUNC(sqrt) (15239.9025L), 123.45L, MAX_ULP, 0, 0);
  check_float ("sqrt (0.7) == 0.83666002653407554797817202578518747",  FUNC(sqrt) (0.7L), 0.83666002653407554797817202578518747L, 0, 0, 0);
}

static void
erf_test (void)
{
  errno = 0;
  FUNC(erf) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("erf (0) == 0",  FUNC(erf) (0), 0, 0, 0, 0);
  check_float ("erf (-0) == -0",  FUNC(erf) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("erf (inf) == 1",  FUNC(erf) (plus_infty), 1, 0, 0, 0);
  check_float ("erf (-inf) == -1",  FUNC(erf) (minus_infty), -1, 0, 0, 0);
  check_float ("erf (NaN) == NaN",  FUNC(erf) (nan_value), nan_value, 0, 0, 0);

  check_float ("erf (0.7) == 0.67780119383741847297",  FUNC(erf) (0.7L), 0.67780119383741847297L, 0, 0, 0);

  check_float ("erf (1.2) == 0.91031397822963538024",  FUNC(erf) (1.2L), 0.91031397822963538024L, 0, 0, 0);
  check_float ("erf (2.0) == 0.99532226501895273416",  FUNC(erf) (2.0), 0.99532226501895273416L, 0, 0, 0);
  check_float ("erf (4.1) == 0.99999999329997234592",  FUNC(erf) (4.1L), 0.99999999329997234592L, 0, 0, 0);
  check_float ("erf (27) == 1.0",  FUNC(erf) (27), 1.0L, 0, 0, 0);
}


static void
erfc_test (void)
{
  errno = 0;
  FUNC(erfc) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("erfc (inf) == 0.0",  FUNC(erfc) (plus_infty), 0.0, 0, 0, 0);
  check_float ("erfc (-inf) == 2.0",  FUNC(erfc) (minus_infty), 2.0, 0, 0, 0);
  check_float ("erfc (0.0) == 1.0",  FUNC(erfc) (0.0), 1.0, 0, 0, 0);
  check_float ("erfc (-0) == 1.0",  FUNC(erfc) (minus_zero), 1.0, 0, 0, 0);
  check_float ("erfc (NaN) == NaN",  FUNC(erfc) (nan_value), nan_value, 0, 0, 0);

  check_float ("erfc (0.7) == 0.32219880616258152702",  FUNC(erfc) (0.7L), 0.32219880616258152702L, MAX_ULP, 0, 0);

  check_float ("erfc (1.2) == 0.089686021770364619762",  FUNC(erfc) (1.2L), 0.089686021770364619762L, MAX_ULP, 0, 0);
  check_float ("erfc (2.0) == 0.0046777349810472658379",  FUNC(erfc) (2.0), 0.0046777349810472658379L, MAX_ULP, 0, 0);
  check_float ("erfc (4.1) == 0.67000276540848983727e-8",  FUNC(erfc) (4.1L), 0.67000276540848983727e-8L, MAX_ULP, 0, 0);
  check_float ("erfc (9) == 0.41370317465138102381e-36",  FUNC(erfc) (9), 0.41370317465138102381e-36L, MAX_ULP, 0, 0);
}

#define gamma lgamma /* XXX scp XXX */
#define gammaf lgammaf /* XXX scp XXX */
static void
gamma_test (void)
{
  errno = 0;
  FUNC(gamma) (1);

  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;
  feclearexcept (FE_ALL_EXCEPT);

  init_max_error ();

  signgam = 0;
  check_float ("gamma (inf) == inf",  FUNC(gamma) (plus_infty), plus_infty, 0, 0, 0);
  signgam = 0;
  check_float ("gamma (0) == inf plus division by zero exception",  FUNC(gamma) (0), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  signgam = 0;
  check_float ("gamma (-3) == inf plus division by zero exception",  FUNC(gamma) (-3), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  signgam = 0;
  check_float ("gamma (-inf) == inf",  FUNC(gamma) (minus_infty), plus_infty, 0, 0, 0);
  signgam = 0;
  check_float ("gamma (NaN) == NaN",  FUNC(gamma) (nan_value), nan_value, 0, 0, 0);

  signgam = 0;
  check_float ("gamma (1) == 0",  FUNC(gamma) (1), 0, 0, 0, 0);
  check_int ("gamma (1) sets signgam to 1", signgam, 1, 0, 0, 0);
  signgam = 0;
  check_float ("gamma (3) == M_LN2l",  FUNC(gamma) (3), M_LN2l, 0, 0, 0);
  check_int ("gamma (3) sets signgam to 1", signgam, 1, 0, 0, 0);

  signgam = 0;
  check_float ("gamma (0.5) == log(sqrt(pi))",  FUNC(gamma) (0.5), M_LOG_SQRT_PIl, 0, 0, 0);
  check_int ("gamma (0.5) sets signgam to 1", signgam, 1, 0, 0, 0);
  signgam = 0;
  check_float ("gamma (-0.5) == log(2*sqrt(pi))",  FUNC(gamma) (-0.5), M_LOG_2_SQRT_PIl, MAX_ULP, 0, 0);
  check_int ("gamma (-0.5) sets signgam to -1", signgam, -1, 0, 0, 0);
}
#undef gamma /* XXX scp XXX */
#undef gammaf /* XXX scp XXX */

static void
lgamma_test (void)
{
  errno = 0;
  FUNC(lgamma) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;
  feclearexcept (FE_ALL_EXCEPT);

  init_max_error ();

  signgam = 0;
  check_float ("lgamma (inf) == inf",  FUNC(lgamma) (plus_infty), plus_infty, 0, 0, 0);
  signgam = 0;
  check_float ("lgamma (0) == inf plus division by zero exception",  FUNC(lgamma) (0), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  signgam = 0;
  check_float ("lgamma (NaN) == NaN",  FUNC(lgamma) (nan_value), nan_value, 0, 0, 0);

  /* lgamma (x) == +inf plus divide by zero exception for integer x <= 0.  */
  signgam = 0;
  check_float ("lgamma (-3) == inf plus division by zero exception",  FUNC(lgamma) (-3), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  signgam = 0;
  check_float ("lgamma (-inf) == inf",  FUNC(lgamma) (minus_infty), plus_infty, 0, 0, 0);

  signgam = 0;
  check_float ("lgamma (1) == 0",  FUNC(lgamma) (1), 0, 0, 0, 0);
  check_int ("lgamma (1) sets signgam to 1", signgam, 1, 0, 0, 0);

  signgam = 0;
  check_float ("lgamma (3) == M_LN2l",  FUNC(lgamma) (3), M_LN2l, 0, 0, 0);
  check_int ("lgamma (3) sets signgam to 1", signgam, 1, 0, 0, 0);

  signgam = 0;
  check_float ("lgamma (0.5) == log(sqrt(pi))",  FUNC(lgamma) (0.5), M_LOG_SQRT_PIl, 0, 0, 0);
  check_int ("lgamma (0.5) sets signgam to 1", signgam, 1, 0, 0, 0);
  signgam = 0;
  check_float ("lgamma (-0.5) == log(2*sqrt(pi))",  FUNC(lgamma) (-0.5), M_LOG_2_SQRT_PIl, MAX_ULP, 0, 0);
  check_int ("lgamma (-0.5) sets signgam to -1", signgam, -1, 0, 0, 0);
  signgam = 0;
  check_float ("lgamma (0.7) == 0.26086724653166651439",  FUNC(lgamma) (0.7L), 0.26086724653166651439L, MAX_ULP, 0, 0);
  check_int ("lgamma (0.7) sets signgam to 1", signgam, 1, 0, 0, 0);
  signgam = 0;
  check_float ("lgamma (1.2) == -0.853740900033158497197e-1",  FUNC(lgamma) (1.2L), -0.853740900033158497197e-1L, MAX_ULP, 0, 0);
  check_int ("lgamma (1.2) sets signgam to 1", signgam, 1, 0, 0, 0);
}

static void
tgamma_test (void)
{
  errno = 0;
  FUNC(tgamma) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;
  feclearexcept (FE_ALL_EXCEPT);

  init_max_error ();

  check_float ("tgamma (inf) == inf",  FUNC(tgamma) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("tgamma (0) == inf plus divide-by-zero",  FUNC(tgamma) (0), plus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("tgamma (-0) == inf plus divide-by-zero",  FUNC(tgamma) (minus_zero), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  /* tgamma (x) == NaN plus invalid exception for integer x <= 0.  */
  check_float ("tgamma (-2) == NaN plus invalid exception",  FUNC(tgamma) (-2), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("tgamma (-inf) == NaN plus invalid exception",  FUNC(tgamma) (minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("tgamma (NaN) == NaN",  FUNC(tgamma) (nan_value), nan_value, 0, 0, 0);

  check_float ("tgamma (0.5) == sqrt (pi)",  FUNC(tgamma) (0.5), M_SQRT_PIl, MAX_ULP, 0, 0);
  check_float ("tgamma (-0.5) == -2 sqrt (pi)",  FUNC(tgamma) (-0.5), -M_2_SQRT_PIl, MAX_ULP, 0, 0);

  check_float ("tgamma (1) == 1",  FUNC(tgamma) (1), 1, 0, 0, 0);
  check_float ("tgamma (4) == 6",  FUNC(tgamma) (4), 6, MAX_ULP, 0, 0);

  check_float ("tgamma (0.7) == 1.29805533264755778568",  FUNC(tgamma) (0.7L), 1.29805533264755778568L, MAX_ULP, 0, 0);
  check_float ("tgamma (1.2) == 0.91816874239976061064",  FUNC(tgamma) (1.2L), 0.91816874239976061064L, 0, 0, 0);
}

static void
ceil_test (void)
{
  init_max_error ();

  check_float ("ceil (0.0) == 0.0",  FUNC(ceil) (0.0), 0.0, 0, 0, 0);
  check_float ("ceil (-0) == -0",  FUNC(ceil) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("ceil (inf) == inf",  FUNC(ceil) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("ceil (-inf) == -inf",  FUNC(ceil) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("ceil (NaN) == NaN",  FUNC(ceil) (nan_value), nan_value, 0, 0, 0);

  check_float ("ceil (pi) == 4.0",  FUNC(ceil) (M_PIl), 4.0, 0, 0, 0);
  check_float ("ceil (-pi) == -3.0",  FUNC(ceil) (-M_PIl), -3.0, 0, 0, 0);
}

static void
floor_test (void)
{
  init_max_error ();

  check_float ("floor (0.0) == 0.0",  FUNC(floor) (0.0), 0.0, 0, 0, 0);
  check_float ("floor (-0) == -0",  FUNC(floor) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("floor (inf) == inf",  FUNC(floor) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("floor (-inf) == -inf",  FUNC(floor) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("floor (NaN) == NaN",  FUNC(floor) (nan_value), nan_value, 0, 0, 0);

  check_float ("floor (pi) == 3.0",  FUNC(floor) (M_PIl), 3.0, 0, 0, 0);
  check_float ("floor (-pi) == -4.0",  FUNC(floor) (-M_PIl), -4.0, 0, 0, 0);
}

static void
nearbyint_test (void)
{
  init_max_error ();

  check_float ("nearbyint (0.0) == 0.0",  FUNC(nearbyint) (0.0), 0.0, 0, 0, 0);
  check_float ("nearbyint (-0) == -0",  FUNC(nearbyint) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("nearbyint (inf) == inf",  FUNC(nearbyint) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("nearbyint (-inf) == -inf",  FUNC(nearbyint) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("nearbyint (NaN) == NaN",  FUNC(nearbyint) (nan_value), nan_value, 0, 0, 0);

  /* Default rounding mode is round to nearest.  */
  check_float ("nearbyint (0.5) == 0.0",  FUNC(nearbyint) (0.5), 0.0, 0, 0, 0);
  check_float ("nearbyint (1.5) == 2.0",  FUNC(nearbyint) (1.5), 2.0, 0, 0, 0);
  check_float ("nearbyint (-0.5) == -0",  FUNC(nearbyint) (-0.5), minus_zero, 0, 0, 0);
  check_float ("nearbyint (-1.5) == -2.0",  FUNC(nearbyint) (-1.5), -2.0, 0, 0, 0);
}

static void
rint_test (void)
{
  init_max_error ();

  check_float ("rint (0.0) == 0.0",  FUNC(rint) (0.0), 0.0, 0, 0, 0);
  check_float ("rint (-0) == -0",  FUNC(rint) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("rint (inf) == inf",  FUNC(rint) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("rint (-inf) == -inf",  FUNC(rint) (minus_infty), minus_infty, 0, 0, 0);

  /* Default rounding mode is round to even.  */
  check_float ("rint (0.5) == 0.0",  FUNC(rint) (0.5), 0.0, 0, 0, 0);
  check_float ("rint (1.5) == 2.0",  FUNC(rint) (1.5), 2.0, 0, 0, 0);
  check_float ("rint (2.5) == 2.0",  FUNC(rint) (2.5), 2.0, 0, 0, 0);
  check_float ("rint (3.5) == 4.0",  FUNC(rint) (3.5), 4.0, 0, 0, 0);
  check_float ("rint (4.5) == 4.0",  FUNC(rint) (4.5), 4.0, 0, 0, 0);
  check_float ("rint (-0.5) == -0.0",  FUNC(rint) (-0.5), -0.0, 0, 0, 0);
  check_float ("rint (-1.5) == -2.0",  FUNC(rint) (-1.5), -2.0, 0, 0, 0);
  check_float ("rint (-2.5) == -2.0",  FUNC(rint) (-2.5), -2.0, 0, 0, 0);
  check_float ("rint (-3.5) == -4.0",  FUNC(rint) (-3.5), -4.0, 0, 0, 0);
  check_float ("rint (-4.5) == -4.0",  FUNC(rint) (-4.5), -4.0, 0, 0, 0);
}

static void
lrint_test (void)
{
  /* XXX this test is incomplete.  We need to have a way to specifiy
     the rounding method and test the critical cases.  So far, only
     unproblematic numbers are tested.  */

  init_max_error ();

  check_long ("lrint (0.0) == 0",  FUNC(lrint) (0.0), 0, 0, 0, 0);
  check_long ("lrint (-0) == 0",  FUNC(lrint) (minus_zero), 0, 0, 0, 0);
  check_long ("lrint (0.2) == 0",  FUNC(lrint) (0.2L), 0, 0, 0, 0);
  check_long ("lrint (-0.2) == 0",  FUNC(lrint) (-0.2L), 0, 0, 0, 0);

  check_long ("lrint (1.4) == 1",  FUNC(lrint) (1.4L), 1, 0, 0, 0);
  check_long ("lrint (-1.4) == -1",  FUNC(lrint) (-1.4L), -1, 0, 0, 0);

  check_long ("lrint (8388600.3) == 8388600",  FUNC(lrint) (8388600.3L), 8388600, 0, 0, 0);
  check_long ("lrint (-8388600.3) == -8388600",  FUNC(lrint) (-8388600.3L), -8388600, 0, 0, 0);
}

static void
round_test (void)
{
  init_max_error ();

  check_float ("round (0) == 0",  FUNC(round) (0), 0, 0, 0, 0);
  check_float ("round (-0) == -0",  FUNC(round) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("round (0.2) == 0.0",  FUNC(round) (0.2L), 0.0, 0, 0, 0);
  check_float ("round (-0.2) == -0",  FUNC(round) (-0.2L), minus_zero, 0, 0, 0);
  check_float ("round (0.5) == 1.0",  FUNC(round) (0.5), 1.0, 0, 0, 0);
  check_float ("round (-0.5) == -1.0",  FUNC(round) (-0.5), -1.0, 0, 0, 0);
  check_float ("round (0.8) == 1.0",  FUNC(round) (0.8L), 1.0, 0, 0, 0);
  check_float ("round (-0.8) == -1.0",  FUNC(round) (-0.8L), -1.0, 0, 0, 0);
  check_float ("round (1.5) == 2.0",  FUNC(round) (1.5), 2.0, 0, 0, 0);
  check_float ("round (-1.5) == -2.0",  FUNC(round) (-1.5), -2.0, 0, 0, 0);
  check_float ("round (2097152.5) == 2097153",  FUNC(round) (2097152.5), 2097153, 0, 0, 0);
  check_float ("round (-2097152.5) == -2097153",  FUNC(round) (-2097152.5), -2097153, 0, 0, 0);
}

static void
lround_test (void)
{
  init_max_error ();

  check_long ("lround (0) == 0",  FUNC(lround) (0), 0, 0, 0, 0);
  check_long ("lround (-0) == 0",  FUNC(lround) (minus_zero), 0, 0, 0, 0);
  check_long ("lround (0.2) == 0.0",  FUNC(lround) (0.2L), 0.0, 0, 0, 0);
  check_long ("lround (-0.2) == 0",  FUNC(lround) (-0.2L), 0, 0, 0, 0);
  check_long ("lround (0.5) == 1",  FUNC(lround) (0.5), 1, 0, 0, 0);
  check_long ("lround (-0.5) == -1",  FUNC(lround) (-0.5), -1, 0, 0, 0);
  check_long ("lround (0.8) == 1",  FUNC(lround) (0.8L), 1, 0, 0, 0);
  check_long ("lround (-0.8) == -1",  FUNC(lround) (-0.8L), -1, 0, 0, 0);
  check_long ("lround (1.5) == 2",  FUNC(lround) (1.5), 2, 0, 0, 0);
  check_long ("lround (-1.5) == -2",  FUNC(lround) (-1.5), -2, 0, 0, 0);
  check_long ("lround (22514.5) == 22515",  FUNC(lround) (22514.5), 22515, 0, 0, 0);
  check_long ("lround (-22514.5) == -22515",  FUNC(lround) (-22514.5), -22515, 0, 0, 0);
#ifndef TEST_FLOAT
  check_long ("lround (2097152.5) == 2097153",  FUNC(lround) (2097152.5), 2097153, 0, 0, 0);
  check_long ("lround (-2097152.5) == -2097153",  FUNC(lround) (-2097152.5), -2097153, 0, 0, 0);
#endif
}

static void
trunc_test (void)
{
  init_max_error ();

  check_float ("trunc (inf) == inf",  FUNC(trunc) (plus_infty), plus_infty, 0, 0, 0);
  check_float ("trunc (-inf) == -inf",  FUNC(trunc) (minus_infty), minus_infty, 0, 0, 0);
  check_float ("trunc (NaN) == NaN",  FUNC(trunc) (nan_value), nan_value, 0, 0, 0);

  check_float ("trunc (0) == 0",  FUNC(trunc) (0), 0, 0, 0, 0);
  check_float ("trunc (-0) == -0",  FUNC(trunc) (minus_zero), minus_zero, 0, 0, 0);
  check_float ("trunc (0.625) == 0",  FUNC(trunc) (0.625), 0, 0, 0, 0);
  check_float ("trunc (-0.625) == -0",  FUNC(trunc) (-0.625), minus_zero, 0, 0, 0);
  check_float ("trunc (1) == 1",  FUNC(trunc) (1), 1, 0, 0, 0);
  check_float ("trunc (-1) == -1",  FUNC(trunc) (-1), -1, 0, 0, 0);
  check_float ("trunc (1.625) == 1",  FUNC(trunc) (1.625), 1, 0, 0, 0);
  check_float ("trunc (-1.625) == -1",  FUNC(trunc) (-1.625), -1, 0, 0, 0);

  check_float ("trunc (1048580.625) == 1048580",  FUNC(trunc) (1048580.625L), 1048580L, 0, 0, 0);
  check_float ("trunc (-1048580.625) == -1048580",  FUNC(trunc) (-1048580.625L), -1048580L, 0, 0, 0);

  check_float ("trunc (8388610.125) == 8388610.0",  FUNC(trunc) (8388610.125L), 8388610.0L, 0, 0, 0);
  check_float ("trunc (-8388610.125) == -8388610.0",  FUNC(trunc) (-8388610.125L), -8388610.0L, 0, 0, 0);

  check_float ("trunc (4294967296.625) == 4294967296.0",  FUNC(trunc) (4294967296.625L), 4294967296.0L, 0, 0, 0);
  check_float ("trunc (-4294967296.625) == -4294967296.0",  FUNC(trunc) (-4294967296.625L), -4294967296.0L, 0, 0, 0);
}

static void
fmod_test (void)
{
  errno = 0;
  FUNC(fmod) (6.5, 2.3L);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  /* fmod (+0, y) == +0 for y != 0.  */
  check_float ("fmod (0, 3) == 0",  FUNC(fmod) (0, 3), 0, 0, 0, 0);

  /* fmod (-0, y) == -0 for y != 0.  */
  check_float ("fmod (-0, 3) == -0",  FUNC(fmod) (minus_zero, 3), minus_zero, 0, 0, 0);

  /* fmod (+inf, y) == NaN plus invalid exception.  */
  check_float ("fmod (inf, 3) == NaN plus invalid exception",  FUNC(fmod) (plus_infty, 3), nan_value, 0, 0, INVALID_EXCEPTION);
  /* fmod (-inf, y) == NaN plus invalid exception.  */
  check_float ("fmod (-inf, 3) == NaN plus invalid exception",  FUNC(fmod) (minus_infty, 3), nan_value, 0, 0, INVALID_EXCEPTION);
  /* fmod (x, +0) == NaN plus invalid exception.  */
  check_float ("fmod (3, 0) == NaN plus invalid exception",  FUNC(fmod) (3, 0), nan_value, 0, 0, INVALID_EXCEPTION);
  /* fmod (x, -0) == NaN plus invalid exception.  */
  check_float ("fmod (3, -0) == NaN plus invalid exception",  FUNC(fmod) (3, minus_zero), nan_value, 0, 0, INVALID_EXCEPTION);

  /* fmod (x, +inf) == x for x not infinite.  */
  check_float ("fmod (3.0, inf) == 3.0",  FUNC(fmod) (3.0, plus_infty), 3.0, 0, 0, 0);
  /* fmod (x, -inf) == x for x not infinite.  */
  check_float ("fmod (3.0, -inf) == 3.0",  FUNC(fmod) (3.0, minus_infty), 3.0, 0, 0, 0);

  check_float ("fmod (NaN, NaN) == NaN",  FUNC(fmod) (nan_value, nan_value), nan_value, 0, 0, 0);

  check_float ("fmod (6.5, 2.3) == 1.9",  FUNC(fmod) (6.5, 2.3L), 1.9L, MAX_ULP, 0, 0);
  check_float ("fmod (-6.5, 2.3) == -1.9",  FUNC(fmod) (-6.5, 2.3L), -1.9L, MAX_ULP, 0, 0);
  check_float ("fmod (6.5, -2.3) == 1.9",  FUNC(fmod) (6.5, -2.3L), 1.9L, MAX_ULP, 0, 0);
  check_float ("fmod (-6.5, -2.3) == -1.9",  FUNC(fmod) (-6.5, -2.3L), -1.9L, MAX_ULP, 0, 0);
}

static void
remainder_test (void)
{
  errno = 0;
  FUNC(remainder) (1.625, 1.0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("remainder (1, 0) == NaN plus invalid exception",  FUNC(remainder) (1, 0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remainder (1, -0) == NaN plus invalid exception",  FUNC(remainder) (1, minus_zero), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remainder (inf, 1) == NaN plus invalid exception",  FUNC(remainder) (plus_infty, 1), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remainder (-inf, 1) == NaN plus invalid exception",  FUNC(remainder) (minus_infty, 1), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remainder (NaN, NaN) == NaN",  FUNC(remainder) (nan_value, nan_value), nan_value, 0, 0, 0);

  check_float ("remainder (1.625, 1.0) == -0.375",  FUNC(remainder) (1.625, 1.0), -0.375, 0, 0, 0);
  check_float ("remainder (-1.625, 1.0) == 0.375",  FUNC(remainder) (-1.625, 1.0), 0.375, 0, 0, 0);
  check_float ("remainder (1.625, -1.0) == -0.375",  FUNC(remainder) (1.625, -1.0), -0.375, 0, 0, 0);
  check_float ("remainder (-1.625, -1.0) == 0.375",  FUNC(remainder) (-1.625, -1.0), 0.375, 0, 0, 0);
  check_float ("remainder (5.0, 2.0) == 1.0",  FUNC(remainder) (5.0, 2.0), 1.0, 0, 0, 0);
  check_float ("remainder (3.0, 2.0) == -1.0",  FUNC(remainder) (3.0, 2.0), -1.0, 0, 0, 0);
}

static void
remquo_test (void)
{
  /* x is needed.  */
  int x;

  errno = 0;
  FUNC(remquo) (1.625, 1.0, &x);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  check_float ("remquo (1, 0, &x) == NaN plus invalid exception",  FUNC(remquo) (1, 0, &x), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remquo (1, -0, &x) == NaN plus invalid exception",  FUNC(remquo) (1, minus_zero, &x), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remquo (inf, 1, &x) == NaN plus invalid exception",  FUNC(remquo) (plus_infty, 1, &x), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remquo (-inf, 1, &x) == NaN plus invalid exception",  FUNC(remquo) (minus_infty, 1, &x), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("remquo (NaN, NaN, &x) == NaN",  FUNC(remquo) (nan_value, nan_value, &x), nan_value, 0, 0, 0);

  check_float ("remquo (1.625, 1.0, &x) == -0.375",  FUNC(remquo) (1.625, 1.0, &x), -0.375, 0, 0, 0);
  check_int ("remquo (1.625, 1.0, &x) sets x to 2", x, 2, 0, 0, 0);
  check_float ("remquo (-1.625, 1.0, &x) == 0.375",  FUNC(remquo) (-1.625, 1.0, &x), 0.375, 0, 0, 0);
  check_int ("remquo (-1.625, 1.0, &x) sets x to -2", x, -2, 0, 0, 0);
  check_float ("remquo (1.625, -1.0, &x) == -0.375",  FUNC(remquo) (1.625, -1.0, &x), -0.375, 0, 0, 0);
  check_int ("remquo (1.625, -1.0, &x) sets x to -2", x, -2, 0, 0, 0);
  check_float ("remquo (-1.625, -1.0, &x) == 0.375",  FUNC(remquo) (-1.625, -1.0, &x), 0.375, 0, 0, 0);
  check_int ("remquo (-1.625, -1.0, &x) sets x to 2", x, 2, 0, 0, 0);

  check_float ("remquo (5, 2, &x) == 1",  FUNC(remquo) (5, 2, &x), 1, 0, 0, 0);
  check_int ("remquo (5, 2, &x) sets x to 2", x, 2, 0, 0, 0);
  check_float ("remquo (3, 2, &x) == -1",  FUNC(remquo) (3, 2, &x), -1, 0, 0, 0);
  check_int ("remquo (3, 2, &x) sets x to 2", x, 2, 0, 0, 0);
}

static void
copysign_test (void)
{
  init_max_error ();

  check_float ("copysign (0, 4) == 0",  FUNC(copysign) (0, 4), 0, 0, 0, 0);
  check_float ("copysign (0, -4) == -0",  FUNC(copysign) (0, -4), minus_zero, 0, 0, 0);
  check_float ("copysign (-0, 4) == 0",  FUNC(copysign) (minus_zero, 4), 0, 0, 0, 0);
  check_float ("copysign (-0, -4) == -0",  FUNC(copysign) (minus_zero, -4), minus_zero, 0, 0, 0);

  check_float ("copysign (inf, 0) == inf",  FUNC(copysign) (plus_infty, 0), plus_infty, 0, 0, 0);
  check_float ("copysign (inf, -0) == -inf",  FUNC(copysign) (plus_infty, minus_zero), minus_infty, 0, 0, 0);
  check_float ("copysign (-inf, 0) == inf",  FUNC(copysign) (minus_infty, 0), plus_infty, 0, 0, 0);
  check_float ("copysign (-inf, -0) == -inf",  FUNC(copysign) (minus_infty, minus_zero), minus_infty, 0, 0, 0);

  check_float ("copysign (0, inf) == 0",  FUNC(copysign) (0, plus_infty), 0, 0, 0, 0);
  check_float ("copysign (0, -0) == -0",  FUNC(copysign) (0, minus_zero), minus_zero, 0, 0, 0);
  check_float ("copysign (-0, inf) == 0",  FUNC(copysign) (minus_zero, plus_infty), 0, 0, 0, 0);
  check_float ("copysign (-0, -0) == -0",  FUNC(copysign) (minus_zero, minus_zero), minus_zero, 0, 0, 0);

  /* XXX More correctly we would have to check the sign of the NaN.  */
  check_float ("copysign (NaN, 0) == NaN",  FUNC(copysign) (nan_value, 0), nan_value, 0, 0, 0);
  check_float ("copysign (NaN, -0) == NaN",  FUNC(copysign) (nan_value, minus_zero), nan_value, 0, 0, 0);
  check_float ("copysign (-NaN, 0) == NaN",  FUNC(copysign) (-nan_value, 0), nan_value, 0, 0, 0);
  check_float ("copysign (-NaN, -0) == NaN",  FUNC(copysign) (-nan_value, minus_zero), nan_value, 0, 0, 0);
}

static void
nextafter_test (void)
{

  init_max_error ();

  check_float ("nextafter (0, 0) == 0",  FUNC(nextafter) (0, 0), 0, 0, 0, 0);
  check_float ("nextafter (-0, 0) == 0",  FUNC(nextafter) (minus_zero, 0), 0, 0, 0, 0);
  check_float ("nextafter (0, -0) == -0",  FUNC(nextafter) (0, minus_zero), minus_zero, 0, 0, 0);
  check_float ("nextafter (-0, -0) == -0",  FUNC(nextafter) (minus_zero, minus_zero), minus_zero, 0, 0, 0);

  check_float ("nextafter (9, 9) == 9",  FUNC(nextafter) (9, 9), 9, 0, 0, 0);
  check_float ("nextafter (-9, -9) == -9",  FUNC(nextafter) (-9, -9), -9, 0, 0, 0);
  check_float ("nextafter (inf, inf) == inf",  FUNC(nextafter) (plus_infty, plus_infty), plus_infty, 0, 0, 0);
  check_float ("nextafter (-inf, -inf) == -inf",  FUNC(nextafter) (minus_infty, minus_infty), minus_infty, 0, 0, 0);

  check_float ("nextafter (NaN, 1.1) == NaN",  FUNC(nextafter) (nan_value, 1.1L), nan_value, 0, 0, 0);
  check_float ("nextafter (1.1, NaN) == NaN",  FUNC(nextafter) (1.1L, nan_value), nan_value, 0, 0, 0);
  check_float ("nextafter (NaN, NaN) == NaN",  FUNC(nextafter) (nan_value, nan_value), nan_value, 0, 0, 0);

  /* XXX We need the hexadecimal FP number representation here for further
     tests.  */
}

static void
fdim_test (void)
{
  init_max_error ();

  check_float ("fdim (0, 0) == 0",  FUNC(fdim) (0, 0), 0, 0, 0, 0);
  check_float ("fdim (9, 0) == 9",  FUNC(fdim) (9, 0), 9, 0, 0, 0);
  check_float ("fdim (0, 9) == 0",  FUNC(fdim) (0, 9), 0, 0, 0, 0);
  check_float ("fdim (-9, 0) == 0",  FUNC(fdim) (-9, 0), 0, 0, 0, 0);
  check_float ("fdim (0, -9) == 9",  FUNC(fdim) (0, -9), 9, 0, 0, 0);

  check_float ("fdim (inf, 9) == inf",  FUNC(fdim) (plus_infty, 9), plus_infty, 0, 0, 0);
  check_float ("fdim (inf, -9) == inf",  FUNC(fdim) (plus_infty, -9), plus_infty, 0, 0, 0);
  check_float ("fdim (-inf, 9) == 0",  FUNC(fdim) (minus_infty, 9), 0, 0, 0, 0);
  check_float ("fdim (-inf, -9) == 0",  FUNC(fdim) (minus_infty, -9), 0, 0, 0, 0);
  check_float ("fdim (9, -inf) == inf",  FUNC(fdim) (9, minus_infty), plus_infty, 0, 0, 0);
  check_float ("fdim (-9, -inf) == inf",  FUNC(fdim) (-9, minus_infty), plus_infty, 0, 0, 0);
  check_float ("fdim (9, inf) == 0",  FUNC(fdim) (9, plus_infty), 0, 0, 0, 0);
  check_float ("fdim (-9, inf) == 0",  FUNC(fdim) (-9, plus_infty), 0, 0, 0, 0);

  check_float ("fdim (0, NaN) == NaN",  FUNC(fdim) (0, nan_value), nan_value, 0, 0, 0);
  check_float ("fdim (9, NaN) == NaN",  FUNC(fdim) (9, nan_value), nan_value, 0, 0, 0);
  check_float ("fdim (-9, NaN) == NaN",  FUNC(fdim) (-9, nan_value), nan_value, 0, 0, 0);
  check_float ("fdim (NaN, 9) == NaN",  FUNC(fdim) (nan_value, 9), nan_value, 0, 0, 0);
  check_float ("fdim (NaN, -9) == NaN",  FUNC(fdim) (nan_value, -9), nan_value, 0, 0, 0);
  check_float ("fdim (inf, NaN) == NaN",  FUNC(fdim) (plus_infty, nan_value), nan_value, 0, 0, 0);
  check_float ("fdim (-inf, NaN) == NaN",  FUNC(fdim) (minus_infty, nan_value), nan_value, 0, 0, 0);
  check_float ("fdim (NaN, inf) == NaN",  FUNC(fdim) (nan_value, plus_infty), nan_value, 0, 0, 0);
  check_float ("fdim (NaN, -inf) == NaN",  FUNC(fdim) (nan_value, minus_infty), nan_value, 0, 0, 0);
  check_float ("fdim (NaN, NaN) == NaN",  FUNC(fdim) (nan_value, nan_value), nan_value, 0, 0, 0);
}

static void
fmax_test (void)
{
  init_max_error ();

  check_float ("fmax (0, 0) == 0",  FUNC(fmax) (0, 0), 0, 0, 0, 0);
  check_float ("fmax (-0, -0) == -0",  FUNC(fmax) (minus_zero, minus_zero), minus_zero, 0, 0, 0);
  check_float ("fmax (9, 0) == 9",  FUNC(fmax) (9, 0), 9, 0, 0, 0);
  check_float ("fmax (0, 9) == 9",  FUNC(fmax) (0, 9), 9, 0, 0, 0);
  check_float ("fmax (-9, 0) == 0",  FUNC(fmax) (-9, 0), 0, 0, 0, 0);
  check_float ("fmax (0, -9) == 0",  FUNC(fmax) (0, -9), 0, 0, 0, 0);

  check_float ("fmax (inf, 9) == inf",  FUNC(fmax) (plus_infty, 9), plus_infty, 0, 0, 0);
  check_float ("fmax (0, inf) == inf",  FUNC(fmax) (0, plus_infty), plus_infty, 0, 0, 0);
  check_float ("fmax (-9, inf) == inf",  FUNC(fmax) (-9, plus_infty), plus_infty, 0, 0, 0);
  check_float ("fmax (inf, -9) == inf",  FUNC(fmax) (plus_infty, -9), plus_infty, 0, 0, 0);

  check_float ("fmax (-inf, 9) == 9",  FUNC(fmax) (minus_infty, 9), 9, 0, 0, 0);
  check_float ("fmax (-inf, -9) == -9",  FUNC(fmax) (minus_infty, -9), -9, 0, 0, 0);
  check_float ("fmax (9, -inf) == 9",  FUNC(fmax) (9, minus_infty), 9, 0, 0, 0);
  check_float ("fmax (-9, -inf) == -9",  FUNC(fmax) (-9, minus_infty), -9, 0, 0, 0);

  check_float ("fmax (0, NaN) == 0",  FUNC(fmax) (0, nan_value), 0, 0, 0, 0);
  check_float ("fmax (9, NaN) == 9",  FUNC(fmax) (9, nan_value), 9, 0, 0, 0);
  check_float ("fmax (-9, NaN) == -9",  FUNC(fmax) (-9, nan_value), -9, 0, 0, 0);
  check_float ("fmax (NaN, 0) == 0",  FUNC(fmax) (nan_value, 0), 0, 0, 0, 0);
  check_float ("fmax (NaN, 9) == 9",  FUNC(fmax) (nan_value, 9), 9, 0, 0, 0);
  check_float ("fmax (NaN, -9) == -9",  FUNC(fmax) (nan_value, -9), -9, 0, 0, 0);
  check_float ("fmax (inf, NaN) == inf",  FUNC(fmax) (plus_infty, nan_value), plus_infty, 0, 0, 0);
  check_float ("fmax (-inf, NaN) == -inf",  FUNC(fmax) (minus_infty, nan_value), minus_infty, 0, 0, 0);
  check_float ("fmax (NaN, inf) == inf",  FUNC(fmax) (nan_value, plus_infty), plus_infty, 0, 0, 0);
  check_float ("fmax (NaN, -inf) == -inf",  FUNC(fmax) (nan_value, minus_infty), minus_infty, 0, 0, 0);
  check_float ("fmax (NaN, NaN) == NaN",  FUNC(fmax) (nan_value, nan_value), nan_value, 0, 0, 0);
}


static void
fmin_test (void)
{
  init_max_error ();

  check_float ("fmin (0, 0) == 0",  FUNC(fmin) (0, 0), 0, 0, 0, 0);
  check_float ("fmin (-0, -0) == -0",  FUNC(fmin) (minus_zero, minus_zero), minus_zero, 0, 0, 0);
  check_float ("fmin (9, 0) == 0",  FUNC(fmin) (9, 0), 0, 0, 0, 0);
  check_float ("fmin (0, 9) == 0",  FUNC(fmin) (0, 9), 0, 0, 0, 0);
  check_float ("fmin (-9, 0) == -9",  FUNC(fmin) (-9, 0), -9, 0, 0, 0);
  check_float ("fmin (0, -9) == -9",  FUNC(fmin) (0, -9), -9, 0, 0, 0);

  check_float ("fmin (inf, 9) == 9",  FUNC(fmin) (plus_infty, 9), 9, 0, 0, 0);
  check_float ("fmin (9, inf) == 9",  FUNC(fmin) (9, plus_infty), 9, 0, 0, 0);
  check_float ("fmin (inf, -9) == -9",  FUNC(fmin) (plus_infty, -9), -9, 0, 0, 0);
  check_float ("fmin (-9, inf) == -9",  FUNC(fmin) (-9, plus_infty), -9, 0, 0, 0);
  check_float ("fmin (-inf, 9) == -inf",  FUNC(fmin) (minus_infty, 9), minus_infty, 0, 0, 0);
  check_float ("fmin (-inf, -9) == -inf",  FUNC(fmin) (minus_infty, -9), minus_infty, 0, 0, 0);
  check_float ("fmin (9, -inf) == -inf",  FUNC(fmin) (9, minus_infty), minus_infty, 0, 0, 0);
  check_float ("fmin (-9, -inf) == -inf",  FUNC(fmin) (-9, minus_infty), minus_infty, 0, 0, 0);

  check_float ("fmin (0, NaN) == 0",  FUNC(fmin) (0, nan_value), 0, 0, 0, 0);
  check_float ("fmin (9, NaN) == 9",  FUNC(fmin) (9, nan_value), 9, 0, 0, 0);
  check_float ("fmin (-9, NaN) == -9",  FUNC(fmin) (-9, nan_value), -9, 0, 0, 0);
  check_float ("fmin (NaN, 0) == 0",  FUNC(fmin) (nan_value, 0), 0, 0, 0, 0);
  check_float ("fmin (NaN, 9) == 9",  FUNC(fmin) (nan_value, 9), 9, 0, 0, 0);
  check_float ("fmin (NaN, -9) == -9",  FUNC(fmin) (nan_value, -9), -9, 0, 0, 0);
  check_float ("fmin (inf, NaN) == inf",  FUNC(fmin) (plus_infty, nan_value), plus_infty, 0, 0, 0);
  check_float ("fmin (-inf, NaN) == -inf",  FUNC(fmin) (minus_infty, nan_value), minus_infty, 0, 0, 0);
  check_float ("fmin (NaN, inf) == inf",  FUNC(fmin) (nan_value, plus_infty), plus_infty, 0, 0, 0);
  check_float ("fmin (NaN, -inf) == -inf",  FUNC(fmin) (nan_value, minus_infty), minus_infty, 0, 0, 0);
  check_float ("fmin (NaN, NaN) == NaN",  FUNC(fmin) (nan_value, nan_value), nan_value, 0, 0, 0);
}

static void
fma_test (void)
{
  init_max_error ();

  check_float ("fma (1.0, 2.0, 3.0) == 5.0",  FUNC(fma) (1.0, 2.0, 3.0), 5.0, 0, 0, 0);
  check_float ("fma (NaN, 2.0, 3.0) == NaN",  FUNC(fma) (nan_value, 2.0, 3.0), nan_value, 0, 0, 0);
  check_float ("fma (1.0, NaN, 3.0) == NaN",  FUNC(fma) (1.0, nan_value, 3.0), nan_value, 0, 0, 0);
  check_float ("fma (1.0, 2.0, NaN) == NaN plus invalid exception allowed",  FUNC(fma) (1.0, 2.0, nan_value), nan_value, 0, 0, INVALID_EXCEPTION_OK);
  check_float ("fma (inf, 0.0, NaN) == NaN plus invalid exception allowed",  FUNC(fma) (plus_infty, 0.0, nan_value), nan_value, 0, 0, INVALID_EXCEPTION_OK);
  check_float ("fma (-inf, 0.0, NaN) == NaN plus invalid exception allowed",  FUNC(fma) (minus_infty, 0.0, nan_value), nan_value, 0, 0, INVALID_EXCEPTION_OK);
  check_float ("fma (0.0, inf, NaN) == NaN plus invalid exception allowed",  FUNC(fma) (0.0, plus_infty, nan_value), nan_value, 0, 0, INVALID_EXCEPTION_OK);
  check_float ("fma (0.0, -inf, NaN) == NaN plus invalid exception allowed",  FUNC(fma) (0.0, minus_infty, nan_value), nan_value, 0, 0, INVALID_EXCEPTION_OK);
  check_float ("fma (inf, 0.0, 1.0) == NaN plus invalid exception",  FUNC(fma) (plus_infty, 0.0, 1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (-inf, 0.0, 1.0) == NaN plus invalid exception",  FUNC(fma) (minus_infty, 0.0, 1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (0.0, inf, 1.0) == NaN plus invalid exception",  FUNC(fma) (0.0, plus_infty, 1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (0.0, -inf, 1.0) == NaN plus invalid exception",  FUNC(fma) (0.0, minus_infty, 1.0), nan_value, 0, 0, INVALID_EXCEPTION);

  check_float ("fma (inf, inf, -inf) == NaN plus invalid exception",  FUNC(fma) (plus_infty, plus_infty, minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (-inf, inf, inf) == NaN plus invalid exception",  FUNC(fma) (minus_infty, plus_infty, plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (inf, -inf, inf) == NaN plus invalid exception",  FUNC(fma) (plus_infty, minus_infty, plus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("fma (-inf, -inf, -inf) == NaN plus invalid exception",  FUNC(fma) (minus_infty, minus_infty, minus_infty), nan_value, 0, 0, INVALID_EXCEPTION);
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

static void
j0_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(j0) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  init_max_error ();

  /* j0 is the Bessel function of the first kind of order 0 */
  check_float ("j0 (NaN) == NaN",  FUNC(j0) (nan_value), nan_value, 0, 0, 0);
  check_float ("j0 (inf) == 0",  FUNC(j0) (plus_infty), 0, 0, 0, 0);
  check_float ("j0 (-1.0) == 0.76519768655796655145",  FUNC(j0) (-1.0), 0.76519768655796655145L, 0, 0, 0);
  check_float ("j0 (0.0) == 1.0",  FUNC(j0) (0.0), 1.0, 0, 0, 0);
  check_float ("j0 (0.1) == 0.99750156206604003228",  FUNC(j0) (0.1L), 0.99750156206604003228L, MAX_ULP, 0, 0);
  check_float ("j0 (0.7) == 0.88120088860740528084",  FUNC(j0) (0.7L), 0.88120088860740528084L, 0, 0, 0);
  check_float ("j0 (1.0) == 0.76519768655796655145",  FUNC(j0) (1.0), 0.76519768655796655145L, 0, 0, 0);
  check_float ("j0 (1.5) == 0.51182767173591812875",  FUNC(j0) (1.5), 0.51182767173591812875L, 0, 0, 0);
  check_float ("j0 (2.0) == 0.22389077914123566805",  FUNC(j0) (2.0), 0.22389077914123566805L, MAX_ULP, 0, 0);
  check_float ("j0 (8.0) == 0.17165080713755390609",  FUNC(j0) (8.0), 0.17165080713755390609L, MAX_ULP, 0, 0);
  check_float ("j0 (10.0) == -0.24593576445134833520",  FUNC(j0) (10.0), -0.24593576445134833520L, MAX_ULP, 0, 0);
}


static void
j1_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(j1) (0);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  /* j1 is the Bessel function of the first kind of order 1 */

  init_max_error ();

  check_float ("j1 (NaN) == NaN",  FUNC(j1) (nan_value), nan_value, 0, 0, 0);
  check_float ("j1 (inf) == 0",  FUNC(j1) (plus_infty), 0, 0, 0, 0);

  check_float ("j1 (-1.0) == -0.44005058574493351596",  FUNC(j1) (-1.0), -0.44005058574493351596L, 0, 0, 0);
  check_float ("j1 (0.0) == 0.0",  FUNC(j1) (0.0), 0.0, 0, 0, 0);
  check_float ("j1 (0.1) == 0.049937526036241997556",  FUNC(j1) (0.1L), 0.049937526036241997556L, 0, 0, 0);
  check_float ("j1 (0.7) == 0.32899574154005894785",  FUNC(j1) (0.7L), 0.32899574154005894785L, 0, 0, 0);
  check_float ("j1 (1.0) == 0.44005058574493351596",  FUNC(j1) (1.0), 0.44005058574493351596L, 0, 0, 0);
  check_float ("j1 (1.5) == 0.55793650791009964199",  FUNC(j1) (1.5), 0.55793650791009964199L, 0, 0, 0);
  check_float ("j1 (2.0) == 0.57672480775687338720",  FUNC(j1) (2.0), 0.57672480775687338720L, MAX_ULP, 0, 0);
  check_float ("j1 (8.0) == 0.23463634685391462438",  FUNC(j1) (8.0), 0.23463634685391462438L, MAX_ULP, 0, 0);
  check_float ("j1 (10.0) == 0.043472746168861436670",  FUNC(j1) (10.0), 0.043472746168861436670L, MAX_ULP, 0, 0);
}

static void
jn_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(jn) (1, 1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  /* jn is the Bessel function of the first kind of order n.  */
  init_max_error ();

  /* jn (0, x) == j0 (x)  */
  check_float ("jn (0, NaN) == NaN",  FUNC(jn) (0, nan_value), nan_value, 0, 0, 0);
  check_float ("jn (0, inf) == 0",  FUNC(jn) (0, plus_infty), 0, 0, 0, 0);
  check_float ("jn (0, -1.0) == 0.76519768655796655145",  FUNC(jn) (0, -1.0), 0.76519768655796655145L, 0, 0, 0);
  check_float ("jn (0, 0.0) == 1.0",  FUNC(jn) (0, 0.0), 1.0, 0, 0, 0);
  check_float ("jn (0, 0.1) == 0.99750156206604003228",  FUNC(jn) (0, 0.1L), 0.99750156206604003228L, MAX_ULP, 0, 0);
  check_float ("jn (0, 0.7) == 0.88120088860740528084",  FUNC(jn) (0, 0.7L), 0.88120088860740528084L, 0, 0, 0);
  check_float ("jn (0, 1.0) == 0.76519768655796655145",  FUNC(jn) (0, 1.0), 0.76519768655796655145L, 0, 0, 0);
  check_float ("jn (0, 1.5) == 0.51182767173591812875",  FUNC(jn) (0, 1.5), 0.51182767173591812875L, 0, 0, 0);
  check_float ("jn (0, 2.0) == 0.22389077914123566805",  FUNC(jn) (0, 2.0), 0.22389077914123566805L, MAX_ULP, 0, 0);
  check_float ("jn (0, 8.0) == 0.17165080713755390609",  FUNC(jn) (0, 8.0), 0.17165080713755390609L, MAX_ULP, 0, 0);
  check_float ("jn (0, 10.0) == -0.24593576445134833520",  FUNC(jn) (0, 10.0), -0.24593576445134833520L, MAX_ULP, 0, 0);

  /* jn (1, x) == j1 (x)  */
  check_float ("jn (1, NaN) == NaN",  FUNC(jn) (1, nan_value), nan_value, 0, 0, 0);
  check_float ("jn (1, inf) == 0",  FUNC(jn) (1, plus_infty), 0, 0, 0, 0);

  check_float ("jn (1, -1.0) == -0.44005058574493351596",  FUNC(jn) (1, -1.0), -0.44005058574493351596L, 0, 0, 0);
  check_float ("jn (1, 0.0) == 0.0",  FUNC(jn) (1, 0.0), 0.0, 0, 0, 0);
  check_float ("jn (1, 0.1) == 0.049937526036241997556",  FUNC(jn) (1, 0.1L), 0.049937526036241997556L, 0, 0, 0);
  check_float ("jn (1, 0.7) == 0.32899574154005894785",  FUNC(jn) (1, 0.7L), 0.32899574154005894785L, 0, 0, 0);
  check_float ("jn (1, 1.0) == 0.44005058574493351596",  FUNC(jn) (1, 1.0), 0.44005058574493351596L, 0, 0, 0);
  check_float ("jn (1, 1.5) == 0.55793650791009964199",  FUNC(jn) (1, 1.5), 0.55793650791009964199L, 0, 0, 0);
  check_float ("jn (1, 2.0) == 0.57672480775687338720",  FUNC(jn) (1, 2.0), 0.57672480775687338720L, MAX_ULP, 0, 0);
  check_float ("jn (1, 8.0) == 0.23463634685391462438",  FUNC(jn) (1, 8.0), 0.23463634685391462438L, MAX_ULP, 0, 0);
  check_float ("jn (1, 10.0) == 0.043472746168861436670",  FUNC(jn) (1, 10.0), 0.043472746168861436670L, MAX_ULP, 0, 0);

  /* jn (3, x)  */
  check_float ("jn (3, NaN) == NaN",  FUNC(jn) (3, nan_value), nan_value, 0, 0, 0);
  check_float ("jn (3, inf) == 0",  FUNC(jn) (3, plus_infty), 0, 0, 0, 0);

  check_float ("jn (3, -1.0) == -0.019563353982668405919",  FUNC(jn) (3, -1.0), -0.019563353982668405919L, MAX_ULP, 0, 0);
  check_float ("jn (3, 0.0) == 0.0",  FUNC(jn) (3, 0.0), 0.0, 0, 0, 0);
  check_float ("jn (3, 0.1) == 0.000020820315754756261429",  FUNC(jn) (3, 0.1L), 0.000020820315754756261429L, MAX_ULP, 0, 0);
  check_float ("jn (3, 0.7) == 0.0069296548267508408077",  FUNC(jn) (3, 0.7L), 0.0069296548267508408077L, MAX_ULP, 0, 0);
  check_float ("jn (3, 1.0) == 0.019563353982668405919",  FUNC(jn) (3, 1.0), 0.019563353982668405919L, MAX_ULP, 0, 0);
  check_float ("jn (3, 2.0) == 0.12894324947440205110",  FUNC(jn) (3, 2.0), 0.12894324947440205110L, MAX_ULP, 0, 0);
  check_float ("jn (3, 10.0) == 0.058379379305186812343",  FUNC(jn) (3, 10.0), 0.058379379305186812343L, MAX_ULP, 0, 0);

  /*  jn (10, x)  */
  check_float ("jn (10, NaN) == NaN",  FUNC(jn) (10, nan_value), nan_value, 0, 0, 0);
  check_float ("jn (10, inf) == 0",  FUNC(jn) (10, plus_infty), 0, 0, 0, 0);

  check_float ("jn (10, -1.0) == 0.26306151236874532070e-9",  FUNC(jn) (10, -1.0), 0.26306151236874532070e-9L, MAX_ULP, 0, 0);
  check_float ("jn (10, 0.0) == 0.0",  FUNC(jn) (10, 0.0), 0.0, 0, 0, 0);
  check_float ("jn (10, 0.1) == 0.26905328954342155795e-19",  FUNC(jn) (10, 0.1L), 0.26905328954342155795e-19L, MAX_ULP, 0, 0);
  check_float ("jn (10, 0.7) == 0.75175911502153953928e-11",  FUNC(jn) (10, 0.7L), 0.75175911502153953928e-11L, MAX_ULP, 0, 0);
  check_float ("jn (10, 1.0) == 0.26306151236874532070e-9",  FUNC(jn) (10, 1.0), 0.26306151236874532070e-9L, MAX_ULP, 0, 0);
  check_float ("jn (10, 2.0) == 0.25153862827167367096e-6",  FUNC(jn) (10, 2.0), 0.25153862827167367096e-6L, MAX_ULP, 0, 0);
  check_float ("jn (10, 10.0) == 0.20748610663335885770",  FUNC(jn) (10, 10.0), 0.20748610663335885770L, MAX_ULP, 0, 0);
}

static void
y0_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(y0) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  /* y0 is the Bessel function of the second kind of order 0 */
  init_max_error ();

  check_float ("y0 (-1.0) == NaN",  FUNC(y0) (-1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("y0 (0.0) == -inf",  FUNC(y0) (0.0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("y0 (NaN) == NaN",  FUNC(y0) (nan_value), nan_value, 0, 0, 0);
  check_float ("y0 (inf) == 0",  FUNC(y0) (plus_infty), 0, 0, 0, 0);

  check_float ("y0 (0.1) == -1.5342386513503668441",  FUNC(y0) (0.1L), -1.5342386513503668441L, MAX_ULP, 0, 0);
  check_float ("y0 (0.7) == -0.19066492933739506743",  FUNC(y0) (0.7L), -0.19066492933739506743L, MAX_ULP, 0, 0);
  check_float ("y0 (1.0) == 0.088256964215676957983",  FUNC(y0) (1.0), 0.088256964215676957983L, MAX_ULP, 0, 0);
  check_float ("y0 (1.5) == 0.38244892379775884396",  FUNC(y0) (1.5), 0.38244892379775884396L, MAX_ULP, 0, 0);
  check_float ("y0 (2.0) == 0.51037567264974511960",  FUNC(y0) (2.0), 0.51037567264974511960L, MAX_ULP, 0, 0);
  check_float ("y0 (8.0) == 0.22352148938756622053",  FUNC(y0) (8.0), 0.22352148938756622053L, MAX_ULP, 0, 0);
  check_float ("y0 (10.0) == 0.055671167283599391424",  FUNC(y0) (10.0), 0.055671167283599391424L, MAX_ULP, 0, 0);
}


static void
y1_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(y1) (1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  /* y1 is the Bessel function of the second kind of order 1 */
  init_max_error ();

  check_float ("y1 (-1.0) == NaN",  FUNC(y1) (-1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("y1 (0.0) == -inf",  FUNC(y1) (0.0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("y1 (inf) == 0",  FUNC(y1) (plus_infty), 0, 0, 0, 0);
  check_float ("y1 (NaN) == NaN",  FUNC(y1) (nan_value), nan_value, 0, 0, 0);

  check_float ("y1 (0.1) == -6.4589510947020269877",  FUNC(y1) (0.1L), -6.4589510947020269877L, MAX_ULP, 0, 0);
  check_float ("y1 (0.7) == -1.1032498719076333697",  FUNC(y1) (0.7L), -1.1032498719076333697L, MAX_ULP, 0, 0);
  check_float ("y1 (1.0) == -0.78121282130028871655",  FUNC(y1) (1.0), -0.78121282130028871655L, MAX_ULP, 0, 0);
  check_float ("y1 (1.5) == -0.41230862697391129595",  FUNC(y1) (1.5), -0.41230862697391129595L, MAX_ULP, 0, 0);
  check_float ("y1 (2.0) == -0.10703243154093754689",  FUNC(y1) (2.0), -0.10703243154093754689L, MAX_ULP, 0, 0);
  check_float ("y1 (8.0) == -0.15806046173124749426",  FUNC(y1) (8.0), -0.15806046173124749426L, MAX_ULP, 0, 0);
  check_float ("y1 (10.0) == 0.24901542420695388392",  FUNC(y1) (10.0), 0.24901542420695388392L, MAX_ULP, 0, 0);
}

static void
yn_test (void)
{
  FLOAT s, c;
  errno = 0;
  FUNC (sincos) (0, &s, &c);
  if (errno == ENOSYS)
    /* Required function not implemented.  */
    return;
  FUNC(yn) (1, 1);
  if (errno == ENOSYS)
    /* Function not implemented.  */
    return;

  /* yn is the Bessel function of the second kind of order n */
  init_max_error ();

  /* yn (0, x) == y0 (x)  */
  check_float ("yn (0, -1.0) == NaN",  FUNC(yn) (0, -1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("yn (0, 0.0) == -inf",  FUNC(yn) (0, 0.0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("yn (0, NaN) == NaN",  FUNC(yn) (0, nan_value), nan_value, 0, 0, 0);
  check_float ("yn (0, inf) == 0",  FUNC(yn) (0, plus_infty), 0, 0, 0, 0);

  check_float ("yn (0, 0.1) == -1.5342386513503668441",  FUNC(yn) (0, 0.1L), -1.5342386513503668441L, MAX_ULP, 0, 0);
  check_float ("yn (0, 0.7) == -0.19066492933739506743",  FUNC(yn) (0, 0.7L), -0.19066492933739506743L, MAX_ULP, 0, 0);
  check_float ("yn (0, 1.0) == 0.088256964215676957983",  FUNC(yn) (0, 1.0), 0.088256964215676957983L, MAX_ULP, 0, 0);
  check_float ("yn (0, 1.5) == 0.38244892379775884396",  FUNC(yn) (0, 1.5), 0.38244892379775884396L, MAX_ULP, 0, 0);
  check_float ("yn (0, 2.0) == 0.51037567264974511960",  FUNC(yn) (0, 2.0), 0.51037567264974511960L, MAX_ULP, 0, 0);
  check_float ("yn (0, 8.0) == 0.22352148938756622053",  FUNC(yn) (0, 8.0), 0.22352148938756622053L, MAX_ULP, 0, 0);
  check_float ("yn (0, 10.0) == 0.055671167283599391424",  FUNC(yn) (0, 10.0), 0.055671167283599391424L, MAX_ULP, 0, 0);

  /* yn (1, x) == y1 (x)  */
  check_float ("yn (1, -1.0) == NaN",  FUNC(yn) (1, -1.0), nan_value, 0, 0, INVALID_EXCEPTION);
  check_float ("yn (1, 0.0) == -inf",  FUNC(yn) (1, 0.0), minus_infty, 0, 0, DIVIDE_BY_ZERO_EXCEPTION);
  check_float ("yn (1, inf) == 0",  FUNC(yn) (1, plus_infty), 0, 0, 0, 0);
  check_float ("yn (1, NaN) == NaN",  FUNC(yn) (1, nan_value), nan_value, 0, 0, 0);

  check_float ("yn (1, 0.1) == -6.4589510947020269877",  FUNC(yn) (1, 0.1L), -6.4589510947020269877L, MAX_ULP, 0, 0);
  check_float ("yn (1, 0.7) == -1.1032498719076333697",  FUNC(yn) (1, 0.7L), -1.1032498719076333697L, MAX_ULP, 0, 0);
  check_float ("yn (1, 1.0) == -0.78121282130028871655",  FUNC(yn) (1, 1.0), -0.78121282130028871655L, MAX_ULP, 0, 0);
  check_float ("yn (1, 1.5) == -0.41230862697391129595",  FUNC(yn) (1, 1.5), -0.41230862697391129595L, MAX_ULP, 0, 0);
  check_float ("yn (1, 2.0) == -0.10703243154093754689",  FUNC(yn) (1, 2.0), -0.10703243154093754689L, MAX_ULP, 0, 0);
  check_float ("yn (1, 8.0) == -0.15806046173124749426",  FUNC(yn) (1, 8.0), -0.15806046173124749426L, MAX_ULP, 0, 0);
  check_float ("yn (1, 10.0) == 0.24901542420695388392",  FUNC(yn) (1, 10.0), 0.24901542420695388392L, MAX_ULP, 0, 0);

  /* yn (3, x)  */
  check_float ("yn (3, inf) == 0",  FUNC(yn) (3, plus_infty), 0, 0, 0, 0);
  check_float ("yn (3, NaN) == NaN",  FUNC(yn) (3, nan_value), nan_value, 0, 0, 0);

  check_float ("yn (3, 0.1) == -5099.3323786129048894",  FUNC(yn) (3, 0.1L), -5099.3323786129048894L, MAX_ULP, 0, 0);
  check_float ("yn (3, 0.7) == -15.819479052819633505",  FUNC(yn) (3, 0.7L), -15.819479052819633505L, MAX_ULP, 0, 0);
  check_float ("yn (3, 1.0) == -5.8215176059647288478",  FUNC(yn) (3, 1.0), -5.8215176059647288478L, 0, 0, 0);
  check_float ("yn (3, 2.0) == -1.1277837768404277861",  FUNC(yn) (3, 2.0), -1.1277837768404277861L, MAX_ULP, 0, 0);
  check_float ("yn (3, 10.0) == -0.25136265718383732978",  FUNC(yn) (3, 10.0), -0.25136265718383732978L, MAX_ULP, 0, 0);

  /* yn (10, x)  */
  check_float ("yn (10, inf) == 0",  FUNC(yn) (10, plus_infty), 0, 0, 0, 0);
  check_float ("yn (10, NaN) == NaN",  FUNC(yn) (10, nan_value), nan_value, 0, 0, 0);

  check_float ("yn (10, 0.1) == -0.11831335132045197885e19",  FUNC(yn) (10, 0.1L), -0.11831335132045197885e19L, MAX_ULP, 0, 0);
  check_float ("yn (10, 0.7) == -0.42447194260703866924e10",  FUNC(yn) (10, 0.7L), -0.42447194260703866924e10L, MAX_ULP, 0, 0);
  check_float ("yn (10, 1.0) == -0.12161801427868918929e9",  FUNC(yn) (10, 1.0), -0.12161801427868918929e9L, MAX_ULP, 0, 0);
  check_float ("yn (10, 2.0) == -129184.54220803928264",  FUNC(yn) (10, 2.0), -129184.54220803928264L, MAX_ULP, 0, 0);
  check_float ("yn (10, 10.0) == -0.35981415218340272205",  FUNC(yn) (10, 10.0), -0.35981415218340272205L, MAX_ULP, 0, 0);

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

  /* Trigonometric functions:  */
  acos_test ();
  asin_test ();
  atan_test ();
  atan2_test ();
  cos_test ();
  sin_test ();
  sincos_test ();
  tan_test ();

  /* Hyperbolic functions:  */
  acosh_test ();
  asinh_test ();
  atanh_test ();
  cosh_test ();
  sinh_test ();
  tanh_test ();

  /* Exponential and logarithmic functions:  */
  exp_test ();
  exp10_test ();
  exp2_test ();
  expm1_test ();
  frexp_test ();
  ldexp_test ();
  log_test ();
  log10_test ();
  log1p_test ();
  log2_test ();
  logb_test ();
  modf_test ();
  ilogb_test ();
  scalbn_test ();
  scalbln_test ();

  /* Power and absolute value functions:  */
  cbrt_test ();
  fabs_test ();
  hypot_test ();
  pow_test ();
  sqrt_test ();

  /* Error and gamma functions:  */
  erf_test ();
  erfc_test ();
  gamma_test ();
  lgamma_test ();
  tgamma_test ();

  /* Nearest integer functions:  */
  ceil_test ();
  floor_test ();
  nearbyint_test ();
  rint_test ();
  lrint_test ();
  round_test ();
  lround_test ();
  trunc_test ();

  /* Remainder functions:  */
  fmod_test ();
  remainder_test ();
  remquo_test ();

  /* Manipulation functions:  */
  copysign_test ();
  nextafter_test ();

  /* maximum, minimum and positive difference functions */
  fdim_test ();
  fmax_test ();
  fmin_test ();

  /* Multiply and add:  */
  fma_test ();

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

  /* Bessel functions:  */
  j0_test ();
  j1_test ();
  jn_test ();
  y0_test ();
  y1_test ();
  yn_test ();

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