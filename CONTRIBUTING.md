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

## Code style

We have a `rustfmt.toml` in the root directory of relibc. Please run `./fmt.sh`
before sending in any merge requests as it will automatically format your code.

With regards to general style:

### Where applicable, prefer using references to raw pointers

This is most obvious when looking at `stdio` functions. If raw pointers were
used instead of references, then the resulting code would be significantly
uglier. Instead try to check for pointer being valid with `pointer::as_ref()`
and `pointer::as_mut()` and then immediately use those references instead.

Internal functions should always take references.

### Use the c types exposed in our platform module instead of Rust's inbuilt integer types

This is so we can guarantee that everything works across platforms. While it is
generally accepted these days that an `int` has 32 bits (which matches against
an `i32`), some platforms have `int` as having 16 bits, and others have long as
being 32 bits instead of 64. If you use the types in platform, then we can
guarantee that your code will "just work" should we port relibc to a different
architecture.

### Use our internal functions

If you need to use a C string, don't reinvent the wheel. We have functions in
the platform module that convert C strings to Rust slices.

We also have structures that wrap files, wrap writable strings, and wrap various
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
