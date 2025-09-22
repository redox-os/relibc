use core::{
    convert::Infallible,
    num::{NonZeroU64, NonZeroUsize},
};

use crate::{
    c_str::{CStr, CString},
    fs::File,
    header::{limits::PATH_MAX, string::strlen},
    io::{prelude::*, BufReader, SeekFrom},
    platform::{
        sys::{S_ISGID, S_ISUID},
        types::*,
    },
};

use redox_rt::{
    proc::{ExtraInfo, FdGuard, FexecResult, InterpOverride},
    sys::Resugid,
    RtTcb,
};
use syscall::{data::Stat, error::*, flag::*};

fn fexec_impl(
    exec_file: FdGuard,
    path: &[u8],
    args: &[&[u8]],
    envs: &[&[u8]],
    total_args_envs_size: usize,
    extrainfo: &ExtraInfo,
    interp_override: Option<InterpOverride>,
) -> Result<Infallible> {
    let memory = FdGuard::new(syscall::open("/scheme/memory", 0)?);

    let addrspace_selection_fd = match redox_rt::proc::fexec_impl(
        exec_file,
        &RtTcb::current().thread_fd(),
        redox_rt::current_proc_fd(),
        &memory,
        path,
        args.iter().rev(),
        envs.iter().rev(),
        total_args_envs_size,
        extrainfo,
        interp_override,
    )? {
        FexecResult::Normal { addrspace_handle } => addrspace_handle,
        FexecResult::Interp {
            image_file,
            path,
            interp_override: new_interp_override,
        } => {
            drop(image_file);
            drop(memory);

            // According to elf(5), PT_INTERP requires that the interpreter path be
            // null-terminated. Violating this should therefore give the "format error" ENOEXEC.
            let path_cstr = CStr::from_bytes_with_nul(&path).map_err(|_| Error::new(ENOEXEC))?;

            return execve(
                Executable::AtPath(path_cstr),
                ArgEnv::Parsed {
                    total_args_envs_size,
                    args,
                    envs,
                },
                Some(new_interp_override),
            );
        }
    };
    drop(memory);

    // Dropping this FD will cause the address space switch.
    drop(addrspace_selection_fd);

    unreachable!();
}
pub enum ArgEnv<'a> {
    C {
        argv: *const *mut c_char,
        envp: *const *mut c_char,
    },
    Parsed {
        args: &'a [&'a [u8]],
        envs: &'a [&'a [u8]],
        total_args_envs_size: usize,
    },
}

