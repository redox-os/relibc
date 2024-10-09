use crate::platform::types::{c_double, c_float};
use inner_libm;
use num_complex::{Complex32, Complex64, ComplexFloat};

#[no_mangle]
pub unsafe extern "C" fn creal(x: Complex64) -> c_double {
    x.re
}

#[no_mangle]
pub unsafe extern "C" fn crealf(x: Complex32) -> c_float {
    x.re
}

#[no_mangle]
pub unsafe extern "C" fn cimag(x: Complex64) -> c_double {
    x.im
}

#[no_mangle]
pub unsafe extern "C" fn cimagf(x: Complex32) -> c_float {
    x.im
}

#[no_mangle]
pub unsafe extern "C" fn cabs(x: Complex64) -> c_double {
    x.abs()
}

#[no_mangle]
pub unsafe extern "C" fn cabsf(x: Complex32) -> c_float {
    x.abs()
}

#[no_mangle]
pub unsafe extern "C" fn carg(x: Complex64) -> c_double {
    x.arg()
}

#[no_mangle]
pub unsafe extern "C" fn cargf(x: Complex32) -> c_float {
    x.arg()
}

#[no_mangle]
pub unsafe extern "C" fn cacos(x: Complex64) -> Complex64 {
    x.acos()
}

#[no_mangle]
pub unsafe extern "C" fn cacosf(x: Complex32) -> Complex32 {
    x.acos()
}

#[no_mangle]
pub unsafe extern "C" fn casin(x: Complex64) -> Complex64 {
    x.asin()
}

#[no_mangle]
pub unsafe extern "C" fn casinf(x: Complex32) -> Complex32 {
    x.asin()
}

#[no_mangle]
pub unsafe extern "C" fn catan(x: Complex64) -> Complex64 {
    x.atan()
}

#[no_mangle]
pub unsafe extern "C" fn catanf(x: Complex32) -> Complex32 {
    x.atan()
}

#[no_mangle]
pub unsafe extern "C" fn ccos(x: Complex64) -> Complex64 {
    x.cos()
}

#[no_mangle]
pub unsafe extern "C" fn ccosf(x: Complex32) -> Complex32 {
    x.cos()
}

#[no_mangle]
pub unsafe extern "C" fn csin(x: Complex64) -> Complex64 {
    x.sin()
}

#[no_mangle]
pub unsafe extern "C" fn csinf(x: Complex32) -> Complex32 {
    x.sin()
}

#[no_mangle]
pub unsafe extern "C" fn ctan(x: Complex64) -> Complex64 {
    x.tan()
}

#[no_mangle]
pub unsafe extern "C" fn ctanf(x: Complex32) -> Complex32 {
    x.tan()
}

#[no_mangle]
pub unsafe extern "C" fn cacosh(x: Complex64) -> Complex64 {
    x.acosh()
}

#[no_mangle]
pub unsafe extern "C" fn cacoshf(x: Complex32) -> Complex32 {
    x.acosh()
}

#[no_mangle]
pub unsafe extern "C" fn casinh(x: Complex64) -> Complex64 {
    x.asinh()
}

#[no_mangle]
pub unsafe extern "C" fn casinhf(x: Complex32) -> Complex32 {
    x.asinh()
}

#[no_mangle]
pub unsafe extern "C" fn catanh(x: Complex64) -> Complex64 {
    x.atanh()
}

#[no_mangle]
pub unsafe extern "C" fn catanhf(x: Complex32) -> Complex32 {
    x.atanh()
}

#[no_mangle]
pub unsafe extern "C" fn ccosh(x: Complex64) -> Complex64 {
    x.cosh()
}

#[no_mangle]
pub unsafe extern "C" fn ccoshf(x: Complex32) -> Complex32 {
    x.cosh()
}

#[no_mangle]
pub unsafe extern "C" fn csinh(x: Complex64) -> Complex64 {
    x.sinh()
}

#[no_mangle]
pub unsafe extern "C" fn csinhf(x: Complex32) -> Complex32 {
    x.sinh()
}

#[no_mangle]
pub unsafe extern "C" fn ctanh(x: Complex64) -> Complex64 {
    x.tanh()
}

#[no_mangle]
pub unsafe extern "C" fn ctanhf(x: Complex32) -> Complex32 {
    x.tanh()
}

#[no_mangle]
pub unsafe extern "C" fn cexp(x: Complex64) -> Complex64 {
    x.exp()
}

#[no_mangle]
pub unsafe extern "C" fn cexpf(x: Complex32) -> Complex32 {
    x.exp()
}

#[no_mangle]
pub unsafe extern "C" fn clog(x: Complex64) -> Complex64 {
    x.ln()
}

#[no_mangle]
pub unsafe extern "C" fn clogf(x: Complex32) -> Complex32 {
    x.ln()
}

#[no_mangle]
pub unsafe extern "C" fn clog10(x: Complex64) -> Complex64 {
    x.log10()
}

#[no_mangle]
pub unsafe extern "C" fn clog10f(x: Complex32) -> Complex32 {
    x.log10()
}

#[no_mangle]
pub unsafe extern "C" fn csqrt(x: Complex64) -> Complex64 {
    x.sqrt()
}

#[no_mangle]
pub unsafe extern "C" fn csqrtf(x: Complex32) -> Complex32 {
    x.sqrt()
}

#[no_mangle]
pub unsafe extern "C" fn conj(x: Complex64) -> Complex64 {
    x.conj()
}

#[no_mangle]
pub unsafe extern "C" fn conjf(x: Complex32) -> Complex32 {
    x.conj()
}

#[no_mangle]
pub unsafe extern "C" fn cproj(x: Complex64) -> Complex64 {
    if x.re.abs() == f64::INFINITY && x.im.abs() == f64::INFINITY {
        Complex64 {
            re: f64::INFINITY,
            im: inner_libm::copysign(0.0, x.im),
        }
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn cprojf(x: Complex32) -> Complex32 {
    if x.re.abs() == f32::INFINITY && x.im.abs() == f32::INFINITY {
        Complex32 {
            re: f32::INFINITY,
            im: inner_libm::copysignf(0.0, x.im),
        }
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn cpow(x: Complex64, y: Complex64) -> Complex64 {
    x.powc(y)
}

#[no_mangle]
pub unsafe extern "C" fn cpowf(x: Complex32, y: Complex32) -> Complex32 {
    x.powc(y)
}
