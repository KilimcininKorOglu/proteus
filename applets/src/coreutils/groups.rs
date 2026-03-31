use proteus_core::ProteusResult;

pub fn run(_args: &[String]) -> ProteusResult<i32> {
    let gid = unsafe { libc::getegid() };
    println!("{gid}");
    Ok(0)
}
