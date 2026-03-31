use proteus_core::ProteusResult;

pub fn run(_args: &[String]) -> ProteusResult<i32> {
    let status = std::process::Command::new("date").status()?;
    Ok(status.code().unwrap_or(1))
}
