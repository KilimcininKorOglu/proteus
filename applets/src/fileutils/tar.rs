use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use proteus_core::ProteusResult;

use super::walk_paths;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut create = false;
    let mut extract = false;
    let mut archive_path: Option<String> = None;
    let mut operands = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-c" => create = true,
            "-x" => extract = true,
            "-f" => {
                let Some(path) = iter.next() else {
                    eprintln!("tar: option requires an argument -- 'f'");
                    return Ok(2);
                };
                archive_path = Some(path.clone());
            }
            value if value.starts_with('-') => {
                for ch in value[1..].chars() {
                    match ch {
                        'c' => create = true,
                        'x' => extract = true,
                        'f' => {
                            let Some(path) = iter.next() else {
                                eprintln!("tar: option requires an argument -- 'f'");
                                return Ok(2);
                            };
                            archive_path = Some(path.clone());
                        }
                        _ => {
                            eprintln!("tar: unsupported option -- '{ch}'");
                            return Ok(2);
                        }
                    }
                }
            }
            value => operands.push(value.to_string()),
        }
    }

    if create == extract {
        eprintln!("tar: specify exactly one of -c or -x");
        return Ok(2);
    }

    let Some(archive_path) = archive_path else {
        eprintln!("tar: missing archive path (-f)");
        return Ok(2);
    };

    if create {
        return create_archive(&archive_path, &operands);
    }

    extract_archive(&archive_path)
}

fn create_archive(archive_path: &str, operands: &[String]) -> ProteusResult<i32> {
    let paths = walk_paths(operands)?;
    let mut output = std::fs::File::create(archive_path)?;

    for path in paths {
        let metadata = std::fs::symlink_metadata(&path)?;
        if metadata.is_dir() {
            continue;
        }

        let path_string = path.to_string_lossy().trim_start_matches('/').to_string();
        let data = std::fs::read(&path)?;
        let header = build_header(&path_string, data.len() as u64, metadata.permissions().mode());
        output.write_all(&header)?;
        output.write_all(&data)?;
        write_padding(&mut output, data.len())?;
    }

    output.write_all(&[0u8; 1024])?;
    Ok(0)
}

fn extract_archive(archive_path: &str) -> ProteusResult<i32> {
    let bytes = std::fs::read(archive_path)?;
    let mut index = 0usize;

    while index + 512 <= bytes.len() {
        let header = &bytes[index..index + 512];
        index += 512;
        if header.iter().all(|byte| *byte == 0) {
            break;
        }

        let path = read_string(&header[0..100]);
        let size = read_octal(&header[124..136]) as usize;
        let typeflag = header[156];
        let path_buf = PathBuf::from(&path);

        if typeflag == b'5' {
            std::fs::create_dir_all(&path_buf)?;
            continue;
        }

        if index + size > bytes.len() {
            return Err("tar: truncated archive".into());
        }

        let data = &bytes[index..index + size];
        index += size;
        let padding = (512 - (size % 512)) % 512;
        index += padding;

        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path_buf, data)?;
    }

    Ok(0)
}

fn build_header(path: &str, size: u64, mode: u32) -> [u8; 512] {
    let mut header = [0u8; 512];
    write_bytes(&mut header[0..100], path.as_bytes());
    write_octal(&mut header[100..108], mode as u64, 7);
    write_octal(&mut header[108..116], 0, 7);
    write_octal(&mut header[116..124], 0, 7);
    write_octal(&mut header[124..136], size, 11);
    write_octal(&mut header[136..148], 0, 11);
    for byte in &mut header[148..156] {
        *byte = b' ';
    }
    header[156] = b'0';
    write_bytes(&mut header[257..263], b"ustar\0");
    write_bytes(&mut header[263..265], b"00");
    let checksum = header.iter().map(|byte| *byte as u32).sum::<u32>();
    write_octal(&mut header[148..156], checksum as u64, 6);
    header[154] = 0;
    header[155] = b' ';
    header
}

fn write_padding(output: &mut std::fs::File, len: usize) -> std::io::Result<()> {
    let padding = (512 - (len % 512)) % 512;
    if padding > 0 {
        output.write_all(&vec![0u8; padding])?;
    }
    Ok(())
}

fn write_bytes(target: &mut [u8], bytes: &[u8]) {
    let len = target.len().min(bytes.len());
    target[..len].copy_from_slice(&bytes[..len]);
}

fn write_octal(target: &mut [u8], value: u64, width: usize) {
    let formatted = format!("{:0width$o}", value, width = width);
    write_bytes(target, formatted.as_bytes());
}

fn read_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn read_octal(bytes: &[u8]) -> u64 {
    let value = read_string(bytes);
    u64::from_str_radix(value.trim(), 8).unwrap_or(0)
}
