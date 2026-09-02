//! Synchronous process seam for trusted coordinator argv launches.
//!
//! Accepts an exact executable + argument vector only. Never takes a shell
//! command string. Clears the inherited environment and applies an explicit map.
//! Concurrently drains stdout and stderr under a combined byte cap, and on Unix
//! places the child in a new process group so timeout/overflow can kill and reap
//! the whole group via `killpg`.

use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
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
    let (tx, rx) = mpsc::channel::<DrainEvent>();
    let tx_err = tx.clone();
    let out_handle = thread::spawn(move || drain_pipe(stdout, PipeKind::Stdout, cap, tx));
    let err_handle = thread::spawn(move || drain_pipe(stderr, PipeKind::Stderr, cap, tx_err));

    let deadline = Instant::now() + spec.timeout;
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut overflow = false;
    let mut timed_out = false;

    while !(stdout_done && stderr_done) {
        let now = Instant::now();
        if now >= deadline {
            timed_out = true;
            break;
        }
        let wait = deadline.saturating_duration_since(now);
        match rx.recv_timeout(wait) {
            Ok(DrainEvent::Chunk {
                kind,
                data,
                overflowed,
            }) => {
                if overflowed {
                    overflow = true;
                    break;
                }
                match kind {
                    PipeKind::Stdout => {
                        stdout_buf.extend_from_slice(&data);
                        if stdout_buf.len() + stderr_buf.len() > cap {
                            overflow = true;
                            break;
                        }
                    }
                    PipeKind::Stderr => {
                        stderr_buf.extend_from_slice(&data);
                        if stdout_buf.len() + stderr_buf.len() > cap {
                            overflow = true;
                            break;
                        }
                    }
                }
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
                timed_out = true;
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if overflow || timed_out {
        let _ = terminate_child_group(&mut child);
        let _ = child.wait();
        // Join drainers; ignore their leftover payloads after kill.
        let _ = out_handle.join();
        let _ = err_handle.join();
        // Drop captured buffers so errors never duplicate huge output.
        drop(stdout_buf);
        drop(stderr_buf);
        if overflow {
            return Err(ProcessError::OutputOverflow { cap });
        }
        return Err(ProcessError::Timeout {
            timeout_ms: spec.timeout.as_millis() as u64,
        });
    }

    // Drain may have finished; still collect any trailing events.
    while let Ok(event) = rx.try_recv() {
        match event {
            DrainEvent::Chunk {
                kind,
                data,
                overflowed,
            } => {
                if overflowed
                    || stdout_buf
                        .len()
                        .saturating_add(stderr_buf.len())
                        .saturating_add(data.len())
                        > cap
                {
                    let _ = terminate_child_group(&mut child);
                    let _ = child.wait();
                    let _ = out_handle.join();
                    let _ = err_handle.join();
                    return Err(ProcessError::OutputOverflow { cap });
                }
                match kind {
                    PipeKind::Stdout => stdout_buf.extend_from_slice(&data),
                    PipeKind::Stderr => stderr_buf.extend_from_slice(&data),
                }
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

    let status = child
        .wait()
        .map_err(|err| ProcessError::Io(format!("wait failed: {err}")))?;

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Err(ProcessError::SignalExit { signal });
        }
    }

    let code = status.code().unwrap_or(-1);
    if code != 0 {
        // Bound displayed payload: keep outputs but error Display only shows code.
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
    Chunk {
        kind: PipeKind,
        data: Vec<u8>,
        overflowed: bool,
    },
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
    tx: mpsc::Sender<DrainEvent>,
) {
    let mut buf = [0u8; 8192];
    let mut seen = 0usize;
    loop {
        match pipe.read(&mut buf) {
            Ok(0) => {
                let _ = tx.send(DrainEvent::Done { kind });
                break;
            }
            Ok(n) => {
                seen = seen.saturating_add(n);
                let overflowed = seen > cap;
                let data = if overflowed {
                    Vec::new()
                } else {
                    buf[..n].to_vec()
                };
                if tx
                    .send(DrainEvent::Chunk {
                        kind,
                        data,
                        overflowed,
                    })
                    .is_err()
                {
                    break;
                }
                if overflowed {
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
        // Negative pid to killpg would be kill(-pgid); killpg is explicit.
        let rc = unsafe { libc::killpg(pid, libc::SIGKILL) };
        if rc != 0 {
            let err = io::Error::last_os_error();
            // ESRCH: already gone — treat as success for reap path.
            if err.raw_os_error() != Some(libc::ESRCH) {
                // Fall back to killing the direct child.
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
        // There is no shell-command API: program + args only.
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
        // SECRET must not leak from parent: we set it only in this process env.
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
                // Display must not dump the captured bodies.
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
    fn system_runner_timeout_kills_child_and_grandchild() {
        let dir = TempDir::new().unwrap();
        let marker = dir.path().join("grandchild-live");
        let script = write_executable(
            dir.path(),
            "timeout.sh",
            &format!(
                r#"#!/bin/bash
set -euo pipefail
# Grandchild ignores HUP/TERM and would survive a plain kill of the parent shell.
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

        // Give the kernel a moment, then assert the grandchild is gone.
        thread::sleep(Duration::from_millis(200));
        if marker.exists() {
            let pid_text = fs::read_to_string(&marker).unwrap_or_default();
            let pid: i32 = pid_text.trim().parse().unwrap_or(-1);
            if pid > 1 {
                let still_alive = unsafe { libc::kill(pid, 0) == 0 };
                assert!(
                    !still_alive,
                    "grandchild pid {pid} still alive after group kill"
                );
            }
        }
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
        // Compile-time: ProcessSpec has no command/shell string field.
        let _ = recorded.env;
    }
}
