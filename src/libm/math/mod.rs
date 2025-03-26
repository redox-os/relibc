use crate::platform::types::{c_char, c_double, c_float, c_int, c_long, c_longlong};
use rust_libm;

#[no_mangle]
pub static mut signgam: c_int = 0;

pub const M_E: f64 = core::f64::consts::E;
pub const M_LOG2E: f64 = core::f64::consts::LOG2_E;
pub const M_LOG10E: f64 = core::f64::consts::LOG10_E;
pub const M_LN2: f64 = core::f64::consts::LN_2;
pub const M_LN10: f64 = core::f64::consts::LN_10;
pub const M_PI: f64 = core::f64::consts::PI;
pub const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
pub const M_PI_4: f64 = core::f64::consts::FRAC_PI_4;
pub const M_1_PI: f64 = core::f64::consts::FRAC_1_PI;
pub const M_2_PI: f64 = core::f64::consts::FRAC_2_PI;
pub const M_2_SQRTPI: f64 = core::f64::consts::FRAC_2_SQRT_PI;
pub const M_SQRT2: f64 = core::f64::consts::SQRT_2;
pub const M_SQRT1_2: f64 = core::f64::consts::FRAC_1_SQRT_2;

pub const MAXFLOAT: c_float = c_float::MAX;

pub const HUGE_VAL: c_double = c_double::INFINITY;
pub const HUGE_VALF: c_float = c_float::INFINITY;

pub const INFINITY: c_float = c_float::INFINITY;
pub const NAN: c_float = c_float::NAN;

pub const FP_FAST_FMA: c_int = 1;
pub const FP_FAST_FMAF: c_int = 1;

pub const FP_ILOGB0: c_int = c_int::MIN;
pub const FP_ILOGBNAN: c_int = c_int::MAX;

pub const MATH_ERRNO: c_int = 1;
pub const MATH_ERREXCEPT: c_int = 2;
pub const math_errhandling: c_int = 2;

pub type float_t = c_float;
pub type double_t = c_double;

#[no_mangle]
pub unsafe extern "C" fn acos(x: c_double) -> c_double {
    rust_libm::acos(x)
}

#[no_mangle]
pub unsafe extern "C" fn acosf(x: c_float) -> c_float {
    rust_libm::acosf(x)
}

#[no_mangle]
pub unsafe extern "C" fn acosh(x: c_double) -> c_double {
    rust_libm::acosh(x)
}

#[no_mangle]
pub unsafe extern "C" fn acoshf(x: c_float) -> c_float {
    rust_libm::acoshf(x)
}

#[no_mangle]
pub unsafe extern "C" fn asin(x: c_double) -> c_double {
    rust_libm::asin(x)
}

#[no_mangle]
pub unsafe extern "C" fn asinf(x: c_float) -> c_float {
    rust_libm::asinf(x)
}

#[no_mangle]
pub unsafe extern "C" fn asinh(x: c_double) -> c_double {
    rust_libm::asinh(x)
}

#[no_mangle]
pub unsafe extern "C" fn asinhf(x: c_float) -> c_float {
    rust_libm::asinhf(x)
}

#[no_mangle]
pub unsafe extern "C" fn atan(x: c_double) -> c_double {
    rust_libm::atan(x)
}

