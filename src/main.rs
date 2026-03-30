use proteus_core::ProteusResult;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let argv0 = args
        .first()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "proteus".into());

    let exit_code = if argv0 == "proteus" {
        dispatch_multi_call(&args[1..])
    } else {
        dispatch_applet(&argv0, &args[1..])
    };

    std::process::exit(exit_code);
}

fn dispatch_multi_call(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("proteus: no applet specified");
        eprintln!("Try 'proteus --list' for a list of available applets.");
        return 1;
    }

    match args[0].as_str() {
        "--list" => {
            list_applets();
            0
        }
        "--version" | "-V" => {
            println!("proteus {VERSION}");
            0
        }
        "--help" | "-h" => {
            print_help();
            0
        }
        applet_name => {
            let applet_args = &args[1..];
            dispatch_applet(applet_name, applet_args)
        }
    }
}

fn dispatch_applet(name: &str, args: &[String]) -> i32 {
    match dispatch(name, args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("proteus: {name}: {e}");
            1
        }
    }
}

fn dispatch(name: &str, args: &[String]) -> ProteusResult<i32> {
    match name {
        // Coreutils
        #[cfg(feature = "cat")]
        "cat" => proteus_applets::coreutils::run_cat(args),
        #[cfg(feature = "ls")]
        "ls" => proteus_applets::coreutils::run_ls(args),
        #[cfg(feature = "cp")]
        "cp" => proteus_applets::coreutils::run_cp(args),
        #[cfg(feature = "mv")]
        "mv" => proteus_applets::coreutils::run_mv(args),
        #[cfg(feature = "rm")]
        "rm" => proteus_applets::coreutils::run_rm(args),
        #[cfg(feature = "echo")]
        "echo" => proteus_applets::coreutils::run_echo(args),
        #[cfg(feature = "head")]
        "head" => proteus_applets::coreutils::run_head(args),
        #[cfg(feature = "tail")]
        "tail" => proteus_applets::coreutils::run_tail(args),
        #[cfg(feature = "wc")]
        "wc" => proteus_applets::coreutils::run_wc(args),
        #[cfg(feature = "pwd")]
        "pwd" => proteus_applets::coreutils::run_pwd(args),
        #[cfg(feature = "mkdir")]
        "mkdir" => proteus_applets::coreutils::run_mkdir(args),
        #[cfg(feature = "rmdir")]
        "rmdir" => proteus_applets::coreutils::run_rmdir(args),
        #[cfg(feature = "touch")]
        "touch" => proteus_applets::coreutils::run_touch(args),
        #[cfg(feature = "chmod")]
        "chmod" => proteus_applets::coreutils::run_chmod(args),
        #[cfg(feature = "chown")]
        "chown" => proteus_applets::coreutils::run_chown(args),
        #[cfg(feature = "chgrp")]
        "chgrp" => proteus_applets::coreutils::run_chgrp(args),
        #[cfg(feature = "ln")]
        "ln" => proteus_applets::coreutils::run_ln(args),
        #[cfg(feature = "basename")]
        "basename" => proteus_applets::coreutils::run_basename(args),
        #[cfg(feature = "dirname")]
        "dirname" => proteus_applets::coreutils::run_dirname(args),
        #[cfg(feature = "true")]
        "true" => proteus_applets::coreutils::run_true(args),
        #[cfg(feature = "false")]
        "false" => proteus_applets::coreutils::run_false(args),

        // Shell
        #[cfg(feature = "sh")]
        "sh" => proteus_shell::run_shell(args),

        _ => {
            eprintln!("proteus: applet '{name}' not found");
            eprintln!("Try 'proteus --list' for a list of available applets.");
            Ok(127)
        }
    }
}

fn list_applets() {
    let mut applets: Vec<&str> = Vec::new();

    #[cfg(feature = "cat")]
    applets.push("cat");
    #[cfg(feature = "ls")]
    applets.push("ls");
    #[cfg(feature = "cp")]
    applets.push("cp");
    #[cfg(feature = "mv")]
    applets.push("mv");
    #[cfg(feature = "rm")]
    applets.push("rm");
    #[cfg(feature = "echo")]
    applets.push("echo");
    #[cfg(feature = "head")]
    applets.push("head");
    #[cfg(feature = "tail")]
    applets.push("tail");
    #[cfg(feature = "wc")]
    applets.push("wc");
    #[cfg(feature = "pwd")]
    applets.push("pwd");
    #[cfg(feature = "mkdir")]
    applets.push("mkdir");
    #[cfg(feature = "rmdir")]
    applets.push("rmdir");
    #[cfg(feature = "touch")]
    applets.push("touch");
    #[cfg(feature = "chmod")]
    applets.push("chmod");
    #[cfg(feature = "chown")]
    applets.push("chown");
    #[cfg(feature = "chgrp")]
    applets.push("chgrp");
    #[cfg(feature = "ln")]
    applets.push("ln");
    #[cfg(feature = "basename")]
    applets.push("basename");
    #[cfg(feature = "dirname")]
    applets.push("dirname");
    #[cfg(feature = "true")]
    applets.push("true");
    #[cfg(feature = "false")]
    applets.push("false");
    #[cfg(feature = "sh")]
    applets.push("sh");

    applets.sort();
    for applet in &applets {
        println!("{applet}");
    }
}

fn print_help() {
    println!("proteus {VERSION}");
    println!("Shape-shifting Unix toolkit — one binary, every tool.");
    println!();
    println!("USAGE:");
    println!("    proteus <applet> [args...]");
    println!("    proteus --list");
    println!("    proteus --version");
    println!("    proteus --help");
    println!();
    println!("Applets are also available via symlinks:");
    println!("    ln -s proteus cat");
    println!("    ./cat file.txt");
}
