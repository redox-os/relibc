#[no_mangle]
pub extern "C" fn clearerr(arg1: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctermid(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fclose(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fdopen(arg1: libc::c_int,
                  arg2: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn feof(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ferror(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fflush(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetc(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetpos(arg1: *mut FILE, arg2: *mut fpos_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgets(arg1: *mut libc::c_char,
                 arg2: libc::c_int, arg3: *mut FILE)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fileno(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn flockfile(arg1: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fopen(arg1: *const libc::c_char,
                 arg2: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fprintf(arg1: *mut FILE, arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputc(arg1: libc::c_int, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputs(arg1: *const libc::c_char, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fread(arg1: *mut libc::c_void, arg2: usize, arg3: usize,
                 arg4: *mut FILE) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn freopen(arg1: *const libc::c_char,
                   arg2: *const libc::c_char, arg3: *mut FILE)
     -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fscanf(arg1: *mut FILE, arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseek(arg1: *mut FILE, arg2: libc::c_long,
                 arg3: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseeko(arg1: *mut FILE, arg2: off_t, arg3: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fsetpos(arg1: *mut FILE, arg2: *const fpos_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftell(arg1: *mut FILE) -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftello(arg1: *mut FILE) -> off_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftrylockfile(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn funlockfile(arg1: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwrite(arg1: *const libc::c_void, arg2: usize,
                  arg3: usize, arg4: *mut FILE) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc_unlocked(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar_unlocked() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getopt(arg1: libc::c_int,
                  arg2: *const *const libc::c_char,
                  arg3: libc::c_char) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gets(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getw(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pclose(arg1: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn perror(arg1: *const libc::c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn popen(arg1: *const libc::c_char,
                 arg2: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn printf(arg1: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putc(arg1: libc::c_int, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putchar(arg1: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putc_unlocked(arg1: libc::c_int, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putchar_unlocked(arg1: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn puts(arg1: *const libc::c_char) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putw(arg1: libc::c_int, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn remove(arg1: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rename(arg1: *const libc::c_char,
                  arg2: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rewind(arg1: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn scanf(arg1: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setbuf(arg1: *mut FILE, arg2: *mut libc::c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setvbuf(arg1: *mut FILE, arg2: *mut libc::c_char,
                   arg3: libc::c_int, arg4: usize)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn snprintf(arg1: *mut libc::c_char, arg2: usize,
                    arg3: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sprintf(arg1: *mut libc::c_char,
                   arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sscanf(arg1: *const libc::c_char,
                  arg2: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tempnam(arg1: *const libc::c_char,
                   arg2: *const libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpfile() -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpnam(arg1: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ungetc(arg1: libc::c_int, arg2: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vfprintf(arg1: *mut FILE, arg2: *const libc::c_char,
                    va_list: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vprintf(arg1: *const libc::c_char,
                   va_list: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vsnprintf(arg1: *mut libc::c_char, arg2: usize,
                     arg3: *const libc::c_char,
                     va_list: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vsprintf(arg1: *mut libc::c_char,
                    arg2: *const libc::c_char,
                    va_list: libc::c_int) -> libc::c_int {
    unimplemented!();
}

