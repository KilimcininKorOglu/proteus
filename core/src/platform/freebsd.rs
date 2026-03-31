use super::{Platform, ProcessInfo};
use crate::error::{ProteusError, ProteusResult};

pub struct FreeBsdPlatform;

pub static FREEBSD_PLATFORM: FreeBsdPlatform = FreeBsdPlatform;

impl Platform for FreeBsdPlatform {
    fn name(&self) -> &'static str {
        "freebsd"
    }

    fn mount(&self, _source: &str, _target: &str, _fstype: &str, _flags: u64) -> ProteusResult<()> {
        Err(ProteusError::Other("mount is not implemented yet for FreeBSD".to_string()))
    }

    fn unmount(&self, _target: &str) -> ProteusResult<()> {
        Err(ProteusError::Other("unmount is not implemented yet for FreeBSD".to_string()))
    }

    fn sysctl(&self, name: &str) -> ProteusResult<String> {
        let output = std::process::Command::new("sysctl")
            .arg("-n")
            .arg(name)
            .output()
            .map_err(ProteusError::Io)?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(ProteusError::Other(String::from_utf8_lossy(&output.stderr).trim().to_string()))
        }
    }

    fn process_list(&self) -> ProteusResult<Vec<ProcessInfo>> {
        let output = std::process::Command::new("ps")
            .args(["-axo", "pid=,ppid=,state=,rss=,comm="])
            .output()
            .map_err(ProteusError::Io)?;

        if !output.status.success() {
            return Err(ProteusError::Other(String::from_utf8_lossy(&output.stderr).trim().to_string()));
        }

        let mut processes = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            processes.push(ProcessInfo {
                pid: parts[0].parse().unwrap_or(0),
                parent_pid: parts[1].parse().unwrap_or(0),
                state: parts[2].to_string(),
                memory_bytes: parts[3].parse::<u64>().unwrap_or(0) * 1024,
                command: parts[4..].join(" "),
            });
        }
        Ok(processes)
    }

    fn hostname(&self) -> ProteusResult<String> {
        let output = std::process::Command::new("hostname")
            .output()
            .map_err(ProteusError::Io)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(ProteusError::Other(String::from_utf8_lossy(&output.stderr).trim().to_string()))
        }
    }

    fn set_hostname(&self, hostname: &str) -> ProteusResult<()> {
        let status = std::process::Command::new("hostname")
            .arg(hostname)
            .status()
            .map_err(ProteusError::Io)?;
        if status.success() {
            Ok(())
        } else {
            Err(ProteusError::Other("failed to set hostname".to_string()))
        }
    }

    fn page_size(&self) -> usize {
        4096
    }
}
