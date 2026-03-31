use std::io::Write;

use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;
use proteus_core::ProteusResult;

use super::read_all_bytes;

const GZIP_HEADER: [u8; 10] = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff];

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut to_stdout = false;
    let mut decompress = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-c" => to_stdout = true,
            "-d" => decompress = true,
            value if value.starts_with('-') => {
                eprintln!("gzip: unsupported option -- '{value}'");
                return Ok(2);
            }
            value => files.push(value.to_string()),
        }
    }

    let bytes = read_all_bytes(&files)?;
    let output = if decompress {
        decode_gzip(&bytes)?
    } else {
        encode_gzip(&bytes)
    };

    if to_stdout || files.is_empty() {
        std::io::stdout().write_all(&output)?;
        return Ok(0);
    }

    let input_name = &files[0];
    let output_path = if decompress && input_name.ends_with(".gz") {
        input_name.trim_end_matches(".gz").to_string()
    } else if decompress {
        format!("{input_name}.out")
    } else {
        format!("{input_name}.gz")
    };

    std::fs::write(output_path, output)?;
    Ok(0)
}

fn encode_gzip(input: &[u8]) -> Vec<u8> {
    let compressed = compress_to_vec(input, 6);
    let mut output = Vec::with_capacity(GZIP_HEADER.len() + compressed.len() + 8);
    output.extend_from_slice(&GZIP_HEADER);
    output.extend_from_slice(&compressed);
    output.extend_from_slice(&crc32(input).to_le_bytes());
    output.extend_from_slice(&(input.len() as u32).to_le_bytes());
    output
}

fn decode_gzip(input: &[u8]) -> ProteusResult<Vec<u8>> {
    if input.len() < 18 || !input.starts_with(&GZIP_HEADER[..3]) {
        return Err("gzip: invalid gzip stream".into());
    }

    let compressed_end = input.len().saturating_sub(8);
    let compressed = &input[10..compressed_end];
    let decompressed = decompress_to_vec(compressed)
        .map_err(|error| format!("gzip: failed to decompress stream: {error:?}"))?;

    let expected_crc = u32::from_le_bytes(input[compressed_end..compressed_end + 4].try_into().unwrap());
    let expected_size = u32::from_le_bytes(input[compressed_end + 4..compressed_end + 8].try_into().unwrap());

    if crc32(&decompressed) != expected_crc {
        return Err("gzip: crc mismatch".into());
    }
    if decompressed.len() as u32 != expected_size {
        return Err("gzip: size mismatch".into());
    }

    Ok(decompressed)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
