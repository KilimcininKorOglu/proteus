use proteus_core::ProteusResult;

pub fn run(_args: &[String]) -> ProteusResult<i32> {
    let uid = unsafe { libc::geteuid() };
    let gid = unsafe { libc::getegid() };
    println!("uid={uid} gid={gid}");
    Ok(0)
}
