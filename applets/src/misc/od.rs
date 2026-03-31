use std::io::{self, Read};

use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut bytes = Vec::new();
    if let Some(path) = args.first() {
        bytes = std::fs::read(path)?;
    } else {
        io::stdin().read_to_end(&mut bytes)?;
    }

    for (offset, chunk) in bytes.chunks(16).enumerate() {
        print!("{:07o} ", offset * 16);
        for byte in chunk {
            print!("{:03o} ", byte);
        }
        println!();
    }
    Ok(0)
}