pub enum Executable<'a> {
    AtPath(CStr<'a>),
    InFd { file: File, arg0: &'a [u8] },
}

pub fn execve(
    exec: Executable<'_>,
    arg_env: ArgEnv,
    interp_override: Option<InterpOverride>,
) -> Result<Infallible> {
    // NOTE: We must omit O_CLOEXEC and close manually, otherwise it will be closed before we
    // have even read it!
    let (mut image_file, arg0) = match exec {
        Executable::AtPath(path) => (
            File::open(path, O_RDONLY as c_int).map_err(|_| Error::new(ENOENT))?,
            path.to_bytes(),
        ),
        Executable::InFd { file, arg0 } => (file, arg0),
    };

    // With execve now being implemented in userspace, we need to check ourselves that this
    // file is actually executable. While checking for read permission is unnecessary as the
    // scheme will not allow us to read otherwise, the execute bit is completely unenforced.
    //
    // But we do (currently) have the permission to mmap executable memory and fill it with any
    // program, even marked non-executable, so really the best we can do is check that nothing is
    // executed by accident.
    //
    // TODO: At some point we might have capabilities limiting the ability to allocate
    // executable memory.

    let mut stat = Stat::default();
    syscall::fstat(*image_file as usize, &mut stat)?;
    let Resugid { ruid, rgid, .. } = redox_rt::sys::posix_getresugid();

    let mode = if ruid == stat.st_uid {
        (stat.st_mode >> 3 * 2) & 0o7
    } else if rgid == stat.st_gid {
        (stat.st_mode >> 3 * 1) & 0o7
    } else {
        stat.st_mode & 0o7
    };

    if mode & 0o1 == 0o0 {
        return Err(Error::new(EPERM));
    }

    let cwd: Box<[u8]> = super::path::clone_cwd().unwrap_or_default().into();
    let default_scheme: Box<[u8]> = super::path::clone_default_scheme()
        .unwrap_or_else(|| Box::from("file"))
        .into();

    // Path to interpreter binary and args if found
    let (interpreter_path, interpreter_args) = { parse_interpreter(&mut image_file)? };

    // Total number of arguments which includes the interpreter if interpreted and its args
    let mut len = 0;
    if interpreter_path.is_some() {
        len = 1;
        if interpreter_args.is_some() {
            len = 2;
        }
    }

    // Count arguments for `exec` which is different from the interpreter's args
    match arg_env {
        ArgEnv::C { argv, .. } => unsafe {
            while !(*argv.add(len)).is_null() {
                len += 1;
            }
        },
        ArgEnv::Parsed { args, .. } => len = args.len(),
    }
    let mut args: Vec<&[u8]> = Vec::with_capacity(len);

    if let Some(interpreter) = &interpreter_path {
        image_file = File::open(CStr::borrow(&interpreter), O_RDONLY as c_int)
            .map_err(|_| Error::new(ENOENT))?;

        // Push interpreter to arguments
        args.push(interpreter.as_bytes());

        // Push interpreter args, if any, to our main arguments
        if let Some(args_ref) = interpreter_args.as_ref() {
            args.push(args_ref.as_bytes());
        }
    } else {
        image_file
            .seek(SeekFrom::Start(0))
            .map_err(|_| Error::new(EIO))?;
    }

    let (total_args_envs_size, args, envs): (usize, Vec<_>, Vec<_>) = match arg_env {
        ArgEnv::C { mut argv, mut envp } => unsafe {
            let mut args_envs_size_without_nul = 0;

            // Arguments
            while !argv.read().is_null() {
                let arg = argv.read();

                let len = strlen(arg);
                args.push(core::slice::from_raw_parts(arg as *const u8, len));
                args_envs_size_without_nul += len;
                argv = argv.add(1);
            }

            // Environment variables
            let mut len = 0;
            while !envp.add(len).read().is_null() {
                len += 1;
            }

            let mut envs: Vec<&[u8]> = Vec::with_capacity(len);
            while !envp.read().is_null() {
                let env = envp.read();

                let len = strlen(env);
                envs.push(core::slice::from_raw_parts(env as *const u8, len));
                args_envs_size_without_nul += len;
                envp = envp.add(1);
            }
            (
                args_envs_size_without_nul + args.len() + envs.len(),
                args,
                envs,
            )
        },
        ArgEnv::Parsed {
            args: new_args,
            envs,
            total_args_envs_size,
        } => {
            let prev_size: usize = args.iter().map(|a| a.len()).sum();
            args.extend(new_args);
            (total_args_envs_size + prev_size, args, Vec::from(envs))
        }
    };

    // Close all O_CLOEXEC file descriptors. TODO: close_range?
    {
        // NOTE: This approach of implementing O_CLOEXEC will not work in multithreaded
        // scenarios. While execve() is undefined according to POSIX if there exist sibling
        // threads, it could still be allowed by keeping certain file descriptors and instead
        // set the active file table.
        let files_fd =
            File::new(syscall::dup(**RtTcb::current().thread_fd(), b"filetable")? as c_int);
        for line in BufReader::new(files_fd).lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            let fd = match line.parse::<usize>() {
                Ok(f) => f,
                Err(_) => continue,
            };

            let flags = syscall::fcntl(fd, F_GETFD, 0)?;

            if flags & O_CLOEXEC == O_CLOEXEC {
                let _ = syscall::close(fd);
            }
        }
    }

    // TODO: Convert image_file to FdGuard earlier?
    let exec_fd_guard = FdGuard::new(image_file.fd as usize);
    core::mem::forget(image_file);

    let sigprocmask = redox_rt::signal::get_sigmask().unwrap();

    let extrainfo = ExtraInfo {
        cwd: Some(&cwd),
        default_scheme: Some(&default_scheme),
        sigignmask: redox_rt::signal::get_sigignmask_to_inherit(),
        sigprocmask,
        umask: redox_rt::sys::get_umask(),
        thr_fd: **RtTcb::current().thread_fd(),
        proc_fd: **redox_rt::current_proc_fd(),
    };
    fexec_impl(
        exec_fd_guard,
        arg0,
        &args,
        &envs,
        total_args_envs_size,
        &extrainfo,
        interp_override,
    )
}

