use proteus_core::ProteusResult;

pub fn run_shell(args: &[String]) -> ProteusResult<i32> {
    let _ = args;
    eprintln!("proteus: shell not yet implemented");
    Ok(1)
}
