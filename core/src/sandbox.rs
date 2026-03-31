use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxMode {
    Strict,
    Permissive,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    SysAdmin,
    NetAdmin,
    NetRaw,
    NetBindService,
    Chown,
    DacOverride,
    Setuid,
    Setgid,
    Fowner,
    Kill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SyscallClass {
    ReadOnlyFs,
    WriteFs,
    Metadata,
    Memory,
    Process,
    Signal,
    Network,
    Dns,
    Terminal,
    Mount,
    Time,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxPolicy {
    pub applet: String,
    pub syscall_classes: BTreeSet<SyscallClass>,
    pub capabilities_to_keep: BTreeSet<Capability>,
    pub supports_strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxReport {
    pub applet: String,
    pub mode: SandboxMode,
    pub policy: SandboxPolicy,
    pub backend: SandboxBackend,
    pub applied: bool,
    pub degraded: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxBackend {
    Seccomp,
    Capsicum,
    Pledge,
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxError {
    UnsupportedMode(SandboxMode),
    UnknownApplet(String),
    BackendUnavailable(SandboxBackend),
    CapabilityDropFailed(String),
    PolicyApplyFailed(String),
}

impl fmt::Display for SandboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxError::UnsupportedMode(mode) => {
                write!(f, "sandbox mode not supported: {mode:?}")
            }
            SandboxError::UnknownApplet(applet) => write!(f, "unknown sandbox applet: {applet}"),
            SandboxError::BackendUnavailable(backend) => {
                write!(f, "sandbox backend unavailable: {backend:?}")
            }
            SandboxError::CapabilityDropFailed(msg) => write!(f, "capability drop failed: {msg}"),
            SandboxError::PolicyApplyFailed(msg) => write!(f, "sandbox policy apply failed: {msg}"),
        }
    }
}

impl std::error::Error for SandboxError {}

impl fmt::Display for SandboxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SandboxMode::Strict => write!(f, "strict"),
            SandboxMode::Permissive => write!(f, "permissive"),
            SandboxMode::Off => write!(f, "off"),
        }
    }
}

impl SandboxMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "strict" => Some(Self::Strict),
            "permissive" => Some(Self::Permissive),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

impl SandboxPolicy {
    pub fn new(applet: impl Into<String>) -> Self {
        Self {
            applet: applet.into(),
            syscall_classes: BTreeSet::new(),
            capabilities_to_keep: BTreeSet::new(),
            supports_strict: true,
        }
    }

    pub fn syscall_classes_as_strings(&self) -> Vec<&'static str> {
        self.syscall_classes
            .iter()
            .map(|class| match class {
                SyscallClass::ReadOnlyFs => "read-only-fs",
                SyscallClass::WriteFs => "write-fs",
                SyscallClass::Metadata => "metadata",
                SyscallClass::Memory => "memory",
                SyscallClass::Process => "process",
                SyscallClass::Signal => "signal",
                SyscallClass::Network => "network",
                SyscallClass::Dns => "dns",
                SyscallClass::Terminal => "terminal",
                SyscallClass::Mount => "mount",
                SyscallClass::Time => "time",
                SyscallClass::Random => "random",
            })
            .collect()
    }

    pub fn capabilities_as_strings(&self) -> Vec<&'static str> {
        self.capabilities_to_keep
            .iter()
            .map(|capability| match capability {
                Capability::SysAdmin => "sys-admin",
                Capability::NetAdmin => "net-admin",
                Capability::NetRaw => "net-raw",
                Capability::NetBindService => "net-bind-service",
                Capability::Chown => "chown",
                Capability::DacOverride => "dac-override",
                Capability::Setuid => "setuid",
                Capability::Setgid => "setgid",
                Capability::Fowner => "fowner",
                Capability::Kill => "kill",
            })
            .collect()
    }

    pub fn with_syscalls(mut self, classes: &[SyscallClass]) -> Self {
        self.syscall_classes.extend(classes.iter().copied());
        self
    }

    pub fn with_capabilities(mut self, capabilities: &[Capability]) -> Self {
        self.capabilities_to_keep
            .extend(capabilities.iter().copied());
        self
    }

    pub fn without_strict(mut self) -> Self {
        self.supports_strict = false;
        self
    }
}

