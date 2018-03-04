#[no_mangle]
pub extern "C" fn clearerr(stream: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctermid(s: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(s: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fclose(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fdopen(fildes: libc::c_int,
                  mode: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn feof(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ferror(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fflush(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetc(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetpos(stream: *mut FILE, pos: *mut fpos_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgets(s: *mut libc::c_char,
                 n: libc::c_int, stream: *mut FILE)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fileno(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn flockfile(file: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fopen(filename: *const libc::c_char,
                 mode: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fprintf(stream: *mut FILE, format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputc(c: libc::c_int, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputs(s: *const libc::c_char, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fread(ptr: *mut libc::c_void, size: usize, nitems: usize,
                 stream: *mut FILE) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn freopen(filename: *const libc::c_char,
                   mode: *const libc::c_char, stream: *mut FILE)
     -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fscanf(stream: *mut FILE, format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseek(stream: *mut FILE, offset: libc::c_long,
                 whence: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseeko(stream: *mut FILE, offset: off_t, whence: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fsetpos(stream: *mut FILE, pos: *const fpos_t)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftell(stream: *mut FILE) -> libc::c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftello(stream: *mut FILE) -> off_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftrylockfile(file: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn funlockfile(file: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwrite(ptr: *const libc::c_void, size: usize,
                  nitems: usize, stream: *mut FILE) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc_unlocked(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar_unlocked() -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getopt(argc: libc::c_int,
                  argv: *const *const libc::c_char,
                  optstring: libc::c_char) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gets(s: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getw(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pclose(stream: *mut FILE) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn perror(s: *const libc::c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn popen(command: *const libc::c_char,
                 mode: *const libc::c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn printf(format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putc(c: libc::c_int, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putchar(c: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putc_unlocked(c: libc::c_int, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putchar_unlocked(c: libc::c_int)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn puts(s: *const libc::c_char) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putw(w: libc::c_int, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn remove(path: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rename(old: *const libc::c_char,
                  new: *const libc::c_char)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rewind(stream: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn scanf(format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setbuf(stream: *mut FILE, buf: *mut libc::c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setvbuf(stream: *mut FILE, buf: *mut libc::c_char,
                   _IOBUF: libc::c_int, BUFSIZ: usize)
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
pub extern "C" fn sprintf(s: *mut libc::c_char,
                   format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn sscanf(s: *const libc::c_char,
                  format: *const libc::c_char, ...)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tempnam(dir: *const libc::c_char,
                   pfx: *const libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpfile() -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpnam(s: *mut libc::c_char)
     -> *mut libc::c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ungetc(c: libc::c_int, stream: *mut FILE)
     -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vfprintf(stream: *mut FILE, format: *const libc::c_char,
                    ap: libc::c_int) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vprintf(format: *const libc::c_char,
                   ap: va_list) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vsnprintf(s: *mut libc::c_char, n: usize,
                     format: *const libc::c_char,
                     ap: va_list) -> libc::c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn vsprintf(s: *mut libc::c_char,
                    format: *const libc::c_char,
                    ap: va_list) -> libc::c_int {
    unimplemented!();
}

