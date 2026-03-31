use std::ffi::{CStr, CString};

use super::{Platform, ProcessInfo};
use crate::error::{ProteusError, ProteusResult};

pub struct LinuxPlatform;

pub static LINUX_PLATFORM: LinuxPlatform = LinuxPlatform;

impl Platform for LinuxPlatform {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn mount(&self, source: &str, target: &str, fstype: &str, flags: u64) -> ProteusResult<()> {
        let source = CString::new(source).map_err(|_| ProteusError::Other("invalid source path".to_string()))?;
        let target = CString::new(target).map_err(|_| ProteusError::Other("invalid target path".to_string()))?;
        let fstype = CString::new(fstype).map_err(|_| ProteusError::Other("invalid filesystem type".to_string()))?;

        let result = unsafe {
            libc::mount(
                source.as_ptr(),
                target.as_ptr(),
                fstype.as_ptr(),
                flags as libc::c_ulong,
                std::ptr::null(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            Err(ProteusError::Io(std::io::Error::last_os_error()))
        }
    }

    fn unmount(&self, target: &str) -> ProteusResult<()> {
        let target = CString::new(target).map_err(|_| ProteusError::Other("invalid target path".to_string()))?;
        let result = unsafe { libc::umount(target.as_ptr()) };
        if result == 0 {
            Ok(())
        } else {
            Err(ProteusError::Io(std::io::Error::last_os_error()))
        }
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
            .args(["-eo", "pid=,ppid=,state=,rss=,comm="])
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
            let pid = parts[0].parse().unwrap_or(0);
            let parent_pid = parts[1].parse().unwrap_or(0);
            let state = parts[2].to_string();
            let memory_kib: u64 = parts[3].parse().unwrap_or(0);
            let command = parts[4..].join(" ");
            processes.push(ProcessInfo {
                pid,
                parent_pid,
                command,
                state,
                memory_bytes: memory_kib * 1024,
            });
        }

        Ok(processes)
    }

    fn hostname(&self) -> ProteusResult<String> {
        let mut buf = [0u8; 256];
        let result = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) };
        if result == 0 {
            let cstr = unsafe { CStr::from_ptr(buf.as_ptr().cast()) };
            Ok(cstr.to_string_lossy().into_owned())
        } else {
            Err(ProteusError::Io(std::io::Error::last_os_error()))
        }
    }

    fn set_hostname(&self, hostname: &str) -> ProteusResult<()> {
        let hostname = CString::new(hostname).map_err(|_| ProteusError::Other("invalid hostname".to_string()))?;
        let result = unsafe { libc::sethostname(hostname.as_ptr(), hostname.as_bytes().len()) };
        if result == 0 {
            Ok(())
        } else {
            Err(ProteusError::Io(std::io::Error::last_os_error()))
        }
    }

    fn page_size(&self) -> usize {
        let result = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if result > 0 { result as usize } else { 4096 }
    }
}
