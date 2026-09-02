//! Synchronous process seam for trusted coordinator argv launches.
//!
//! Accepts an exact executable + argument vector only. Never takes a shell
//! command string. Clears the inherited environment and applies an explicit map.
//! Concurrently drains stdout and stderr under a combined byte cap, and on Unix
//! places the child in a new process group so timeout/overflow can kill and reap
//! the whole group via `killpg`.
//!
//! Wall-clock supervision continues until the child exits **and** both pipes
//! reach EOF, or until the deadline/overflow path kills the group. Closing the
//! child's own descriptors does not end supervision early.
//!
//! Combined output cap is a hard memory bound: drainers share an
//! `AtomicUsize` reservation counter so channel payloads plus receiver buffers
//! never exceed `cap`. The only slack is the 8 KiB read scratch buffer local to
//! each drainer (not retained after a rejected reservation).

use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors raised while launching or supervising a child process.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProcessError {
    /// Spec failed structural checks before spawn.
    #[error("invalid process spec: {0}")]
    Invalid(String),
    /// Operating-system failure while spawning or waiting.
    #[error("process io error: {0}")]
    Io(String),
    /// Combined stdout+stderr exceeded the configured byte cap.
    #[error("process output exceeded {cap} bytes")]
    OutputOverflow { cap: usize },
    /// Child did not exit before the timeout; the process group was killed.
    #[error("process timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u64 },
    /// Child exited with a non-zero status.
    #[error("process exited with status {code}")]
    NonZeroExit {
        code: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    /// Child terminated by signal (Unix) without a normal exit code.
    #[error("process terminated by signal {signal}")]
    SignalExit { signal: i32 },
}

/// Exact argv launch request. No shell-string field exists on purpose.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    /// Absolute or PATH-resolved executable (first argv element is separate).
    pub program: PathBuf,
    /// Remaining argv elements after `program`.
    pub args: Vec<String>,
    /// Working directory for the child.
    pub cwd: PathBuf,
    /// Explicit environment after clearing inheritance. Keys are case-sensitive.
    pub env: BTreeMap<String, String>,
    /// Combined stdout + stderr capture ceiling in bytes.
    pub output_cap: usize,
    /// Wall-clock timeout for the whole child lifetime.
    pub timeout: Duration,
}

impl ProcessSpec {
    /// Build a spec from validated pieces. Rejects empty program and zero caps.
    pub fn new(
        program: impl Into<PathBuf>,
        args: impl IntoIterator<Item = impl Into<String>>,
        cwd: impl Into<PathBuf>,
        env: BTreeMap<String, String>,
        output_cap: usize,
        timeout: Duration,
    ) -> Result<Self, ProcessError> {
        let program = program.into();
        if program.as_os_str().is_empty() {
            return Err(ProcessError::Invalid("program must be nonempty".to_owned()));
        }
        if output_cap == 0 {
            return Err(ProcessError::Invalid(
                "output_cap must be greater than zero".to_owned(),
            ));
        }
        if timeout.is_zero() {
            return Err(ProcessError::Invalid(
                "timeout must be greater than zero".to_owned(),
            ));
        }
        let args: Vec<String> = args.into_iter().map(Into::into).collect();
        for (index, arg) in args.iter().enumerate() {
            if arg.contains('\0') {
                return Err(ProcessError::Invalid(format!("args[{index}] contains NUL")));
            }
        }
        Ok(Self {
            program,
            args,
            cwd: cwd.into(),
            env,
            output_cap,
            timeout,
        })
    }
}

/// Successful child result with bounded captured output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl ProcessOutput {
    /// Total captured bytes across both pipes.
    pub fn total_bytes(&self) -> usize {
        self.stdout.len().saturating_add(self.stderr.len())
    }
}

/// Synchronous process runner seam (real + test fake).
pub trait ProcessRunner {
    /// Launch `spec` and wait for completion under cap/timeout rules.
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError>;
}

