use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn unique_temp_dir(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("proteus-{name}-{stamp}"));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn printf_and_seq_produce_expected_output() {
    let printf_output = Command::new(proteus_binary())
        .args(["printf", "%s %d %b", "value", "7", "line\\n"])
        .output()
        .unwrap();
    assert!(printf_output.status.success());
    assert_eq!(String::from_utf8_lossy(&printf_output.stdout), "value 7 line\n");

    let seq_output = Command::new(proteus_binary())
        .args(["seq", "2", "4"])
        .output()
        .unwrap();
    assert!(seq_output.status.success());
    assert_eq!(String::from_utf8_lossy(&seq_output.stdout), "2\n3\n4\n");
}

#[test]
fn tee_and_od_process_real_files() {
    let dir = unique_temp_dir("coreutils");
    let output_file = dir.join("tee.txt");

    let tee_output = Command::new(proteus_binary())
        .arg("tee")
        .arg(output_file.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(b"alpha\n")?;
            child.wait_with_output()
        })
        .unwrap();
    assert!(tee_output.status.success());
    assert_eq!(String::from_utf8_lossy(&tee_output.stdout), "alpha\n");
    assert_eq!(fs::read_to_string(&output_file).unwrap(), "alpha\n");

    let od_output = Command::new(proteus_binary())
        .args(["od", output_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(od_output.status.success());
    assert!(String::from_utf8_lossy(&od_output.stdout).contains("141"));
}

#[test]
fn env_uname_and_identity_commands_run() {
    let env_output = Command::new(proteus_binary())
        .args(["env", "TEST_KEY=VALUE", "sh", "-c", "echo $TEST_KEY"])
        .output()
        .unwrap();
    assert!(env_output.status.success());
    assert!(String::from_utf8_lossy(&env_output.stdout).contains("VALUE"));

    let uname_output = Command::new(proteus_binary())
        .args(["uname"])
        .output()
        .unwrap();
    assert!(uname_output.status.success());
    assert!(!String::from_utf8_lossy(&uname_output.stdout).trim().is_empty());

    let id_output = Command::new(proteus_binary()).args(["id"]).output().unwrap();
    assert!(id_output.status.success());
    assert!(String::from_utf8_lossy(&id_output.stdout).contains("uid="));

    let whoami_output = Command::new(proteus_binary()).args(["whoami"]).output().unwrap();
    assert!(whoami_output.status.success());
    assert!(!String::from_utf8_lossy(&whoami_output.stdout).trim().is_empty());

    let groups_output = Command::new(proteus_binary()).args(["groups"]).output().unwrap();
    assert!(groups_output.status.success());
    assert!(!String::from_utf8_lossy(&groups_output.stdout).trim().is_empty());
}
