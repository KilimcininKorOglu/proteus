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
fn list_command_includes_core_applets() {
    let output = Command::new(proteus_binary())
        .args(["--list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cat"));
    assert!(stdout.contains("grep"));
    assert!(stdout.contains("find"));
}
