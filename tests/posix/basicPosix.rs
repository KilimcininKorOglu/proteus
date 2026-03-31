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
fn posix_info_reports_metadata_for_core_and_text_commands() {
    let output = Command::new(proteus_binary())
        .args(["--posix-info", "sort"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: sort"));
    assert!(stdout.contains("posix_level:"));
    assert!(stdout.contains("feature_flag: sort"));
}
