use proteus_core::platform::current_platform;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let all = args.iter().any(|arg| arg == "-a");
    let platform = current_platform();
    let sysname = platform.name();
    let nodename = platform.hostname().unwrap_or_else(|_| "unknown".to_string());
    let release = std::env::consts::OS;
    let machine = std::env::consts::ARCH;

    if all {
        println!("{sysname} {nodename} {release} {machine}");
    } else {
        println!("{sysname}");
    }
    Ok(0)
}
