use crate::error::ProteusResult;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "freebsd")]
pub mod freebsd;
#[cfg(target_os = "macos")]
pub mod macos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub command: String,
    pub state: String,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountInfo {
    pub source: String,
    pub target: String,
    pub fstype: String,
    pub flags: u64,
}

pub trait Platform {
    fn name(&self) -> &'static str;
    fn mount(&self, source: &str, target: &str, fstype: &str, flags: u64) -> ProteusResult<()>;
    fn unmount(&self, target: &str) -> ProteusResult<()>;
    fn sysctl(&self, name: &str) -> ProteusResult<String>;
    fn process_list(&self) -> ProteusResult<Vec<ProcessInfo>>;
    fn hostname(&self) -> ProteusResult<String>;
    fn set_hostname(&self, hostname: &str) -> ProteusResult<()>;
    fn page_size(&self) -> usize;
}

pub fn current_platform() -> &'static dyn Platform {
    #[cfg(target_os = "linux")]
    {
        &linux::LINUX_PLATFORM
    }
    #[cfg(target_os = "freebsd")]
    {
        &freebsd::FREEBSD_PLATFORM
    }
    #[cfg(target_os = "macos")]
    {
        &macos::MACOS_PLATFORM
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
    {
        &unsupported::UNSUPPORTED_PLATFORM
    }
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos")))]
mod unsupported {
    use super::{Platform, ProcessInfo};
    use crate::error::{ProteusError, ProteusResult};

    pub struct UnsupportedPlatform;

    pub static UNSUPPORTED_PLATFORM: UnsupportedPlatform = UnsupportedPlatform;

    impl Platform for UnsupportedPlatform {
        fn name(&self) -> &'static str {
            "unsupported"
        }

        fn mount(&self, _source: &str, _target: &str, _fstype: &str, _flags: u64) -> ProteusResult<()> {
            Err(ProteusError::Other("mount is unsupported on this platform".to_string()))
        }

        fn unmount(&self, _target: &str) -> ProteusResult<()> {
            Err(ProteusError::Other("unmount is unsupported on this platform".to_string()))
        }

        fn sysctl(&self, _name: &str) -> ProteusResult<String> {
            Err(ProteusError::Other("sysctl is unsupported on this platform".to_string()))
        }

        fn process_list(&self) -> ProteusResult<Vec<ProcessInfo>> {
            Err(ProteusError::Other("process listing is unsupported on this platform".to_string()))
        }

        fn hostname(&self) -> ProteusResult<String> {
            Err(ProteusError::Other("hostname is unsupported on this platform".to_string()))
        }

        fn set_hostname(&self, _hostname: &str) -> ProteusResult<()> {
            Err(ProteusError::Other("set_hostname is unsupported on this platform".to_string()))
        }

        fn page_size(&self) -> usize {
            4096
        }
    }
}
