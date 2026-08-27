//! Small Linux launcher for the verification worker.
//!
//! The socketpair is created before this process starts. After the inherited
//! descriptor is installed, no-new-privs and a seccomp filter deny creation of
//! any new socket. The inherited broker fd remains usable, and the filter is
//! inherited by Python and descendants.

use std::env;
use std::process::{Command, ExitCode};

const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;
const PR_SET_SECCOMP: libc::c_int = 22;
const SECCOMP_MODE_FILTER: libc::c_ulong = 2;
const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
const EPERM: u32 = 1;
const SECCOMP_DATA_ARCH: u32 = 4;
const SECCOMP_DATA_NR: u32 = 0;

// These values are the Linux UAPI AUDIT_ARCH_* values from
// linux/uapi/linux/audit.h. Keep the selected value target-bound: accepting a
// second architecture in one binary would make the syscall policy ambiguous.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const AUDIT_ARCH_X86_64: u32 = 0xc000_003e;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const CURRENT_AUDIT_ARCH: u32 = AUDIT_ARCH_X86_64;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const AUDIT_ARCH_AARCH64: u32 = 0xc000_00b7;

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
const CURRENT_AUDIT_ARCH: u32 = AUDIT_ARCH_AARCH64;

// This binary is deliberately Linux-only and must never silently select the
// x86_64 policy for an unsupported target.
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
compile_error!("ytdlp-sandbox supports only Linux x86_64 and AArch64");

fn stmt(code: u16, k: u32) -> libc::sock_filter {
    libc::sock_filter {
        code,
        jt: 0,
        jf: 0,
        k,
    }
}

fn jump(code: u16, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
    libc::sock_filter { code, jt, jf, k }
}

fn install_filter() -> Result<(), String> {
    let filter = [
        stmt(
            (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16,
            SECCOMP_DATA_ARCH,
        ),
        jump(
            (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
            CURRENT_AUDIT_ARCH,
            1,
            0,
        ),
        stmt(
            (libc::BPF_RET | libc::BPF_K) as u16,
            SECCOMP_RET_KILL_PROCESS,
        ),
        stmt(
            (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16,
            SECCOMP_DATA_NR,
        ),
        jump(
            (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
            libc::SYS_socket as u32,
            2,
            0,
        ),
        jump(
            (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
            libc::SYS_socketpair as u32,
            1,
            0,
        ),
        stmt(
            (libc::BPF_RET | libc::BPF_K) as u16,
            libc::SECCOMP_RET_ALLOW,
        ),
        stmt(
            (libc::BPF_RET | libc::BPF_K) as u16,
            SECCOMP_RET_ERRNO | EPERM,
        ),
    ];
    let mut program = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_ptr() as *mut libc::sock_filter,
    };
    unsafe {
        if libc::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        if libc::prctl(PR_SET_SECCOMP, SECCOMP_MODE_FILTER, &mut program) != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn audit_arch_is_exactly_bound_to_the_compile_target() {
        let expected = match std::env::consts::ARCH {
            "x86_64" => 0xc000_003e,
            "aarch64" => 0xc000_00b7,
            architecture => panic!("unsupported architecture: {architecture}"),
        };
        assert_eq!(super::CURRENT_AUDIT_ARCH, expected);
    }
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(flag) = args.next() else {
        return ExitCode::from(64);
    };
    if flag != "--fd" || args.next().is_none() || args.next().as_deref() != Some("--") {
        return ExitCode::from(64);
    }
    // The parent supplies the fixed descriptor number. Do not accept an
    // arbitrary inherited fd or a caller-selected environment configuration.
    if install_filter().is_err() {
        return ExitCode::from(111);
    }
    let Some(program) = args.next() else {
        return ExitCode::from(64);
    };
    let status = Command::new(program).args(args).status();
    match status {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(_) => ExitCode::from(127),
    }
}