/// Production runner using `std::process::Command`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        run_system(spec)
    }
}

fn run_system(spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
    validate_cwd(&spec.cwd)?;

    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .current_dir(&spec.cwd)
        .env_clear()
        .envs(&spec.env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Put the child in a new process group so killpg reaps grandchildren too.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let mut child = command
        .spawn()
        .map_err(|err| ProcessError::Io(format!("spawn {}: {err}", spec.program.display())))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProcessError::Io("missing stdout pipe".to_owned()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ProcessError::Io("missing stderr pipe".to_owned()))?;

    let cap = spec.output_cap;
    // Shared reservation so channel + buffers never exceed `cap`.
    let reserved = Arc::new(AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel::<DrainEvent>();
    let tx_err = tx.clone();
    let reserved_out = Arc::clone(&reserved);
    let reserved_err = Arc::clone(&reserved);
    let out_handle =
        thread::spawn(move || drain_pipe(stdout, PipeKind::Stdout, cap, reserved_out, tx));
    let err_handle =
        thread::spawn(move || drain_pipe(stderr, PipeKind::Stderr, cap, reserved_err, tx_err));

    let deadline = Instant::now() + spec.timeout;
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut child_status: Option<std::process::ExitStatus> = None;
    let mut overflow = false;
    let mut timed_out = false;

    // Supervise until: (child exited AND both pipes EOF) OR timeout OR overflow.
    // Pipe EOF alone is not enough (child may close fds and keep running).
    // Child exit alone is not enough (descendants may still hold pipe writers).
    loop {
        if overflow {
            break;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            break;
        }

        if child_status.is_none() {
            match child.try_wait() {
                Ok(Some(status)) => child_status = Some(status),
                Ok(None) => {}
                Err(err) => {
                    let _ = terminate_child_group(&mut child);
                    let _ = child.wait();
                    let _ = out_handle.join();
                    let _ = err_handle.join();
                    return Err(ProcessError::Io(format!("wait failed: {err}")));
                }
            }
        }

        if child_status.is_some() && stdout_done && stderr_done {
            break;
        }

        let now = Instant::now();
        let wait = deadline
            .saturating_duration_since(now)
            .min(Duration::from_millis(20));

        // Once drainers are gone the channel is permanently disconnected; do not
        // call recv again (it returns immediately and would hot-spin). Poll the
        // child with a single sleep per iteration.
        if stdout_done && stderr_done {
            if child_status.is_some() {
                break;
            }
            thread::sleep(wait);
            continue;
        }

        match rx.recv_timeout(wait) {
            Ok(DrainEvent::Chunk { kind, data }) => match kind {
                PipeKind::Stdout => stdout_buf.extend_from_slice(&data),
                PipeKind::Stderr => stderr_buf.extend_from_slice(&data),
            },
            Ok(DrainEvent::Overflow) => {
                overflow = true;
            }
            Ok(DrainEvent::Done { kind }) => match kind {
                PipeKind::Stdout => stdout_done = true,
                PipeKind::Stderr => stderr_done = true,
            },
            Ok(DrainEvent::IoError { message }) => {
                let _ = terminate_child_group(&mut child);
                let _ = child.wait();
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(ProcessError::Io(message));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Poll interval elapsed; loop re-checks deadline and child.
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Drainers finished; mark pipes done. Next iteration sleeps via
                // the stdout_done&&stderr_done branch (no double sleep here).
                stdout_done = true;
                stderr_done = true;
            }
        }
    }

    if overflow || timed_out {
        let _ = terminate_child_group(&mut child);
        let _ = child.wait();
        let _ = out_handle.join();
        let _ = err_handle.join();
        // Drain any remaining channel events without retaining bodies.
        while let Ok(_event) = rx.try_recv() {}
        drop(stdout_buf);
        drop(stderr_buf);
        if overflow {
            return Err(ProcessError::OutputOverflow { cap });
        }
        return Err(ProcessError::Timeout {
            timeout_ms: spec.timeout.as_millis() as u64,
        });
    }

    // Collect any trailing reserved chunks after exit+EOF.
    while let Ok(event) = rx.try_recv() {
        match event {
            DrainEvent::Chunk { kind, data } => match kind {
                PipeKind::Stdout => stdout_buf.extend_from_slice(&data),
                PipeKind::Stderr => stderr_buf.extend_from_slice(&data),
            },
            DrainEvent::Overflow => {
                let _ = terminate_child_group(&mut child);
                let _ = child.wait();
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(ProcessError::OutputOverflow { cap });
            }
            DrainEvent::Done { .. } => {}
            DrainEvent::IoError { message } => {
                let _ = terminate_child_group(&mut child);
                let _ = child.wait();
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(ProcessError::Io(message));
            }
        }
    }

    let _ = out_handle.join();
    let _ = err_handle.join();

    let status = match child_status {
        Some(status) => status,
        None => {
            // Should not happen: loop only exits cleanly when status is Some.
            // Defensive deadline-bounded wait.
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => break status,
                    Ok(None) => {
                        if Instant::now() >= deadline {
                            let _ = terminate_child_group(&mut child);
                            let _ = child.wait();
                            return Err(ProcessError::Timeout {
                                timeout_ms: spec.timeout.as_millis() as u64,
                            });
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => {
                        return Err(ProcessError::Io(format!("wait failed: {err}")));
                    }
                }
            }
        }
    };

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Err(ProcessError::SignalExit { signal });
        }
    }

    let code = status.code().unwrap_or(-1);
    if code != 0 {
        return Err(ProcessError::NonZeroExit {
            code,
            stdout: stdout_buf,
            stderr: stderr_buf,
        });
    }

    let output = ProcessOutput {
        status: code,
        stdout: stdout_buf,
        stderr: stderr_buf,
    };
    if output.total_bytes() > cap {
        return Err(ProcessError::OutputOverflow { cap });
    }
    Ok(output)
}

