# Contributing

## Table of contents
1. [What to do](#what-to-do)
2. [Code style](#code-style)
3. [Sending merge requests](#sending-merge-requests)
4. [Writing tests](#writing-tests)
5. [Running tests](#running-tests)

Maintaining a libc is tough work, and we'd love some help!

## What to do

For now, we are still trying to get full libc compatibility before we move on to
any optimisation.

- We currently have a number of unimplemented functions. Search for 
    `unimplemented!()` and hop right in!
- If you notice any missing functionality, feel free to add it in

## Code style and philosophy

We have a `rustfmt.toml` in the root directory of relibc. Please run `./fmt.sh`
before sending in any merge requests as it will automatically format your code.

Rust gives a powerful advantage over C in that the type system can encode a much richer set of constraints, enabling the compiler to (almost fully) be able to give guarantees that safe code in a properly constructed codebase, can't cause any Undefined Behavior. While a libc implementation obviously requires unsafe at most places it provides C-compatible interfaces, relibc has recently become much better at using abstractions that can reduce the risk of UB and logic errors, by being more Rust-like. It's thus recommended to try moving as much unsafe and C-isms as possible to the "leaf functions". Some of the building blocks for this are still being developed though, or are not yet as ergonomic, so contributions are of course welcome!

### Use Rust-like error handling

We provide the `Errno` error type, which is a very thin wrapper over the possible C-style error numbers that can be returned. This means the internal implementations usually never need to access `errno` or return

Prefer this:

```rust
// (where the impl is located)
fn some_implementation_function(arg: Arg) -> Result<(), Errno> {
    if arg == 0 {
        Ok(())
    } else {
        Err(Errno(EOPNOTSUPP))
    }
}

// (where the header module is located)
#[no_mangle]
pub extern "C" unsafe fn some_interface_function(arg: Arg) -> c_int {
    some_implementation_function(arg).or_minus_one_errno()
}
```

over this:

```rust
fn some_implementation_function(arg: Arg) -> c_int {
    if arg == 0 {
        0
    } else {
        platform::ERRNO.set(-EOPNOTSUPP);
        -1
    }
}

#[no_mangle]
pub extern "C" unsafe fn some_interface_function(arg: Arg) -> c_int {
    some_implementation_function(arg)
}
```

Even in small functions, it can sometimes be good to create a closure and then immediately call it, i.e.
```rust
#[no_mangle]
pub extern "C" unsafe fn some_interface_function(arg: Arg) -> c_int {
    (|| {
        Err(Errno(EOPNOTSUPP))
    })().or_minus_one_errno()
}
```

### Use safe wrappers over raw C types and patterns, where possible

Some interfaces like `getenv` and `environ` will be inherently unsafe and are mostly impractical to create safe wrappers for. However, in many situations, even C-style invariants like NUL-terminated strings and arrays can be safely encoded in Rust. In particular, we can use the `CStr` and `WStr` wrappers for C strings, `NulTerminated<T>` for nul-terminated arrays, `Out<T>` for the possibly uninitialized "out-pointer" pattern, and of course regular Rust slices. For example (the actual standard and code is a bit different), prefer this

```rust
fn getsockname_impl(socket: c_int, address_dst: Out<[u8]>, some_extra_field: Option<CStr<'_>>) -> Result<socklen_t> {
    // ...
    let true_value: Vec<u8> = get_true_value()?;
    address_dst.copy_common_length_from_slice(&true_value);
    Ok(true_value)
}

// Interface is used to get the name of a socket. It takes a buffer whose length can be read from the pointer, which is then used to return the true length to allow buffer enlargement etc if needed. The return value is just used for error handling here.
#[no_mangle]
pub extern "C" unsafe fn getsockname_c(socket: c_int, address_buf: *mut c_void, address_len_inout: *mut socklen_t, some_extra_field: *const c_char) -> c_int {
    // unsafe is restricted to C-adjacent code...
    let dst = unsafe { Out::from_raw_parts_mut(address_buf.cast(), address_len_inout.read() as _) };
    let some_extra_field = unsafe { CStr::from_nullable_ptr(some_extra_field) };

    let res = getsockname_impl(socket, dst);
    if let Ok(true_len) = res {
        unsafe {
            address_len_inout.write(true_len);
        }
    }
    // ... as is errno
    res.or_minus_one_errno()
}
```

over simply forwarding these C-isms to the implementation.

### Make public structs/typedefs opaque when the standard does not specify its fields

For type definitions like `FILE *` and `pthread_t` etc., there's no requirement by POSIX that C code can access its fields, and hence it will be advantageous to declare an internal Rust struct that can use C-incompatible safe types like `String`, `Vec<T>`, `File` etc, and an outside opaque struct whose length and alignment matches.

This should be preferred over using raw pointer equivalents, even though it can be a bit more verbose. The C implementation functions will then always dereference e.g. `FILE *` into e.g. `&File`. See the `FILE *`, `pthread_t` <-> `Pthread` definitions for how this can be done. It's usually good to also reserve some space as ABI breakage will require header upstream crates to be patched, but it's always possible (at some perf cost) to make it larger by boxing.

Of course, if the standard *does* require individual fields of a particular struct to be accessible, or if the struct needs to be accessed with macros, this may not be possible.

### Use the c types exposed in our platform module instead of Rust's inbuilt integer types

This is so we can guarantee that everything works across platforms. While it is
generally accepted these days that an `int` has 32 bits (which matches against
an `i32`), some platforms have `int` as having 16 bits, and others have long as
being 32 bits instead of 64. If you use the types in platform, then we can
guarantee that your code will "just work" should we port relibc to a different
architecture.

### Use our other functions

We have structures that wrap files, wrap writable strings, and wrap various
other commonly used things that you should use instead of rolling your own.

## Sending merge requests

If you have sent us a merge request, first of all, thanks for taking your time
to help us!

The first thing to note is that we do most of our development on our 
[GitLab server](https://gitlab.redox-os.org/redox-os/relibc), and as such it is
possible that none of the maintainers will see your merge request if it is
opened on GitHub.

In your merge request, please put in the description:
- What functions (if any) have been implemented or changed
- The rationale behind your merge request (e.g. why you thought this change was
    required. If you are just implementing some functions, you can ignore this)
- Any issues that are related to the merge request

We have CI attached to our GitLab instance, so all merge requests are checked to
make sure that they are tested before they are merged. Please write tests for
the functions that you add/change and test locally on your own machine
***before*** submitting a merge request.

## Writing tests

Every function that gets written needs to have a test in C in order to make sure
it works as intended. Here are a few guidelines for writing good tests.

### Ensure that any literals you have are mapped to variables instead of being directly passed to a function.

Sometimes compilers take literals put into libc functions and run them 
internally during compilation, which can cause some false positives.  All tests
are compiled with `-fno-builtin`, which theoretically solves this issue, but
just in case, it'd be a good idea to map inputs to variables.

```c
#include "string.h"
#include "stdio.h"

int main(void) {
    // Don't do this
    printf("%d\n", strcspn("Hello", "Hi"));

    // Do this
    char *first = "Hello";
    char *second = "Hi";
    printf("%d\n", strcspn(first, second));
}
```

### Ensure your tests cover every section of code.

What happens if a string in `strcmp()` is shorter than the other string? What 
happens if the first argument to `strcspn()` is longer than the second string?  
In order to make sure that all functions work as expected, we ask that any tests
cover as much of the code that you have written as possible.

## Running tests

Running tests is an important part in trying to find bugs. Before opening a
merge request, we ask that you test on your own machine to make sure there are
no regressions.

You can run tests with `make test` in the root directory of relibc to compile
relibc, compile the tests and run them. This *will* print a lot of output to
stdout, so be warned!

You can test against verified correct output with `make verify` in the tests 
directory. You will need to manually create the correct output and put it in the
tests/expected directory. Running any `make` commands in the tests directory
will ***not*** rebuild relibc, so you'll need to go back to the root directory
if you need to rebuild relibc.
