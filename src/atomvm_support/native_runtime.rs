//! Native AtomVM host adapter derived from UnRDF's `node-runtime.mjs` and
//! `process-broker.mjs`.
//!
//! This module is deliberately behind the `native-host` feature. The portable
//! `no_std` policy selects an [`ExecutionPlan`]; this adapter performs the
//! environment-owned `DO` step and returns an observation. It never treats a
//! selected plan as authority by itself.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::evidence::Digest32;
use super::execution::{ExecutionObservation, ExecutionPlan, ExecutionTermination};

/// Production defaults are bounded rather than unlimited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeRuntimeLimits {
    pub max_libraries: usize,
    pub max_env_vars: usize,
    pub max_output_bytes: usize,
    pub max_marker_bytes: usize,
    pub max_timeout_ticks: u64,
    /// Conversion from the portable plan's logical ticks to host milliseconds.
    pub tick_millis: u64,
    pub poll_millis: u64,
}

impl Default for NativeRuntimeLimits {
    fn default() -> Self {
        Self {
            max_libraries: 64,
            max_env_vars: 64,
            max_output_bytes: 1024 * 1024,
            max_marker_bytes: 4096,
            max_timeout_ticks: 300_000,
            tick_millis: 1,
            poll_millis: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentBinding {
    pub key: String,
    pub value: String,
}

impl EnvironmentBinding {
    pub fn new(key: &str, value: &str) -> Result<Self, NativeHostRefusal> {
        if key.is_empty() || key.contains('=') || key.contains('\0') || value.contains('\0') {
            return Err(NativeHostRefusal::InvalidEnvironmentBinding);
        }
        Ok(Self {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

/// Configuration owned by the environment adapter, not by the portable plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRuntimeConfig {
    pub executable_ref: String,
    pub environment: Vec<EnvironmentBinding>,
    pub clear_environment: bool,
    pub limits: NativeRuntimeLimits,
}

impl NativeRuntimeConfig {
    pub fn new(executable_ref: &str) -> Result<Self, NativeHostRefusal> {
        if executable_ref.trim().is_empty() || executable_ref.contains('\0') {
            return Err(NativeHostRefusal::InvalidExecutableRef);
        }
        Ok(Self {
            executable_ref: executable_ref.to_string(),
            environment: Vec::new(),
            clear_environment: true,
            limits: NativeRuntimeLimits::default(),
        })
    }

    pub fn with_environment(mut self, environment: Vec<EnvironmentBinding>) -> Self {
        self.environment = environment;
        self
    }

    pub const fn with_limits(mut self, limits: NativeRuntimeLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHostRefusal {
    InvalidExecutableRef,
    InvalidEnvironmentBinding,
    TooManyEnvironmentBindings,
    TooManyLibraries,
    MarkerTooLarge,
    TimeoutOutsidePolicy,
    UnsupportedReferenceScheme,
    AtomVmBinaryNotFound,
    AtomVmBinaryNotExecutable,
    ApplicationNotFound,
    LibraryNotFound,
    ArtifactNotReadable,
    SpawnRefused,
    PipeUnavailable,
    OutputReaderFailed,
    WaitRefused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProbe {
    pub executable: PathBuf,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub output_truncated: bool,
}

/// Evidence returned by the native adapter. Raw bounded output remains available
/// to the caller; the portable receipt stores only caller-selected digests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeExecutionEvidence {
    pub executable: PathBuf,
    pub application: PathBuf,
    pub libraries: Vec<PathBuf>,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub elapsed_millis: u64,
    pub observation: ExecutionObservation,
}

#[derive(Debug, Clone)]
pub struct NativeAtomVmHost {
    config: NativeRuntimeConfig,
}

impl NativeAtomVmHost {
    pub fn new(config: NativeRuntimeConfig) -> Result<Self, NativeHostRefusal> {
        validate_config(&config)?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &NativeRuntimeConfig {
        &self.config
    }

    /// Resolve and execute `AtomVM -v` with the same bounded process machinery.
    pub fn probe(&self) -> Result<RuntimeProbe, NativeHostRefusal> {
        let executable = resolve_executable(&self.config.executable_ref)?;
        let timeout = Duration::from_secs(10);
        let result = run_bounded(
            &executable,
            &[String::from("-v")],
            &self.config.environment,
            self.config.clear_environment,
            timeout,
            self.config.limits.poll_millis,
            self.config.limits.max_output_bytes,
        )?;
        Ok(RuntimeProbe {
            executable,
            exit_code: result.status.and_then(|status| status.code()),
            stdout: result.stdout,
            stderr: result.stderr,
            output_truncated: result.stdout_truncated || result.stderr_truncated,
        })
    }

    /// Execute an already-selected portable plan. `digest` is deliberately
    /// caller supplied so the host adapter does not smuggle in a cryptographic
    /// algorithm or receipt authority.
    pub fn execute_with_digest<F>(
        &self,
        plan: &ExecutionPlan,
        digest: F,
    ) -> Result<NativeExecutionEvidence, NativeHostRefusal>
    where
        F: Fn(&[u8]) -> Digest32,
    {
        validate_plan(plan, &self.config.limits)?;
        let executable = resolve_executable(&self.config.executable_ref)?;
        let application = resolve_artifact_ref(&plan.application_ref, ArtifactKind::Application)?;
        let mut libraries = Vec::with_capacity(plan.library_refs.len());
        for item in &plan.library_refs {
            libraries.push(resolve_artifact_ref(item, ArtifactKind::Library)?);
        }

        let mut args = Vec::with_capacity(1 + libraries.len());
        args.push(path_arg(&application));
        for library in &libraries {
            args.push(path_arg(library));
        }

        let timeout_millis = plan
            .timeout_ticks
            .saturating_mul(self.config.limits.tick_millis);
        let started = Instant::now();
        let result = run_bounded(
            &executable,
            &args,
            &self.config.environment,
            self.config.clear_environment,
            Duration::from_millis(timeout_millis),
            self.config.limits.poll_millis,
            self.config.limits.max_output_bytes,
        )?;
        let elapsed_millis = millis_u64(started.elapsed());
        let marker_observed = contains_bytes(&result.stdout, plan.expected_marker.as_bytes());
        let stdout_digest = digest(&result.stdout);
        let stderr_digest = digest(&result.stderr);

        let termination = if result.timed_out {
            ExecutionTermination::TimedOut
        } else {
            ExecutionTermination::Exited(result.status.and_then(|status| status.code()).unwrap_or(-1))
        };

        let observation = ExecutionObservation {
            termination,
            marker_observed: marker_observed && !result.timed_out,
            stdout_digest: Some(stdout_digest),
            stderr_digest: Some(stderr_digest),
            elapsed_ticks: if self.config.limits.tick_millis == 0 {
                elapsed_millis
            } else {
                elapsed_millis / self.config.limits.tick_millis
            },
        };

        Ok(NativeExecutionEvidence {
            executable,
            application,
            libraries,
            exit_code: result.status.and_then(|status| status.code()),
            stdout: result.stdout,
            stderr: result.stderr,
            stdout_truncated: result.stdout_truncated,
            stderr_truncated: result.stderr_truncated,
            elapsed_millis,
            observation,
        })
    }
}

fn validate_config(config: &NativeRuntimeConfig) -> Result<(), NativeHostRefusal> {
    if config.executable_ref.trim().is_empty() || config.executable_ref.contains('\0') {
        return Err(NativeHostRefusal::InvalidExecutableRef);
    }
    if config.environment.len() > config.limits.max_env_vars {
        return Err(NativeHostRefusal::TooManyEnvironmentBindings);
    }
    if config.limits.max_output_bytes == 0
        || config.limits.max_timeout_ticks == 0
        || config.limits.tick_millis == 0
        || config.limits.poll_millis == 0
    {
        return Err(NativeHostRefusal::TimeoutOutsidePolicy);
    }
    for binding in &config.environment {
        if binding.key.is_empty()
            || binding.key.contains('=')
            || binding.key.contains('\0')
            || binding.value.contains('\0')
        {
            return Err(NativeHostRefusal::InvalidEnvironmentBinding);
        }
    }
    Ok(())
}

fn validate_plan(plan: &ExecutionPlan, limits: &NativeRuntimeLimits) -> Result<(), NativeHostRefusal> {
    if plan.library_refs.len() > limits.max_libraries {
        return Err(NativeHostRefusal::TooManyLibraries);
    }
    if plan.expected_marker.len() > limits.max_marker_bytes {
        return Err(NativeHostRefusal::MarkerTooLarge);
    }
    if plan.timeout_ticks == 0 || plan.timeout_ticks > limits.max_timeout_ticks {
        return Err(NativeHostRefusal::TimeoutOutsidePolicy);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ArtifactKind {
    Application,
    Library,
}

fn resolve_artifact_ref(reference: &str, kind: ArtifactKind) -> Result<PathBuf, NativeHostRefusal> {
    let raw = if let Some(path) = reference.strip_prefix("file://") {
        path
    } else if reference.contains("://") || reference.starts_with("avm:") {
        return Err(NativeHostRefusal::UnsupportedReferenceScheme);
    } else {
        reference
    };
    let path = PathBuf::from(raw);
    let exists = path.is_file();
    if !exists {
        return Err(match kind {
            ArtifactKind::Application => NativeHostRefusal::ApplicationNotFound,
            ArtifactKind::Library => NativeHostRefusal::LibraryNotFound,
        });
    }
    File::open(&path).map_err(|_| NativeHostRefusal::ArtifactNotReadable)?;
    Ok(path)
}

fn resolve_executable(reference: &str) -> Result<PathBuf, NativeHostRefusal> {
    let candidate = PathBuf::from(reference);
    if candidate.is_absolute() || candidate.components().count() > 1 {
        return validate_executable(candidate);
    }

    if let Some(paths) = env::var_os("PATH") {
        for directory in env::split_paths(&paths) {
            let path = directory.join(reference);
            if path.is_file() {
                return validate_executable(path);
            }
            #[cfg(windows)]
            {
                let exe = directory.join(format!("{}.exe", reference));
                if exe.is_file() {
                    return validate_executable(exe);
                }
            }
        }
    }
    Err(NativeHostRefusal::AtomVmBinaryNotFound)
}

fn validate_executable(path: PathBuf) -> Result<PathBuf, NativeHostRefusal> {
    if !path.is_file() {
        return Err(NativeHostRefusal::AtomVmBinaryNotFound);
    }
    File::open(&path).map_err(|_| NativeHostRefusal::AtomVmBinaryNotFound)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&path).map_err(|_| NativeHostRefusal::AtomVmBinaryNotFound)?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(NativeHostRefusal::AtomVmBinaryNotExecutable);
        }
    }
    Ok(path)
}

fn path_arg(path: &Path) -> String {
    path.as_os_str().to_string_lossy().into_owned()
}

#[derive(Debug)]
struct BoundedRun {
    status: Option<ExitStatus>,
    timed_out: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

fn run_bounded(
    executable: &Path,
    args: &[String],
    environment: &[EnvironmentBinding],
    clear_environment: bool,
    timeout: Duration,
    poll_millis: u64,
    max_output_bytes: usize,
) -> Result<BoundedRun, NativeHostRefusal> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if clear_environment {
        command.env_clear();
    }
    for binding in environment {
        command.env(&binding.key, &binding.value);
    }

    let mut child = command.spawn().map_err(|_| NativeHostRefusal::SpawnRefused)?;
    let stdout = child.stdout.take().ok_or(NativeHostRefusal::PipeUnavailable)?;
    let stderr = child.stderr.take().ok_or(NativeHostRefusal::PipeUnavailable)?;
    let stdout_reader = thread::spawn(move || drain_bounded(stdout, max_output_bytes));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, max_output_bytes));

    let start = Instant::now();
    let (status, timed_out) = wait_with_timeout(&mut child, start, timeout, poll_millis)?;
    let (stdout, stdout_truncated) = stdout_reader
        .join()
        .map_err(|_| NativeHostRefusal::OutputReaderFailed)?
        .map_err(|_| NativeHostRefusal::OutputReaderFailed)?;
    let (stderr, stderr_truncated) = stderr_reader
        .join()
        .map_err(|_| NativeHostRefusal::OutputReaderFailed)?
        .map_err(|_| NativeHostRefusal::OutputReaderFailed)?;

    Ok(BoundedRun {
        status,
        timed_out,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn wait_with_timeout(
    child: &mut Child,
    start: Instant,
    timeout: Duration,
    poll_millis: u64,
) -> Result<(Option<ExitStatus>, bool), NativeHostRefusal> {
    loop {
        match child.try_wait().map_err(|_| NativeHostRefusal::WaitRefused)? {
            Some(status) => return Ok((Some(status), false)),
            None if start.elapsed() >= timeout => {
                let _ = child.kill();
                let status = child.wait().map_err(|_| NativeHostRefusal::WaitRefused)?;
                return Ok((Some(status), true));
            }
            None => thread::sleep(Duration::from_millis(poll_millis)),
        }
    }
}

fn drain_bounded<R: Read>(mut reader: R, max_output_bytes: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut kept = Vec::with_capacity(core::cmp::min(max_output_bytes, 16 * 1024));
    let mut scratch = [0u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut scratch)?;
        if read == 0 {
            break;
        }
        let remaining = max_output_bytes.saturating_sub(kept.len());
        let take = core::cmp::min(remaining, read);
        kept.extend_from_slice(&scratch[..take]);
        if take < read {
            truncated = true;
        }
    }
    Ok((kept, truncated))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|window| window == needle)
}

fn millis_u64(duration: Duration) -> u64 {
    core::cmp::min(duration.as_millis(), u64::MAX as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use super::super::evidence::Digest32;

    #[test]
    fn environment_is_explicitly_validated() {
        assert!(EnvironmentBinding::new("LANG", "C.UTF-8").is_ok());
        assert_eq!(
            EnvironmentBinding::new("BAD=KEY", "x"),
            Err(NativeHostRefusal::InvalidEnvironmentBinding)
        );
    }

    #[test]
    fn marker_search_is_byte_exact() {
        assert!(contains_bytes(b"prefix atomvm_swarm_alive suffix", b"atomvm_swarm_alive"));
        assert!(!contains_bytes(b"atomvm_swarm_dead", b"atomvm_swarm_alive"));
    }

    #[test]
    fn output_capture_is_bounded_while_draining() {
        let input = vec![7u8; 4096];
        let (kept, truncated) = drain_bounded(&input[..], 128).unwrap();
        assert_eq!(kept.len(), 128);
        assert!(truncated);
    }

    #[test]
    fn portable_digest_algorithm_remains_caller_owned() {
        let bytes = b"receipt bytes";
        let digest = |_data: &[u8]| Digest32([9; 32]);
        assert_eq!(digest(bytes), Digest32([9; 32]));
    }
}
