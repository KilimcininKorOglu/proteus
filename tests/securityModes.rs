use std::path::PathBuf;
use std::process::Command;

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

#[test]
fn sandbox_info_reports_known_policy() {
    let output = Command::new(proteus_binary())
        .args(["--sandbox-info", "cat"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("applet: cat"));
    assert!(stdout.contains("mode: strict"));
    assert!(stdout.contains("syscalls:"));
}

#[test]
fn sandbox_info_reports_strict_request_and_fallback_note_for_xargs() {
    let output = Command::new(proteus_binary())
        .args(["--sandbox-info", "xargs"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("applet: xargs"));
    assert!(stdout.contains("mode: strict"));
    assert!(stdout.contains("supports_strict: no"));
    assert!(stdout.contains("falling back to permissive"));
}

#[test]
fn sandbox_info_rejects_unknown_applet() {
    let output = Command::new(proteus_binary())
        .args(["--sandbox-info", "unknown-applet"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown sandbox applet"));
}
