use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let (start, end) = match args.len() {
        1 => (1i64, args[0].parse::<i64>().unwrap_or(1)),
        2 => (
            args[0].parse::<i64>().unwrap_or(1),
            args[1].parse::<i64>().unwrap_or(1),
        ),
        _ => {
            eprintln!("seq: expected END or START END");
            return Ok(2);
        }
    };

    if start <= end {
        for value in start..=end {
            println!("{value}");
        }
    } else {
        for value in (end..=start).rev() {
            println!("{value}");
        }
    }
    Ok(0)
}
