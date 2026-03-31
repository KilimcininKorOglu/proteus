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
fn sort_and_uniq_process_real_files() {
    let dir = unique_temp_dir("text");
    let input = dir.join("lines.txt");
    fs::write(&input, "beta\nalpha\nbeta\n").unwrap();

    let sort_output = Command::new(proteus_binary())
        .args(["sort", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(sort_output.status.success());
    assert_eq!(String::from_utf8_lossy(&sort_output.stdout), "alpha\nbeta\nbeta\n");

    let uniq_output = Command::new(proteus_binary())
        .args(["uniq", "-c", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(uniq_output.status.success());
    assert!(String::from_utf8_lossy(&uniq_output.stdout).contains("beta"));
}

#[test]
fn find_and_awk_cover_v03_batch_paths() {
    let dir = unique_temp_dir("fileutils");
    let nested = dir.join("sub");
    fs::create_dir_all(&nested).unwrap();
    let file = nested.join("data.txt");
    fs::write(&file, "alpha one\nbeta two\n").unwrap();

    let find_output = Command::new(proteus_binary())
        .args(["find", dir.to_str().unwrap(), "-name", "*txt"])
        .output()
        .unwrap();
    assert!(find_output.status.success());
    assert!(String::from_utf8_lossy(&find_output.stdout).contains("data.txt"));

    let awk_output = Command::new(proteus_binary())
        .args(["awk", "{print $2}", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(awk_output.status.success());
    assert_eq!(String::from_utf8_lossy(&awk_output.stdout), "one\ntwo\n");
}

#[test]
fn gzip_round_trip_preserves_contents() {
    let dir = unique_temp_dir("gzip");
    let input = dir.join("alpha.txt");
    fs::write(&input, "alpha one\nbeta two\n").unwrap();

    let compressed = Command::new(proteus_binary())
        .args(["gzip", "-c", input.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(compressed.status.success());
    let archive = dir.join("alpha.txt.gz");
    fs::write(&archive, &compressed.stdout).unwrap();

    let decompressed = Command::new(proteus_binary())
        .args(["gzip", "-d", "-c", archive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(decompressed.status.success());
    assert_eq!(String::from_utf8_lossy(&decompressed.stdout), "alpha one\nbeta two\n");
}

#[test]
fn tar_create_and_extract_round_trip() {
    let source = unique_temp_dir("tar-src");
    let extract = unique_temp_dir("tar-out");
    let nested = source.join("sub");
    fs::create_dir_all(&nested).unwrap();
    let alpha = source.join("alpha.txt");
    let nested_file = nested.join("nested.txt");
    fs::write(&alpha, "alpha one\n").unwrap();
    fs::write(&nested_file, "nested data\n").unwrap();
    let archive = source.join("archive.tar");

    let create = Command::new(proteus_binary())
        .args([
            "tar",
            "-cf",
            archive.to_str().unwrap(),
            alpha.to_str().unwrap(),
            nested_file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(create.status.success());

    let extract_status = Command::new(proteus_binary())
        .current_dir(&extract)
        .args(["tar", "-xf", archive.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(extract_status.status.success());

    let extracted_alpha = extract.join(alpha.strip_prefix("/").unwrap_or(&alpha));
    let extracted_nested = extract.join(nested_file.strip_prefix("/").unwrap_or(&nested_file));
    assert_eq!(fs::read_to_string(extracted_alpha).unwrap(), "alpha one\n");
    assert_eq!(fs::read_to_string(extracted_nested).unwrap(), "nested data\n");
}
