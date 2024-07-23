use crate::{
    c_str::{CStr, CString},
    fs::File,
    header::string::strlen,
    io::{prelude::*, BufReader, SeekFrom},
    platform::{
        sys::{S_ISGID, S_ISUID},
        types::*,
    },
};

use redox_rt::proc::{ExtraInfo, FdGuard, FexecResult, InterpOverride};
use syscall::{data::Stat, error::*, flag::*};

fn fexec_impl(
    exec_file: FdGuard,
    open_via_dup: FdGuard,
    path: &[u8],
    args: &[&[u8]],
    envs: &[&[u8]],
    total_args_envs_size: usize,
    extrainfo: &ExtraInfo,
    interp_override: Option<InterpOverride>,
) -> Result<usize> {
    let memory = FdGuard::new(syscall::open("memory:", 0)?);

    let addrspace_selection_fd = match redox_rt::proc::fexec_impl(
        exec_file,
        open_via_dup,
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
            open_via_dup,
            path,
            interp_override: new_interp_override,
        } => {
            drop(image_file);
            drop(open_via_dup);
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
) -> Result<usize> {
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
    // executable memory, and in that case we might use the `escalate:` scheme as we already do
    // when the binary needs setuid/setgid.

    let mut stat = Stat::default();
    syscall::fstat(*image_file as usize, &mut stat)?;
    let uid = syscall::getuid()?;
    let gid = syscall::getuid()?;

    let mode = if uid == stat.st_uid as usize {
        (stat.st_mode >> 3 * 2) & 0o7
    } else if gid == stat.st_gid as usize {
        (stat.st_mode >> 3 * 1) & 0o7
    } else {
        stat.st_mode & 0o7
    };

    if mode & 0o1 == 0o0 {
        return Err(Error::new(EPERM));
    }
    let wants_setugid = stat.st_mode & ((S_ISUID | S_ISGID) as u16) != 0;

    let cwd: Box<[u8]> = super::path::clone_cwd().unwrap_or_default().into();

    // Count arguments
    let mut len = 0;

    match arg_env {
        ArgEnv::C { argv, .. } => unsafe {
            while !(*argv.add(len)).is_null() {
                len += 1;
            }
        },
        ArgEnv::Parsed { args, .. } => len = args.len(),
    }

    let mut args: Vec<&[u8]> = Vec::with_capacity(len);

    // Read shebang (for example #!/bin/sh)
    let mut _interpreter_path = None;
    let is_interpreted = {
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
        shebang == *b"#!"
    };
    // Since the fexec implementation is almost fully done in userspace, the kernel can no longer
    // set UID/GID accordingly, and this code checking for them before using interfaces to upgrade
    // UID/GID, can not be trusted. So we ask the `escalate:` scheme for help. Note that
    // `escalate:` can be deliberately excluded from the scheme namespace to deny privilege
    // escalation (such as su/sudo/doas) for untrusted processes.
    //
    // According to execve(2), Linux and most other UNIXes ignore setuid/setgid for interpreted
    // executables and thereby simply keep the privileges as is. For compatibility we do that
    // too.

    if is_interpreted {
        // TODO: Does this support prepending args to the interpreter? E.g.
        // #!/usr/bin/env python3

        // So, this file is interpreted.
        // Then, read the actual interpreter:
        let mut interpreter = Vec::new();
        BufReader::new(&mut image_file)
            .read_until(b'\n', &mut interpreter)
            .map_err(|_| Error::new(EIO))?;
        if interpreter.ends_with(&[b'\n']) {
            interpreter.pop().unwrap();
        }
        let cstring = CString::new(interpreter).map_err(|_| Error::new(ENOEXEC))?;
        image_file = File::open(CStr::borrow(&cstring), O_RDONLY as c_int)
            .map_err(|_| Error::new(ENOENT))?;

        // Make sure path is kept alive long enough, and push it to the arguments
        _interpreter_path = Some(cstring);
        let path_ref = _interpreter_path.as_ref().unwrap();
        args.push(path_ref.as_bytes());
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
        let files_fd = File::new(syscall::open("thisproc:current/filetable", O_RDONLY)? as c_int);
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

    let this_context_fd = FdGuard::new(syscall::open("thisproc:current/open_via_dup", 0)?);
    // TODO: Convert image_file to FdGuard earlier?
    let exec_fd_guard = FdGuard::new(image_file.fd as usize);
    core::mem::forget(image_file);

    if !is_interpreted && wants_setugid {
        // We are now going to invoke `escalate:` rather than loading the program ourselves.
        let escalate_fd = FdGuard::new(syscall::open("escalate:", O_WRONLY)?);

        // First, send the context handle of this process to escalated.
        send_fd_guard(*escalate_fd, this_context_fd)?;

        // Then, send the file descriptor containing the file descriptor to be executed.
        send_fd_guard(*escalate_fd, exec_fd_guard)?;

        // Then, write the path (argv[0]).
        let _ = syscall::write(*escalate_fd, arg0);

        // Second, we write the flattened args and envs with NUL characters separating
        // individual items. This can be copied directly into the new executable's memory.
        let _ = syscall::write(*escalate_fd, &flatten_with_nul(args))?;
        let _ = syscall::write(*escalate_fd, &flatten_with_nul(envs))?;
        let _ = syscall::write(*escalate_fd, &cwd)?;

        // Closing will notify the scheme, and from that point we will no longer have control
        // over this process (unless it fails). We do this manually since drop cannot handle
        // errors.
        let fd = *escalate_fd as usize;
        core::mem::forget(escalate_fd);

        syscall::close(fd)?;

        unreachable!()
    } else {
        let sigprocmask = redox_rt::signal::get_sigmask().unwrap();

        let extrainfo = ExtraInfo {
            cwd: Some(&cwd),
            sigignmask: 0,
            sigprocmask,
        };
        fexec_impl(
            exec_fd_guard,
            this_context_fd,
            arg0,
            &args,
            &envs,
            total_args_envs_size,
            &extrainfo,
            interp_override,
        )
    }
}
fn flatten_with_nul<T>(iter: impl IntoIterator<Item = T>) -> Box<[u8]>
where
    T: AsRef<[u8]>,
{
    let mut vec = Vec::new();
    for item in iter {
        vec.extend(item.as_ref());
        vec.push(b'\0');
    }
    vec.into_boxed_slice()
}

fn send_fd_guard(dst_socket: usize, fd: FdGuard) -> Result<()> {
    syscall::sendfd(dst_socket, *fd, 0, 0)?;
    // The kernel closes file descriptors that are sent, so don't call SYS_CLOSE redundantly.
    core::mem::forget(fd);
    Ok(())
}
