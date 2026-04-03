//! `math.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/math.h.html>.

use crate::platform::types::{c_double, c_float, c_int};

// TODO constants (some already defined in C)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/acos.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn acos(x: c_double) -> c_double {
    libm::acos(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/acos.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn acosf(x: c_float) -> c_float {
    libm::acosf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/acosh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn acosh(x: c_double) -> c_double {
    libm::acosh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/acosh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn acoshf(x: c_float) -> c_float {
    libm::acoshf(x)
}

// TODO acoshl, acosl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/asin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn asin(x: c_double) -> c_double {
    libm::asin(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/asin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn asinf(x: c_float) -> c_float {
    libm::asinf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/asinh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn asinh(x: c_double) -> c_double {
    libm::asinh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/asinh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn asinhf(x: c_float) -> c_float {
    libm::asinhf(x)
}

// TODO asinhl, asinl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atan.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atan(x: c_double) -> c_double {
    libm::atan(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atan2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atan2(y: c_double, x: c_double) -> c_double {
    libm::atan2(y, x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atan2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atan2f(y: c_float, x: c_float) -> c_float {
    libm::atan2f(y, x)
}

// TODO atan2l (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atan.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atanf(x: c_float) -> c_float {
    libm::atanf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atanh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atanh(x: c_double) -> c_double {
    libm::atanh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/atanh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn atanhf(x: c_float) -> c_float {
    libm::atanhf(x)
}

// TODO atanhl, atanl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cbrt.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn cbrt(x: c_double) -> c_double {
    libm::cbrt(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cbrt.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn cbrtf(x: c_float) -> c_float {
    libm::cbrtf(x)
}

// TODO cbrtl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ceil.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ceil(x: c_double) -> c_double {
    libm::ceil(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ceil.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ceilf(x: c_float) -> c_float {
    libm::ceilf(x)
}

// TODO ceill (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/copysign.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn copysign(x: c_double, y: c_double) -> c_double {
    libm::copysign(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/copysign.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn copysignf(x: c_float, y: c_float) -> c_float {
    libm::copysignf(x, y)
}

// TODO copysignl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cos.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn cos(x: c_double) -> c_double {
    libm::cos(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cos.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn cosf(x: c_float) -> c_float {
    libm::cosf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cosh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn cosh(x: c_double) -> c_double {
    libm::cosh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cosh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn coshf(x: c_float) -> c_float {
    libm::coshf(x)
}

// TODO coshl, cosl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/erf.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn erf(x: c_double) -> c_double {
    libm::erf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/erfc.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn erfc(x: c_double) -> c_double {
    libm::erfc(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/erfc.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn erfcf(x: c_float) -> c_float {
    libm::erfcf(x)
}

// TODO erfcl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/erf.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn erff(x: c_float) -> c_float {
    libm::erff(x)
}

// TODO erfl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn exp(x: c_double) -> c_double {
    libm::exp(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exp2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn exp2(x: c_double) -> c_double {
    libm::exp2(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exp2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn exp2f(x: c_float) -> c_float {
    libm::exp2f(x)
}

// TODO exp2l (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/exp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn expf(x: c_float) -> c_float {
    libm::expf(x)
}

// TODO expl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/expm1.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn expm1(x: c_double) -> c_double {
    libm::expm1(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/expm1.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn expm1f(x: c_float) -> c_float {
    libm::expm1f(x)
}

// TODO expm1l (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fabs.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fabs(x: c_double) -> c_double {
    libm::fabs(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fabsf.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fabsf(x: c_float) -> c_float {
    libm::fabsf(x)
}

// TODO fabsl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdim.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fdim(x: c_double, y: c_double) -> c_double {
    libm::fdim(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fdim.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fdimf(x: c_float, y: c_float) -> c_float {
    libm::fdimf(x, y)
}

// TODO fdiml (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/floor.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn floor(x: c_double) -> c_double {
    libm::floor(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/floor.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn floorf(x: c_float) -> c_float {
    libm::floorf(x)
}

// TODO floorl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fma(x: c_double, y: c_double, z: c_double) -> c_double {
    libm::fma(x, y, z)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmaf(x: c_float, y: c_float, z: c_float) -> c_float {
    libm::fmaf(x, y, z)
}

// TODO fmal (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmax.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmax(x: c_double, y: c_double) -> c_double {
    libm::fmax(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmax.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmaxf(x: c_float, y: c_float) -> c_float {
    libm::fmaxf(x, y)
}

// TODO fmaxl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmin(x: c_double, y: c_double) -> c_double {
    libm::fmin(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fminf(x: c_float, y: c_float) -> c_float {
    libm::fminf(x, y)
}

// TODO fminl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmod.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmod(x: c_double, y: c_double) -> c_double {
    libm::fmod(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fmod.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmodf(x: c_float, y: c_float) -> c_float {
    libm::fmodf(x, y)
}

// TODO fmodl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/frexp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn frexp(num: c_double, exp: *mut c_int) -> c_double {
    let (number, exponent) = libm::frexp(num);
    unsafe { *exp = exponent; }
    number
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/frexp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn frexpf(num: c_float, exp: *mut c_int) -> c_float {
    let (number, exponent) = libm::frexpf(num);
    unsafe { *exp = exponent; }
    number
}

// TODO frexpl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/hypot.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn hypot(x: c_double, y: c_double) -> c_double {
    libm::hypot(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/hypot.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn hypotf(x: c_float, y: c_float) -> c_float {
    libm::hypotf(x, y)
}

// TODO hypotl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ilogb.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ilogb(x: c_double) -> c_int {
    libm::ilogb(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ilogb.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ilogbf(x: c_float) -> c_int {
    libm::ilogbf(x)
}

// TODO ilogbl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/j0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn j0(x: c_double) -> c_double {
    libm::j0(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/j0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn j1(x: c_double) -> c_double {
    libm::j1(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/j0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn jn(n: c_int, x: c_double) -> c_double {
    libm::jn(n, x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldexp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ldexp(x: c_double, exp: c_int) -> c_double {
    libm::ldexp(x, exp)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ldexp.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn ldexpf(x: c_float, exp: c_int) -> c_float {
    libm::ldexpf(x, exp)
}

// TODO ldexpl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lgamma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn lgamma(x: c_double) -> c_double {
    libm::lgamma(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lgamma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn lgammaf(x: c_float) -> c_float {
    libm::lgammaf(x)
}

// TODO lgammal (long double)

// TODO llrint (c_double to c_longlong)
// TODO llrintf (c_float to c_longlong)
// TODO llrintl (long double to c_longlong)

// TODO llround (c_double to c_longlong)
// TODO llroundf (c_float to c_longlong)
// TODO llroundl (long double to c_longlong)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log(x: c_double) -> c_double {
    libm::log(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log10.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log10(x: c_double) -> c_double {
    libm::log10(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log10.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log10f(x: c_float) -> c_float {
    libm::log10f(x)
}

// TODO log10l (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log1p.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log1p(x: c_double) -> c_double {
    libm::log1p(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log1p.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log1pf(x: c_float) -> c_float {
    libm::log1pf(x)
}

// TODO log1pl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log2(x: c_double) -> c_double {
    libm::log2(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log2.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn log2f(x: c_float) -> c_float {
    libm::log2f(x)
}

// TODO log2l (long double)

// TODO logb (c_double)
// TODO logbf (c_float)
// TODO logbl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/log.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn logf(x: c_float) -> c_float {
    libm::logf(x)
}

// TODO logl (long double)

// TODO lrint (c_double)
// TODO lrintf (c_float)
// TODO lrintl (long double)

// TODO lround (c_double)
// TODO lroundf (c_float)
// TODO lroundl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/modf.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn modf(x: c_double, iptr: *mut c_double) -> c_double {
    let (integral, fractional) = libm::modf(x);
    unsafe { *iptr = integral; }
    fractional
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/modf.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn modff(value: c_float, iptr: *mut c_float) -> c_float {
    let (integral, fractional) = libm::modff(value);
    unsafe { *iptr = integral; }
    fractional
}

// TODO modfl (long double)

// TODO do we want to support quiet NaN or just return 0?
// TODO nan (c_double)
// TODO nanf (c_float)
// TODO nanl (long double)

// TODO nearbyint (c_double)
// TODO nearbyintf (c_float)
// TODO nearbyintl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nextafter.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn nextafter(x: c_double, y: c_double) -> c_double {
    libm::nextafter(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/nextafter.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn nextafterf(x: c_float, y: c_float) -> c_float {
    libm::nextafterf(x, y)
}

// TODO nextafterl (long double)

// TODO nexttoward (c_double)
// TODO nexttowardf (c_float)
// TODO nexttowardl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pow.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn pow(x: c_double, y: c_double) -> c_double {
    libm::pow(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pow.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn powf(x: c_float, y: c_float) -> c_float {
    libm::powf(x, y)
}

// TODO powl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/remainder.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn remainder(x: c_double, y: c_double) -> c_double {
    libm::remainder(x, y)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/remainder.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn remainderf(x: c_float, y: c_float) -> c_float {
    libm::remainderf(x, y)
}

// TODO remainderl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/remquo.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn remquo(x: c_double, y: c_double, quo: *mut c_int) -> c_double {
    let (remainder, quotient) = libm::remquo(x, y);
    unsafe { *quo = quotient; }
    remainder
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/remquo.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn remquof(x: c_float, y: c_float, quo: *mut c_int) -> c_float {
    let (remainder, quotient) = libm::remquof(x, y);
    unsafe { *quo = quotient; }
    remainder
}

// TODO remquol (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rint.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn rint(x: c_double) -> c_double {
    libm::rint(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/rint.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn rintf(x: c_float) -> c_float {
    libm::rintf(x)
}

// TODO rintl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/round.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn round(x: c_double) -> c_double {
    libm::round(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/round.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn roundf(x: c_float) -> c_float {
    libm::roundf(x)
}

// TODO roundl (long double)

// TODO scalbln (c_double, c_long)
// TODO scalblnf (c_float, c_long)
// TODO scalblnl (long double, c_long)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/scalbln.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn scalbn(x: c_double, n: c_int) -> c_double {
    libm::scalbn(x, n)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/scalbln.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn scalbnf(x: c_float, n: c_int) -> c_float {
    libm::scalbnf(x, n)
}

// TODO scalbnl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sin(x: c_double) -> c_double {
    libm::sin(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sin.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sinf(x: c_float) -> c_float {
    libm::sinf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sinh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sinh(x: c_double) -> c_double {
    libm::sinh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sinh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sinhf(x: c_float) -> c_float {
    libm::sinhf(x)
}

// TODO sinhl (long double)
// TODO sinl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sqrt.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqrt(x: c_double) -> c_double {
    libm::sqrt(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sqrt.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqrtf(x: c_float) -> c_float {
    libm::sqrtf(x)
}

// TODO sqrtl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tan.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tan(x: c_double) -> c_double {
    libm::tan(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tan.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tanf(x: c_float) -> c_float {
    libm::tanf(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tanh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tanh(x: c_double) -> c_double {
    libm::tanh(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tanh.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tanhf(x: c_float) -> c_float {
    libm::tanhf(x)
}

// TODO tanhl (long double)
// TODO tanl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tgamma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tgamma(x: c_double) -> c_double {
    libm::tgamma(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tgamma.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn tgammaf(x: c_float) -> c_float {
    libm::tgammaf(x)
}

// TODO tgammal (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/trunc.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn trunc(x: c_double) -> c_double {
    libm::trunc(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/trunc.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn truncf(x: c_float) -> c_float {
    libm::truncf(x)
}

// TODO truncl (long double)

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/y0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn y0(x: c_double) -> c_double {
    libm::y0(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/y0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn y1(x: c_double) -> c_double {
    libm::y1(x)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/y0.html>.
//#[unsafe(no_mangle)]
pub unsafe extern "C" fn yn(n: c_int, x: c_double) -> c_double {
    libm::yn(n, x)
}