#[no_mangle]
pub unsafe extern "C" fn atan2(x: c_double, y: c_double) -> c_double {
    rust_libm::atan2(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn atan2f(x: c_float, y: c_float) -> c_float {
    rust_libm::atan2f(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn atanf(x: c_float) -> c_float {
    rust_libm::atanf(x)
}

#[no_mangle]
pub unsafe extern "C" fn atanh(x: c_double) -> c_double {
    rust_libm::atanh(x)
}

#[no_mangle]
pub unsafe extern "C" fn atanhf(x: c_float) -> c_float {
    rust_libm::atanhf(x)
}

#[no_mangle]
pub unsafe extern "C" fn cbrt(x: c_double) -> c_double {
    rust_libm::cbrt(x)
}

#[no_mangle]
pub unsafe extern "C" fn cbrtf(x: c_float) -> c_float {
    rust_libm::cbrtf(x)
}

#[no_mangle]
pub unsafe extern "C" fn ceil(x: c_double) -> c_double {
    rust_libm::ceil(x)
}

#[no_mangle]
pub unsafe extern "C" fn ceilf(x: c_float) -> c_float {
    rust_libm::ceilf(x)
}

#[no_mangle]
pub unsafe extern "C" fn copysign(x: c_double, y: c_double) -> c_double {
    rust_libm::copysign(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn copysignf(x: c_float, y: c_float) -> c_float {
    rust_libm::copysignf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn cos(x: c_double) -> c_double {
    rust_libm::cos(x)
}

#[no_mangle]
pub unsafe extern "C" fn cosf(x: c_float) -> c_float {
    rust_libm::cosf(x)
}

#[no_mangle]
pub unsafe extern "C" fn cosh(x: c_double) -> c_double {
    rust_libm::cosh(x)
}

#[no_mangle]
pub unsafe extern "C" fn coshf(x: c_float) -> c_float {
    rust_libm::coshf(x)
}

#[no_mangle]
pub unsafe extern "C" fn drem(x: f64, y: f64) -> f64 {
    remainder(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn dremf(x: c_float, y: c_float) -> c_float {
    remainderf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn erf(x: c_double) -> c_double {
    rust_libm::erf(x)
}

#[no_mangle]
pub unsafe extern "C" fn erfc(x: c_double) -> c_double {
    rust_libm::erfc(x)
}

#[no_mangle]
pub unsafe extern "C" fn erfcf(x: c_float) -> c_float {
    rust_libm::erfcf(x)
}

#[no_mangle]
pub unsafe extern "C" fn erff(x: c_float) -> c_float {
    rust_libm::erff(x)
}

#[no_mangle]
pub unsafe extern "C" fn exp(x: c_double) -> c_double {
    rust_libm::exp(x)
}

#[no_mangle]
pub unsafe extern "C" fn exp2(x: c_double) -> c_double {
    rust_libm::exp2(x)
}

#[no_mangle]
pub unsafe extern "C" fn exp2f(x: c_float) -> c_float {
    rust_libm::exp2f(x)
}

#[no_mangle]
pub unsafe extern "C" fn exp10(x: c_double) -> c_double {
    rust_libm::exp10(x)
}

#[no_mangle]
pub unsafe extern "C" fn exp10f(x: c_float) -> c_float {
    rust_libm::exp10f(x)
}

#[no_mangle]
pub unsafe extern "C" fn expf(x: c_float) -> c_float {
    rust_libm::expf(x)
}

#[no_mangle]
pub unsafe extern "C" fn expm1(x: c_double) -> c_double {
    rust_libm::expm1(x)
}

#[no_mangle]
pub unsafe extern "C" fn expm1f(x: c_float) -> c_float {
    rust_libm::expm1f(x)
}

#[no_mangle]
pub unsafe extern "C" fn fabs(x: c_double) -> c_double {
    rust_libm::fabs(x)
}

#[no_mangle]
pub unsafe extern "C" fn fabsf(x: c_float) -> c_float {
    rust_libm::fabsf(x)
}

#[no_mangle]
pub unsafe extern "C" fn fdim(x: c_double, y: c_double) -> c_double {
    rust_libm::fdim(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fdimf(x: c_float, y: c_float) -> c_float {
    rust_libm::fdimf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn floor(x: c_double) -> c_double {
    rust_libm::floor(x)
}

#[no_mangle]
pub unsafe extern "C" fn floorf(x: c_float) -> c_float {
    rust_libm::floorf(x)
}

#[no_mangle]
pub unsafe extern "C" fn fma(x: c_double, y: c_double, z: c_double) -> c_double {
    rust_libm::fma(x, y, z)
}

#[no_mangle]
pub unsafe extern "C" fn fmaf(x: c_float, y: c_float, z: c_float) -> c_float {
    rust_libm::fmaf(x, y, z)
}

#[no_mangle]
pub unsafe extern "C" fn fmax(x: c_double, y: c_double) -> c_double {
    rust_libm::fmax(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fmaxf(x: c_float, y: c_float) -> c_float {
    rust_libm::fmaxf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fmin(x: c_double, y: c_double) -> c_double {
    rust_libm::fmin(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fminf(x: c_float, y: c_float) -> c_float {
    rust_libm::fminf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fmod(x: c_double, y: c_double) -> c_double {
    rust_libm::fmod(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn fmodf(x: c_float, y: c_float) -> c_float {
    rust_libm::fmodf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn frexp(x: c_double, y: *mut c_int) -> c_double {
    let (a, b) = rust_libm::frexp(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn frexpf(x: c_float, y: *mut c_int) -> c_float {
    let (a, b) = rust_libm::frexpf(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn hypot(x: c_double, y: c_double) -> c_double {
    rust_libm::hypot(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn hypotf(x: c_float, y: c_float) -> c_float {
    rust_libm::hypotf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn ilogb(x: c_double) -> c_int {
    rust_libm::ilogb(x)
}

#[no_mangle]
pub unsafe extern "C" fn ilogbf(x: c_float) -> c_int {
    rust_libm::ilogbf(x)
}

#[no_mangle]
pub unsafe extern "C" fn j0(x: c_double) -> c_double {
    rust_libm::j0(x)
}

#[no_mangle]
pub unsafe extern "C" fn j0f(x: c_float) -> c_float {
    rust_libm::j0f(x)
}

#[no_mangle]
pub unsafe extern "C" fn j1(x: c_double) -> c_double {
    rust_libm::j1(x)
}

#[no_mangle]
pub unsafe extern "C" fn j1f(x: c_float) -> c_float {
    rust_libm::j1f(x)
}

#[no_mangle]
pub unsafe extern "C" fn jn(x: c_int, y: c_double) -> c_double {
    rust_libm::jn(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn jnf(x: c_int, y: c_float) -> c_float {
    rust_libm::jnf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn ldexp(x: c_double, y: c_int) -> c_double {
    rust_libm::ldexp(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn ldexpf(x: c_float, y: c_int) -> c_float {
    rust_libm::ldexpf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn lgamma(x: c_double) -> c_double {
    rust_libm::lgamma(x)
}

#[no_mangle]
pub unsafe extern "C" fn lgammaf(x: c_float) -> c_float {
    rust_libm::lgammaf(x)
}

#[no_mangle]
pub unsafe extern "C" fn lgamma_r(x: c_double, y: *mut c_int) -> c_double {
    let (a, b) = rust_libm::lgamma_r(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn lgammaf_r(x: c_float, y: *mut c_int) -> c_float {
    let (a, b) = rust_libm::lgammaf_r(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn llrint(x: c_double) -> c_longlong {
    rint(x) as c_longlong
}

#[no_mangle]
pub unsafe extern "C" fn llrintf(x: c_float) -> c_longlong {
    rintf(x) as c_longlong
}

#[no_mangle]
pub unsafe extern "C" fn llround(x: c_double) -> c_longlong {
    round(x) as c_longlong
}

#[no_mangle]
pub unsafe extern "C" fn llroundf(x: c_float) -> c_longlong {
    roundf(x) as c_longlong
}

#[no_mangle]
pub unsafe extern "C" fn log(x: c_double) -> c_double {
    rust_libm::log(x)
}

#[no_mangle]
pub unsafe extern "C" fn log1p(x: c_double) -> c_double {
    rust_libm::log1p(x)
}

#[no_mangle]
pub unsafe extern "C" fn log1pf(x: c_float) -> c_float {
    rust_libm::log1pf(x)
}

#[no_mangle]
pub unsafe extern "C" fn log2(x: c_double) -> c_double {
    rust_libm::log2(x)
}

#[no_mangle]
pub unsafe extern "C" fn log2f(x: c_float) -> c_float {
    rust_libm::log2f(x)
}

#[no_mangle]
pub unsafe extern "C" fn log10(x: c_double) -> c_double {
    rust_libm::log10(x)
}

#[no_mangle]
pub unsafe extern "C" fn log10f(x: c_float) -> c_float {
    rust_libm::log10f(x)
}

#[no_mangle]
pub unsafe extern "C" fn logb(x: c_double) -> c_double {
    if x.is_nan() {
        x - 0.0
    } else if x == 0.0 {
        -c_double::INFINITY
    } else if fabs(x) == c_double::INFINITY {
        c_double::INFINITY
    } else {
        ilogb(x) as c_double
    }
}

#[no_mangle]
pub unsafe extern "C" fn logbf(x: c_float) -> c_float {
    if x.is_nan() {
        x - 0.0
    } else if x == 0.0 {
        -c_float::INFINITY
    } else if fabsf(x) == c_float::INFINITY {
        c_float::INFINITY
    } else {
        ilogbf(x) as c_float
    }
}

#[no_mangle]
pub unsafe extern "C" fn logf(x: c_float) -> c_float {
    rust_libm::logf(x)
}

// TODO: We don't support floating-point exceptions,
// for now we ignore `FE_INEXACT`.

#[no_mangle]
pub unsafe extern "C" fn lrint(x: c_double) -> c_long {
    rint(x) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn lrintf(x: c_float) -> c_long {
    rintf(x) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn lround(x: c_double) -> c_long {
    round(x) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn lroundf(x: c_float) -> c_long {
    roundf(x) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn modf(x: c_double, y: *mut c_double) -> c_double {
    let (a, b) = rust_libm::modf(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn modff(x: c_float, y: *mut c_float) -> c_float {
    let (a, b) = rust_libm::modff(x);
    *y = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn nan(_x: *const c_char) -> c_double {
    c_double::NAN
}

#[no_mangle]
pub unsafe extern "C" fn nanf(_x: *const c_char) -> c_float {
    c_float::NAN
}

#[no_mangle]
pub unsafe extern "C" fn nextafter(x: c_double, y: c_double) -> c_double {
    rust_libm::nextafter(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn nextafterf(x: c_float, y: c_float) -> c_float {
    rust_libm::nextafterf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn pow(x: c_double, y: c_double) -> c_double {
    rust_libm::pow(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn powf(x: c_float, y: c_float) -> c_float {
    rust_libm::powf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn remainder(x: c_double, y: c_double) -> c_double {
    rust_libm::remainder(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn remainderf(x: c_float, y: c_float) -> c_float {
    rust_libm::remainderf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn remquo(x: c_double, y: c_double, z: *mut c_int) -> c_double {
    let (a, b) = rust_libm::remquo(x, y);
    *z = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn remquof(x: c_float, y: c_float, z: *mut c_int) -> c_float {
    let (a, b) = rust_libm::remquof(x, y);
    *z = b;
    a
}

#[no_mangle]
pub unsafe extern "C" fn round(x: c_double) -> c_double {
    rust_libm::round(x)
}

#[no_mangle]
pub unsafe extern "C" fn roundf(x: c_float) -> c_float {
    rust_libm::roundf(x)
}

#[no_mangle]
pub unsafe extern "C" fn scalbln(x: c_double, y: c_long) -> c_double {
    let y = y.clamp(c_int::MIN.into(), c_int::MAX.into()) as _;
    scalbn(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn scalblnf(x: c_float, y: c_long) -> c_float {
    let y = y.clamp(c_int::MIN.into(), c_int::MAX.into()) as _;
    scalbnf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn scalbn(x: c_double, y: c_int) -> c_double {
    rust_libm::scalbn(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn scalb(x: f64, exp: f64) -> f64 {
    if x.is_nan() {
        x - 0.0
    } else if exp.is_nan() {
        exp - 0.0
    } else if !exp.is_finite() {
        if exp > 0.0 {
            x * exp
        } else {
            x / -exp
        }
    } else if rint(exp) != exp {
        f64::NAN
    } else if exp > 65000.0 {
        scalbn(x, 65000)
    } else if -exp > 65000.0 {
        scalbn(x, -65000)
    } else {
        scalbn(x, exp as i32)
    }
}

#[no_mangle]
pub unsafe extern "C" fn scalbf(x: c_float, exp: c_float) -> c_float {
    if x.is_nan() {
        x - 0.0
    } else if exp.is_nan() {
        exp - 0.0
    } else if !exp.is_finite() {
        if exp > 0.0 {
            x * exp
        } else {
            x / -exp
        }
    } else if rintf(exp) != exp {
        c_float::NAN
    } else if exp > 65000.0 {
        scalbnf(x, 65000)
    } else if -exp > 65000.0 {
        scalbnf(x, -65000)
    } else {
        scalbnf(x, exp as i32)
    }
}

#[no_mangle]
pub unsafe extern "C" fn scalbnf(x: c_float, y: c_int) -> c_float {
    rust_libm::scalbnf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn sin(x: c_double) -> c_double {
    rust_libm::sin(x)
}

#[no_mangle]
pub unsafe extern "C" fn sincos(x: c_double, y: *mut c_double, z: *mut c_double) {
    let (a, b) = rust_libm::sincos(x);
    *y = a;
    *z = b;
}

#[no_mangle]
pub unsafe extern "C" fn sincosf(x: c_float, y: *mut c_float, z: *mut c_float) {
    let (a, b) = rust_libm::sincosf(x);
    *y = a;
    *z = b;
}

#[no_mangle]
pub unsafe extern "C" fn sinf(x: c_float) -> c_float {
    rust_libm::sinf(x)
}

#[no_mangle]
pub unsafe extern "C" fn sinh(x: c_double) -> c_double {
    rust_libm::sinh(x)
}

#[no_mangle]
pub unsafe extern "C" fn sinhf(x: c_float) -> c_float {
    rust_libm::sinhf(x)
}

#[no_mangle]
pub unsafe extern "C" fn sqrt(x: c_double) -> c_double {
    rust_libm::sqrt(x)
}

#[no_mangle]
pub unsafe extern "C" fn sqrtf(x: c_float) -> c_float {
    rust_libm::sqrtf(x)
}

#[no_mangle]
pub unsafe extern "C" fn tan(x: c_double) -> c_double {
    rust_libm::tan(x)
}

#[no_mangle]
pub unsafe extern "C" fn tanf(x: c_float) -> c_float {
    rust_libm::tanf(x)
}

#[no_mangle]
pub unsafe extern "C" fn tanh(x: c_double) -> c_double {
    rust_libm::tanh(x)
}

#[no_mangle]
pub unsafe extern "C" fn tanhf(x: c_float) -> c_float {
    rust_libm::tanhf(x)
}

#[no_mangle]
pub unsafe extern "C" fn tgamma(x: c_double) -> c_double {
    rust_libm::tgamma(x)
}

#[no_mangle]
pub unsafe extern "C" fn tgammaf(x: c_float) -> c_float {
    rust_libm::tgammaf(x)
}

#[no_mangle]
pub unsafe extern "C" fn trunc(x: c_double) -> c_double {
    rust_libm::trunc(x)
}

#[no_mangle]
pub unsafe extern "C" fn truncf(x: c_float) -> c_float {
    rust_libm::truncf(x)
}

#[no_mangle]
pub unsafe extern "C" fn y0(x: c_double) -> c_double {
    rust_libm::y0(x)
}

#[no_mangle]
pub unsafe extern "C" fn y0f(x: c_float) -> c_float {
    rust_libm::y0f(x)
}

#[no_mangle]
pub unsafe extern "C" fn y1(x: c_double) -> c_double {
    rust_libm::y1(x)
}

#[no_mangle]
pub unsafe extern "C" fn y1f(x: c_float) -> c_float {
    rust_libm::y1f(x)
}

#[no_mangle]
pub unsafe extern "C" fn yn(x: c_int, y: c_double) -> c_double {
    rust_libm::yn(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn ynf(x: c_int, y: c_float) -> c_float {
    rust_libm::ynf(x, y)
}

#[no_mangle]
pub unsafe extern "C" fn rint(x: c_double) -> c_double {
    rust_libm::rint(x)
}

#[no_mangle]
pub unsafe extern "C" fn rintf(x: c_float) -> c_float {
    rust_libm::rintf(x)
}

// `nearbyint` differs from `rint` in that it doesn't raise
// `FE_INEXACT`. But we don't support floating-point exceptions
// anyway, so don't worry about it.
#[no_mangle]
pub unsafe extern "C" fn nearbyint(x: c_double) -> c_double {
    rust_libm::rint(x)
}

#[no_mangle]
pub unsafe extern "C" fn nearbyintf(x: c_float) -> c_float {
    rust_libm::rintf(x)
}