fn validate_cwd(cwd: &Path) -> Result<(), ProcessError> {
    let meta = std::fs::metadata(cwd)
        .map_err(|err| ProcessError::Invalid(format!("cwd {}: {err}", cwd.display())))?;
    if !meta.is_dir() {
        return Err(ProcessError::Invalid(format!(
            "cwd is not a directory: {}",
            cwd.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum PipeKind {
    Stdout,
    Stderr,
}

enum DrainEvent {
    /// Reserved chunk already counted against the shared combined cap.
    Chunk {
        kind: PipeKind,
        data: Vec<u8>,
    },
    /// Combined reservation would exceed cap; no payload retained.
    Overflow,
    Done {
        kind: PipeKind,
    },
    IoError {
        message: String,
    },
}

fn drain_pipe<R: Read + Send + 'static>(
    mut pipe: R,
    kind: PipeKind,
    cap: usize,
    reserved: Arc<AtomicUsize>,
    tx: mpsc::Sender<DrainEvent>,
) {
    // Scratch only — never retained after a failed reservation.
    let mut buf = [0u8; 8192];
    loop {
        match pipe.read(&mut buf) {
            Ok(0) => {
                let _ = tx.send(DrainEvent::Done { kind });
                break;
            }
            Ok(n) => {
                // Atomically reserve `n` against the combined cap. On overflow,
                // release nothing (we never added) and signal without retaining
                // the rejected bytes. Channel payloads are only successful
                // reservations, so peak retained memory ≤ cap (+ local scratch).
                let mut current = reserved.load(Ordering::Relaxed);
                let overflowed = loop {
                    if current.saturating_add(n) > cap {
                        break true;
                    }
                    match reserved.compare_exchange_weak(
                        current,
                        current + n,
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break false,
                        Err(observed) => current = observed,
                    }
                };
                if overflowed {
                    let _ = tx.send(DrainEvent::Overflow);
                    break;
                }
                let data = buf[..n].to_vec();
                if tx.send(DrainEvent::Chunk { kind, data }).is_err() {
                    break;
                }
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                let _ = tx.send(DrainEvent::IoError {
                    message: format!("read {kind:?}: {err}"),
                });
                break;
            }
        }
    }
}

fn terminate_child_group(child: &mut std::process::Child) -> io::Result<()> {
    #[cfg(unix)]
    {
        let pid = child.id() as libc::pid_t;
        let rc = unsafe { libc::killpg(pid, libc::SIGKILL) };
        if rc != 0 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() != Some(libc::ESRCH) {
                let _ = child.kill();
                return Err(err);
            }
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        child.kill()
    }
}

/// Test double that records specs and returns scripted results.
#[derive(Default)]
pub struct FakeProcessRunner {
    pub calls: std::sync::Mutex<Vec<ProcessSpec>>,
    pub handler: std::sync::Mutex<
        Option<Box<dyn FnMut(&ProcessSpec) -> Result<ProcessOutput, ProcessError> + Send>>,
    >,
}

impl FakeProcessRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_handler<F>(&self, handler: F)
    where
        F: FnMut(&ProcessSpec) -> Result<ProcessOutput, ProcessError> + Send + 'static,
    {
        *self.handler.lock().expect("handler lock") = Some(Box::new(handler));
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().expect("calls lock").len()
    }

    pub fn last_spec(&self) -> Option<ProcessSpec> {
        self.calls.lock().expect("calls lock").last().cloned()
    }
}

impl ProcessRunner for FakeProcessRunner {
    fn run(&self, spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        self.calls.lock().expect("calls lock").push(spec.clone());
        let mut guard = self.handler.lock().expect("handler lock");
        match guard.as_mut() {
            Some(handler) => handler(spec),
            None => Ok(ProcessOutput {
                status: 0,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;

    fn write_executable(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, body).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    fn base_env() -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();
        env.insert("PATH".to_owned(), "/usr/bin:/bin".to_owned());
        env
    }

    #[test]
    fn rejects_shell_string_program_shape_via_spec_api() {
        let err = ProcessSpec::new(
            "",
            ["-c", "echo hi"],
            "/tmp",
            base_env(),
            1024,
            Duration::from_secs(1),
        )
        .unwrap_err();
        assert!(matches!(err, ProcessError::Invalid(_)));
    }

    #[test]
    fn system_runner_captures_success_and_clears_env() {
        let dir = TempDir::new().unwrap();
        let script = write_executable(
            dir.path(),
            "printenv.sh",
            "#!/bin/bash\necho -n \"HOME=${HOME:-}\"\necho -n \"|SECRET=${SECRET:-}\"\necho -n ok >&2\n",
        );
        let mut env = base_env();
        env.insert("HOME".to_owned(), "/tmp/mission-home".to_owned());
        std::env::set_var("SECRET", "should-not-leak");
        let spec = ProcessSpec::new(
            "/bin/bash",
            [script.to_string_lossy().into_owned()],
            dir.path(),
            env,
            4096,
            Duration::from_secs(5),
        )
        .unwrap();
        let out = SystemProcessRunner.run(&spec).unwrap();
        std::env::remove_var("SECRET");
        assert_eq!(out.status, 0);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("HOME=/tmp/mission-home"), "{stdout}");
        assert!(stdout.contains("SECRET="), "{stdout}");
        assert!(!stdout.contains("should-not-leak"), "{stdout}");
        assert_eq!(String::from_utf8_lossy(&out.stderr), "ok");
    }

    #[test]
    fn system_runner_nonzero_exit_is_error_without_display_duplication() {
        let dir = TempDir::new().unwrap();
        let script = write_executable(
            dir.path(),
            "fail.sh",
            "#!/bin/bash\necho boom-out\necho boom-err >&2\nexit 7\n",
        );
        let spec = ProcessSpec::new(
            "/bin/bash",
            [script.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            4096,
            Duration::from_secs(5),
        )
        .unwrap();
        let err = SystemProcessRunner.run(&spec).unwrap_err();
        match &err {
            ProcessError::NonZeroExit {
                code,
                stdout,
                stderr,
            } => {
                assert_eq!(*code, 7);
                assert_eq!(stdout, b"boom-out\n");
                assert_eq!(stderr, b"boom-err\n");
                let display = err.to_string();
                assert!(display.contains("status 7"));
                assert!(!display.contains("boom-out"));
                assert!(!display.contains("boom-err"));
            }
            other => panic!("expected NonZeroExit, got {other:?}"),
        }
    }

    #[test]
    fn system_runner_combined_cap_overflow_across_pipes() {
        let dir = TempDir::new().unwrap();
        // 30 bytes stdout + 30 bytes stderr with cap 40 must overflow.
        let script = write_executable(
            dir.path(),
            "flood.sh",
            "#!/bin/bash\npython3 - <<'PY'\nimport sys\nsys.stdout.write('o'*30)\nsys.stdout.flush()\nsys.stderr.write('e'*30)\nsys.stderr.flush()\nimport time\ntime.sleep(2)\nPY\n",
        );
        let spec = ProcessSpec::new(
            "/bin/bash",
            [script.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            40,
            Duration::from_secs(5),
        )
        .unwrap();
        let err = SystemProcessRunner.run(&spec).unwrap_err();
        assert!(
            matches!(err, ProcessError::OutputOverflow { cap: 40 }),
            "got {err:?}"
        );
        let display = err.to_string();
        assert!(!display.contains("oooo"));
    }

    #[test]
    fn system_runner_hard_cap_peak_memory_two_pipes() {
        let dir = TempDir::new().unwrap();

        // Success path near the cap: total retained lengths must stay ≤ cap.
        let under = write_executable(
            dir.path(),
            "under_cap.sh",
            "#!/bin/bash\npython3 - <<'PY'\nimport sys\nsys.stdout.write('a'*30)\nsys.stdout.flush()\nsys.stderr.write('b'*30)\nsys.stderr.flush()\nPY\n",
        );
        let under_spec = ProcessSpec::new(
            "/bin/bash",
            [under.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            64,
            Duration::from_secs(5),
        )
        .unwrap();
        let out = SystemProcessRunner.run(&under_spec).unwrap();
        assert_eq!(out.status, 0);
        assert_eq!(out.stdout.len(), 30);
        assert_eq!(out.stderr.len(), 30);
        assert!(
            out.total_bytes() <= 64,
            "retained output must honour combined cap, got {}",
            out.total_bytes()
        );

        // Overflow path: concurrent writers beyond cap → OutputOverflow, no dump.
        let over = write_executable(
            dir.path(),
            "hardcap.sh",
            "#!/bin/bash\npython3 - <<'PY'\nimport os, threading, time\ndef blast(fd, n):\n    os.write(fd, b'x'*n)\nt1=threading.Thread(target=blast, args=(1, 200_000))\nt2=threading.Thread(target=blast, args=(2, 200_000))\nt1.start(); t2.start(); t1.join(); t2.join()\ntime.sleep(1)\nPY\n",
        );
        let cap = 8192usize;
        let over_spec = ProcessSpec::new(
            "/bin/bash",
            [over.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            cap,
            Duration::from_secs(5),
        )
        .unwrap();
        let err = SystemProcessRunner.run(&over_spec).unwrap_err();
        assert!(
            matches!(err, ProcessError::OutputOverflow { cap: c } if c == cap),
            "got {err:?}"
        );
        assert!(!err.to_string().contains("xxxx"));
    }

    #[test]
    fn system_runner_timeout_kills_child_and_grandchild() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("grandchild-live");
        let script = write_executable(
            dir.path(),
            "timeout.sh",
            &format!(
                r#"#!/bin/bash
set -euo pipefail
python3 - <<'PY' &
import os, signal, time
from pathlib import Path
signal.signal(signal.SIGTERM, signal.SIG_IGN)
signal.signal(signal.SIGHUP, signal.SIG_IGN)
Path(r"{marker}").write_text(str(os.getpid()))
while True:
    time.sleep(0.2)
PY
wait
"#,
                marker = marker.display()
            ),
        );
        let spec = ProcessSpec::new(
            "/bin/bash",
            [script.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            64 * 1024,
            Duration::from_millis(400),
        )
        .unwrap();
        let started = Instant::now();
        let err = SystemProcessRunner.run(&spec).unwrap_err();
        assert!(
            matches!(err, ProcessError::Timeout { .. }),
            "expected timeout, got {err:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "timeout path took too long: {:?}",
            started.elapsed()
        );
        thread::sleep(Duration::from_millis(200));
        assert!(marker.exists(), "grandchild must have written its pid");
        let pid_text = fs::read_to_string(&marker).unwrap();
        let pid: i32 = pid_text.trim().parse().expect("pid");
        assert!(pid > 1);
        let still_alive = unsafe { libc::kill(pid, 0) == 0 };
        assert!(
            !still_alive,
            "grandchild pid {pid} still alive after group kill"
        );
    }

    #[test]
    fn system_runner_timeout_when_child_closes_pipes_and_keeps_running() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("descendant-pid");
        let script = write_executable(
            dir.path(),
            "close_pipes.sh",
            &format!(
                r#"#!/bin/bash
set -euo pipefail
# Record this process's pid, then close pipes and hang (descendant of bash).
echo $$ > "{marker}"
exec 1>&-
exec 2>&-
sleep 60
"#,
                marker = marker.display()
            ),
        );
        let timeout = Duration::from_millis(500);
        let spec = ProcessSpec::new(
            "/bin/bash",
            [script.to_string_lossy().into_owned()],
            dir.path(),
            base_env(),
            64 * 1024,
            timeout,
        )
        .unwrap();
        let started = Instant::now();
        let err = SystemProcessRunner.run(&spec).unwrap_err();
        let elapsed = started.elapsed();
        assert!(
            matches!(err, ProcessError::Timeout { .. }),
            "expected Timeout after closed pipes, got {err:?} elapsed={elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "elapsed {elapsed:?} exceeded deadline bound"
        );
        assert!(
            elapsed >= timeout.saturating_sub(Duration::from_millis(100)),
            "elapsed {elapsed:?} finished before deadline {timeout:?}"
        );

        // Causal reaping: descendant pid must have been written and must be gone.
        assert!(
            marker.exists(),
            "closed-pipes child must write descendant pid before hanging"
        );
        let pid_text = fs::read_to_string(&marker).unwrap();
        let pid: i32 = pid_text.trim().parse().expect("descendant pid");
        assert!(pid > 1, "invalid pid {pid}");
        thread::sleep(Duration::from_millis(100));
        let still_alive = unsafe { libc::kill(pid, 0) == 0 };
        assert!(
            !still_alive,
            "descendant pid {pid} still alive after closed-pipes timeout kill/reap"
        );
    }

    #[test]
    fn fake_runner_records_exact_argv_without_shell_field() {
        let fake = FakeProcessRunner::new();
        fake.set_handler(|_| {
            Ok(ProcessOutput {
                status: 0,
                stdout: b"ok".to_vec(),
                stderr: Vec::new(),
            })
        });
        let spec = ProcessSpec::new(
            "bash",
            ["scripts/opportunity-next-mission.sh"],
            "/repo",
            base_env(),
            1024,
            Duration::from_secs(1),
        )
        .unwrap();
        let out = fake.run(&spec).unwrap();
        assert_eq!(out.stdout, b"ok");
        let recorded = fake.last_spec().unwrap();
        assert_eq!(recorded.program, PathBuf::from("bash"));
        assert_eq!(
            recorded.args,
            vec!["scripts/opportunity-next-mission.sh".to_owned()]
        );
        let _ = recorded.env;
    }
}