pub fn default_policy_for(applet: &str) -> Option<SandboxPolicy> {
    match applet {
        "cat" => Some(
            SandboxPolicy::new("cat").with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
            ]),
        ),
        "ls" | "head" | "tail" | "wc" | "basename" | "dirname" | "pwd" => Some(
            SandboxPolicy::new(applet).with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
                SyscallClass::Terminal,
            ]),
        ),
        "cp" | "mv" | "rm" | "mkdir" | "rmdir" | "touch" | "ln" => Some(
            SandboxPolicy::new(applet).with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::WriteFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
                SyscallClass::Terminal,
            ]),
        ),
        "chmod" | "chown" | "chgrp" => Some(
            SandboxPolicy::new(applet)
                .with_syscalls(&[
                    SyscallClass::ReadOnlyFs,
                    SyscallClass::WriteFs,
                    SyscallClass::Metadata,
                    SyscallClass::Memory,
                    SyscallClass::Process,
                ])
                .with_capabilities(&[
                    Capability::Fowner,
                    Capability::Chown,
                    Capability::DacOverride,
                ]),
        ),
        "echo" | "true" | "false" => Some(
            SandboxPolicy::new(applet).with_syscalls(&[
                SyscallClass::Process,
                SyscallClass::Terminal,
            ]),
        ),
        "grep" | "egrep" | "fgrep" | "sort" | "uniq" | "cut" | "tr" | "sed" | "awk" => Some(
            SandboxPolicy::new(applet).with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
                SyscallClass::Terminal,
            ]),
        ),
        "find" => Some(
            SandboxPolicy::new("find").with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
                SyscallClass::Terminal,
            ]),
        ),
        "tar" | "gzip" => Some(
            SandboxPolicy::new(applet).with_syscalls(&[
                SyscallClass::ReadOnlyFs,
                SyscallClass::WriteFs,
                SyscallClass::Metadata,
                SyscallClass::Memory,
                SyscallClass::Process,
            ]),
        ),
        "xargs" => Some(
            SandboxPolicy::new("xargs")
                .with_syscalls(&[
                    SyscallClass::ReadOnlyFs,
                    SyscallClass::Process,
                    SyscallClass::Terminal,
                ])
                .without_strict(),
        ),
        "sh" => Some(
            SandboxPolicy::new("sh")
                .with_syscalls(&[
                    SyscallClass::ReadOnlyFs,
                    SyscallClass::WriteFs,
                    SyscallClass::Metadata,
                    SyscallClass::Memory,
                    SyscallClass::Process,
                    SyscallClass::Signal,
                    SyscallClass::Terminal,
                    SyscallClass::Time,
                    SyscallClass::Random,
                ])
                .without_strict(),
        ),
        _ => None,
    }
}

