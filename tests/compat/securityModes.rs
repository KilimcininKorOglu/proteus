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
fn help_mentions_sandbox_info_command() {
    let output = Command::new(proteus_binary())
        .args(["--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("--sandbox-info <applet>"));
}
