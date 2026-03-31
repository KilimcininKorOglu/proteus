use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::PathBuf};

fn proteus_binary() -> String {
    std::env::var("CARGO_BIN_EXE_proteus").unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("proteus")
            .display()
            .to_string()
    })
}

fn unique_temp_file(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("proteus-{name}-{stamp}"))
}

#[test]
fn shell_conditionals_follow_exit_status() {
    let output = Command::new(proteus_binary())
        .args(["sh", "-c", "false && echo no; true && echo yes; false || echo fallback"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "yes\nfallback\n");
}

#[test]
fn shell_redirection_and_pipeline_work() {
    let file = unique_temp_file("shell-out");
    let script = format!("echo hello > {} && cat < {}", file.display(), file.display());

    let output = Command::new(proteus_binary())
        .args(["sh", "-c", &script])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}

#[test]
fn shell_expands_args_and_last_exit() {
    let output = Command::new(proteus_binary())
        .args(["sh", "-c", "echo arg:$1 status:$?", "--", "demo"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "arg:demo status:0\n");
}

#[test]
fn shell_pipeline_regression_stays_working() {
    let output = Command::new(proteus_binary())
        .args(["sh", "-c", "echo first second | grep second"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "first second\n");
}

#[test]
fn shell_redirection_creates_real_file() {
    let file = unique_temp_file("shell-file");
    let script = format!("echo saved > {}", file.display());

    let output = Command::new(proteus_binary())
        .args(["sh", "-c", &script])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(fs::read_to_string(file).unwrap(), "saved\n");
}
