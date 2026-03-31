use proteus_core::ProteusResult;

pub fn run(_args: &[String]) -> ProteusResult<i32> {
    if let Ok(user) = std::env::var("USER") {
        println!("{user}");
        return Ok(0);
    }

    let uid = unsafe { libc::geteuid() };
    println!("{uid}");
    Ok(0)
}