// Parse the interpreter and its args if `reader` starts with a shebang (#!).
//
// # Return
// * Path to the interpreter and its args, if any
// * `None` if no shebang
// * An error if parsing failed
//
// # Errors
// * E2BIG: The full path of the shebang is greater than [`PATH_MAX`]
// * ENOEXEC: Invalid shebang line, such as a line of all whitespace
// * EIO: Failure reading from `reader`
fn parse_interpreter<R>(image_file: &mut R) -> Result<(Option<CString>, Option<CString>)>
where
    R: Read + Seek,
{
    // Read shebang (for example #!/bin/sh)
    let mut read = 0;
    let mut shebang = [0; 2];

    while read < 2 {
        match image_file
            .read(&mut shebang)
            .map_err(|_| Error::new(ENOEXEC))?
        {
            0 => break,
            i => read += i,
        }
    }
    if shebang != *b"#!" {
        return Ok((None, None));
    }

    // BufReader is created after parsing the shebang because it doesn't make sense to buffer
    // bytes to read two bytes especially if `image_file` is NOT a script.
    let mut reader_ = BufReader::new(image_file);
    let reader = &mut reader_;

    // Skip prepended whitespace for interpreter
    // Ex: #! /usr/bin/python
    let pos = reader
        .bytes()
        .position(|byte| byte.ok().is_some_and(|byte| !byte.is_ascii_whitespace()))
        .and_then(|pos| (pos + 2).try_into().ok())
        // Fail if all whitespace or empty
        .ok_or_else(|| Error::new(ENOEXEC))?;
    // We read the non-whitespace character which sets reader position one past it.
    // Seeking back to that position is essentially free since reads are buffered and it's
    // unlikely that there was enough whitespace that we performed multiple reads.
    reader
        .seek(SeekFrom::Start(pos))
        .map_err(|_| Error::new(EIO))?;

    // Scan the first line once for the mandatory interpreter and optional args.
    // This is nicer than using `read_until` or `read_line` because it avoids having to scan the
    // data twice to check if there are args.
    let mut interp_offset = None;
    let mut args_offset = None;
    for (i, byte) in reader.bytes().enumerate() {
        let byte = byte.map_err(|_| Error::new(EIO))?;

        match (byte, interp_offset, args_offset) {
            // No args; only interpreter
            (b'\n', None, None) => {
                interp_offset = NonZeroUsize::new(i);
                break;
            }
            // Interpreter found, so we're scanning for where the args ends
            (b'\n', Some(_), None) => {
                args_offset = NonZeroUsize::new(i);
                break;
            }
            // Found args so interpreter ends at `i`
            (b' ', None, None) => {
                interp_offset = NonZeroUsize::new(i);
            }
            _ => {}
        }
    }

    // Interpreter is mandatory since we found #! earlier
    let Some(interp_offset) = interp_offset.map(NonZeroUsize::get) else {
        return Err(Error::new(ENOEXEC));
    };
    // We need u64s and usizes; converting them now is easier
    let Ok(interp_offset_u64) = interp_offset.try_into() else {
        return Err(Error::new(E2BIG));
    };
    let args_offset_u64: Option<NonZeroU64> = args_offset
        .map(|offset| offset.try_into())
        .transpose()
        .map_err(|_| Error::new(E2BIG))?;

    // Spec: full length of the shebang can't exceed max path length
    let shebang_len = pos
        .checked_add(interp_offset_u64)
        .and_then(|len| len.checked_add(args_offset_u64.map(NonZeroU64::get).unwrap_or_default()))
        .ok_or_else(|| Error::new(E2BIG))?;
    // PATH_MAX is a small number that fits into u64 so `as` is okay
    if shebang_len > PATH_MAX as u64 {
        return Err(Error::new(E2BIG));
    }

    // Rewind to the beginning of the interpreter.
    // As above, this is essentially free because the internal buf size is several times larger
    // than PATH_MAX by default, and our shebang_len < PATH_MAX as checked above.
    reader
        .seek(SeekFrom::Start(pos))
        .map_err(|_| Error::new(E2BIG))?;

    let mut interpreter = Vec::with_capacity(interp_offset);
    reader
        .take(interp_offset_u64)
        .read_to_end(&mut interpreter)
        .map_err(|_| Error::new(EIO))?;

    // Read args, but treat as an opaque block to pass to the interpreter.
    // Linux and FreeBSD both pass the args as is to the interpreter whereas macOS splits
    // the args similar to `/usr/bin/env -S`.
    // POSIX leaves the behavior up to the implementation.
    // It's simpler to rely on env because well behaved, portable scripts will use
    // it to ensure correct operation on Linux/FreeBSD.
    // Splitting args ourselves gains little while reinventing env -S
    let interpreter_args = if let (Some(offset), Some(offset_u64)) = (
        args_offset.map(NonZeroUsize::get),
        args_offset_u64.map(NonZeroU64::get),
    ) {
        let len = offset - interp_offset - 1;
        let len_u64 = offset_u64 - interp_offset_u64 - 1;
        let mut args = Vec::with_capacity(len);

        // Eat initial whitespace
        reader.consume(1);
        reader
            .take(len_u64)
            .read_to_end(&mut args)
            .map_err(|_| Error::new(E2BIG))?;
        // Eat '\n'
        reader.consume(1);

        let args = CString::new(args).map_err(|_| Error::new(ENOEXEC))?;
        Some(args)
    } else {
        None
    };

    let interpreter = CString::new(interpreter).map_err(|_| Error::new(ENOEXEC))?;
    Ok((Some(interpreter), interpreter_args))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::parse_interpreter;

    // Shebangs without a script attached
    const NO_FRILLS: &str = "#!/bin/sh\n";
    const NO_FRILLS_EXPECTED: &str = "/bin/sh";

    const SPACE_B4_INTERP: &str = "#! /bin/sh\n";
    const SPACE_B4_INTERP_EXPECTED: &str = "/bin/sh";

    const NO_FRILLS_ENV: &str = "#!/usr/bin/env sh\n";
    const NO_FRILLS_ENV_EXPECTED: &str = "/usr/bin/env";
    const NO_FRILLS_ENV_EXPECTED_ARGS: &str = "sh";

    const SPACE_B4_ENV: &str = "#! /usr/bin/env sh\n";
    const SPACE_B4_EXPECTED: &str = NO_FRILLS_ENV_EXPECTED;
    const SPACE_B4_EXPECTED_ARGS: &str = NO_FRILLS_ENV_EXPECTED_ARGS;

    const MULT_SPACES_B4: &str = "#!                        /usr/bin/env sh\n";
    const MULT_SPACES_B4_EXPECTED: &str = NO_FRILLS_ENV_EXPECTED;
    const MULT_SPACES_B4_EXPECTED_ARGS: &str = NO_FRILLS_ENV_EXPECTED_ARGS;

    // Shebangs with a script attached
    // These test that the parser doesn't run off the first line
    const NO_FRILLS_W_SCRIPT: &str = r#"#!/bin/sh
    echo "Hello from Redox""#;
    const NO_FRILLS_W_SCRIPT_EXPECTED: &str = NO_FRILLS_EXPECTED;

    const SPACE_B4_INTERP_W_SCRIPT: &str = r#"#! /bin/sh
    echo "Doctor Eigenvalue""#;
    const SPACE_B4_INTERP_W_SCRIPT_EXPECTED: &str = NO_FRILLS_EXPECTED;

    const MULT_ARGUMENTS: &str = r#"#! /usr/bin/env -S python -OO
    assert False
    print("This totally works")
    "#;
    const MULT_ARGUMENTS_EXPECTED: &str = NO_FRILLS_ENV_EXPECTED;
    const MULT_ARGUMENTS_EXPECTED_ARGS: &str = "-S python -OO";

    // No hashbang conditions
    const NO_SHEBANG: &str = "/bin/sh";
    const EMPTY: &str = "";

    // Error conditions
    const SHEBANG_NO_INTERP: &str = "#!";
    const SHEBANG_NO_INTERP_SPACE: &str = "#! ";
    const SHEBANG_NO_INTERP_SCRIPT: &str = "#!\necho ${PATH}";

    fn success(input: &str, expected_interp: &str, expected_args: Option<&str>) {
        let mut reader = Cursor::new(input);
        let (actual_interp, actual_args) = parse_interpreter(&mut reader)
            .unwrap_or_else(|e| panic!("Shebang ({input}) should parse\n\t{e}"));

        let actual_interp = actual_interp
            .expect("Expected an interpreter")
            .into_string()
            .expect("Interpreter is ASCII (valid UTF-8)");
        assert_eq!(expected_interp, actual_interp);

        if let Some(expected_args) = expected_args {
            let actual_args = actual_args
                .expect("Expected arguments to interpreter")
                .into_string()
                .expect("Args string is ASCII (valid UTF-8)");
            assert_eq!(expected_args, actual_args);
        }
    }

    #[test]
    fn parse_interpreter_without_space() {
        success(NO_FRILLS, NO_FRILLS_EXPECTED, None);
    }

    #[test]
    fn parse_interpreter_with_space() {
        success(SPACE_B4_INTERP, SPACE_B4_INTERP_EXPECTED, None);
    }

    #[test]
    fn parse_interpreter_with_arg() {
        success(
            NO_FRILLS_ENV,
            NO_FRILLS_ENV_EXPECTED,
            Some(NO_FRILLS_ENV_EXPECTED_ARGS),
        );
    }

    #[test]
    fn parse_interpreter_with_arg_and_space() {
        success(
            SPACE_B4_ENV,
            SPACE_B4_EXPECTED,
            Some(SPACE_B4_EXPECTED_ARGS),
        );
    }

    #[test]
    fn parse_interpreter_with_multiple_spaces() {
        success(
            MULT_SPACES_B4,
            MULT_SPACES_B4_EXPECTED,
            Some(MULT_SPACES_B4_EXPECTED_ARGS),
        );
    }

    #[test]
    fn parse_interpreter_with_script() {
        success(NO_FRILLS_W_SCRIPT, NO_FRILLS_W_SCRIPT_EXPECTED, None);
    }

    #[test]
    fn parse_interpreter_with_script_and_space() {
        success(
            SPACE_B4_INTERP_W_SCRIPT,
            SPACE_B4_INTERP_W_SCRIPT_EXPECTED,
            None,
        );
    }

    #[test]
    fn parse_interpreter_with_script_args_space() {
        success(
            MULT_ARGUMENTS,
            MULT_ARGUMENTS_EXPECTED,
            Some(MULT_ARGUMENTS_EXPECTED_ARGS),
        );
    }

    #[test]
    fn parse_interpreter_no_shebang() {
        let mut reader = Cursor::new(NO_SHEBANG);
        let (interpreter, args) =
            parse_interpreter(&mut reader).expect("Shouldn't fail if file doesn't have a shebang");

        assert!(
            interpreter.is_none(),
            "Interpreter should be `None` if shebang isn't present"
        );
        assert!(
            args.is_none(),
            "Args should be empty without an interpreter."
        );
    }

    #[test]
    fn parse_interpreter_empty() {
        let mut reader = Cursor::new(EMPTY);
        let (interpreter, args) =
            parse_interpreter(&mut reader).expect("Shouldn't fail if file doesn't have a shebang");

        assert!(
            interpreter.is_none(),
            "Interpreter should be `None` for empty image"
        );
        assert!(args.is_none(), "Args should be empty for empty image");
    }

    #[test]
    fn parse_interpreter_no_interpreter_fail() {
        let mut reader = Cursor::new(SHEBANG_NO_INTERP);
        parse_interpreter(&mut reader)
            .expect_err("A hashbang without an interpreter should return an error");
    }

    #[test]
    fn parse_interpreter_no_interpreter_space_fail() {
        let mut reader = Cursor::new(SHEBANG_NO_INTERP_SPACE);
        parse_interpreter(&mut reader)
            .expect_err("A hashbang without an interpreter should return an error");
    }

    #[test]
    fn parse_interpreter_no_interpreter_script_fail() {
        let mut reader = Cursor::new(SHEBANG_NO_INTERP_SCRIPT);
        parse_interpreter(&mut reader)
            .expect_err("A hashbang without an interpreter should return an error");
    }
}