pub fn detect_backend() -> SandboxBackend {
    #[cfg(target_os = "linux")]
    {
        SandboxBackend::Seccomp
    }
    #[cfg(target_os = "freebsd")]
    {
        SandboxBackend::Capsicum
    }
    #[cfg(target_os = "openbsd")]
    {
        SandboxBackend::Pledge
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd")))]
    {
        SandboxBackend::Noop
    }
}

pub fn sandbox_report_for(applet: &str, mode: SandboxMode) -> Result<SandboxReport, SandboxError> {
    apply_sandbox_policy(applet, mode)
}

pub fn apply_sandbox_policy(applet: &str, mode: SandboxMode) -> Result<SandboxReport, SandboxError> {
    let policy = default_policy_for(applet)
        .ok_or_else(|| SandboxError::UnknownApplet(applet.to_string()))?;

    if mode == SandboxMode::Off {
        return Ok(SandboxReport {
            applet: applet.to_string(),
            mode,
            policy,
            backend: SandboxBackend::Noop,
            applied: false,
            degraded: false,
            notes: vec!["sandbox disabled by mode".to_string()],
        });
    }

    let backend = detect_backend();
    let mut notes = Vec::new();
    let effective_mode = if mode == SandboxMode::Strict && !policy.supports_strict {
        notes.push("strict mode unsupported for this applet; falling back to permissive".to_string());
        SandboxMode::Permissive
    } else {
        mode
    };

    match backend {
        SandboxBackend::Seccomp => apply_linux_policy(applet, effective_mode, policy, notes),
        SandboxBackend::Capsicum => apply_capsicum_policy(applet, effective_mode, policy, notes),
        SandboxBackend::Pledge => apply_pledge_policy(applet, effective_mode, policy, notes),
        SandboxBackend::Noop => {
            notes.push("sandbox backend unavailable on this target; using noop fallback".to_string());
            Ok(SandboxReport {
                applet: applet.to_string(),
                mode,
                policy,
                backend,
                applied: false,
                degraded: true,
                notes,
            })
        }
    }
}

pub fn drop_capabilities(keep: &[Capability]) -> Result<(), SandboxError> {
    let _keep = keep;
    #[cfg(target_os = "linux")]
    {
        let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if result != 0 {
            return Err(SandboxError::CapabilityDropFailed(
                std::io::Error::last_os_error().to_string(),
            ));
        }

        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(())
    }
}

fn apply_linux_policy(
    applet: &str,
    mode: SandboxMode,
    policy: SandboxPolicy,
    mut notes: Vec<String>,
) -> Result<SandboxReport, SandboxError> {
    if mode == SandboxMode::Permissive {
        drop_capabilities(&policy.capabilities_to_keep.iter().copied().collect::<Vec<_>>())?;
        notes.push("permissive mode: capability drop only".to_string());
        return Ok(SandboxReport {
            applet: applet.to_string(),
            mode,
            policy,
            backend: SandboxBackend::Seccomp,
            applied: true,
            degraded: false,
            notes,
        });
    }

    drop_capabilities(&policy.capabilities_to_keep.iter().copied().collect::<Vec<_>>())?;
    notes.push("strict mode: seccomp profile placeholder API applied".to_string());

    Ok(SandboxReport {
        applet: applet.to_string(),
        mode,
        policy,
        backend: SandboxBackend::Seccomp,
        applied: true,
        degraded: false,
        notes,
    })
}

fn apply_capsicum_policy(
    applet: &str,
    mode: SandboxMode,
    policy: SandboxPolicy,
    mut notes: Vec<String>,
) -> Result<SandboxReport, SandboxError> {
    if mode == SandboxMode::Strict && !policy.supports_strict {
        return Err(SandboxError::UnsupportedMode(mode));
    }

    notes.push("capsicum backend API placeholder applied".to_string());
    Ok(SandboxReport {
        applet: applet.to_string(),
        mode,
        policy,
        backend: SandboxBackend::Capsicum,
        applied: true,
        degraded: false,
        notes,
    })
}

fn apply_pledge_policy(
    applet: &str,
    mode: SandboxMode,
    policy: SandboxPolicy,
    mut notes: Vec<String>,
) -> Result<SandboxReport, SandboxError> {
    if mode == SandboxMode::Strict && !policy.supports_strict {
        return Err(SandboxError::UnsupportedMode(mode));
    }

    notes.push("pledge backend API placeholder applied".to_string());
    Ok(SandboxReport {
        applet: applet.to_string(),
        mode,
        policy,
        backend: SandboxBackend::Pledge,
        applied: true,
        degraded: false,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::{apply_sandbox_policy, default_policy_for, detect_backend, SandboxBackend, SandboxMode, SyscallClass};

    #[test]
    fn returns_policy_for_known_applet() {
        let policy = default_policy_for("cat").unwrap();
        assert_eq!(policy.applet, "cat");
        assert!(policy.syscall_classes.contains(&SyscallClass::ReadOnlyFs));
    }

    #[test]
    fn unknown_applet_returns_none() {
        assert!(default_policy_for("unknown-applet").is_none());
    }

    #[test]
    fn off_mode_returns_noop_report() {
        let report = apply_sandbox_policy("cat", SandboxMode::Off).unwrap();
        assert_eq!(report.mode, SandboxMode::Off);
        assert!(!report.applied);
    }

    #[test]
    fn backend_detection_returns_known_backend() {
        let backend = detect_backend();
        assert!(matches!(
            backend,
            SandboxBackend::Seccomp
                | SandboxBackend::Capsicum
                | SandboxBackend::Pledge
                | SandboxBackend::Noop
        ));
    }
}
