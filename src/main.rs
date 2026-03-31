use std::path::{Path, PathBuf};

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
        "--install" => install_applets(&args[1..]),
        "--uninstall" => uninstall_applets(&args[1..]),
        applet_name => {
            let applet_args = &args[1..];
            dispatch_applet(applet_name, applet_args)
        }
    }
}

fn dispatch_applet(name: &str, args: &[String]) -> i32 {
    match dispatch(name, args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("proteus: {name}: {error}");
            1
        }
    }
}

fn dispatch(name: &str, args: &[String]) -> ProteusResult<i32> {
    match name {
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
    for applet in available_applets() {
        println!("{applet}");
    }
}

fn available_applets() -> Vec<&'static str> {
    let mut applets: Vec<&'static str> = Vec::new();

    #[cfg(feature = "basename")]
    applets.push("basename");
    #[cfg(feature = "cat")]
    applets.push("cat");
    #[cfg(feature = "chgrp")]
    applets.push("chgrp");
    #[cfg(feature = "chmod")]
    applets.push("chmod");
    #[cfg(feature = "chown")]
    applets.push("chown");
    #[cfg(feature = "cp")]
    applets.push("cp");
    #[cfg(feature = "dirname")]
    applets.push("dirname");
    #[cfg(feature = "echo")]
    applets.push("echo");
    #[cfg(feature = "false")]
    applets.push("false");
    #[cfg(feature = "head")]
    applets.push("head");
    #[cfg(feature = "ln")]
    applets.push("ln");
    #[cfg(feature = "ls")]
    applets.push("ls");
    #[cfg(feature = "mkdir")]
    applets.push("mkdir");
    #[cfg(feature = "mv")]
    applets.push("mv");
    #[cfg(feature = "pwd")]
    applets.push("pwd");
    #[cfg(feature = "rm")]
    applets.push("rm");
    #[cfg(feature = "rmdir")]
    applets.push("rmdir");
    #[cfg(feature = "sh")]
    applets.push("sh");
    #[cfg(feature = "tail")]
    applets.push("tail");
    #[cfg(feature = "touch")]
    applets.push("touch");
    #[cfg(feature = "true")]
    applets.push("true");
    #[cfg(feature = "wc")]
    applets.push("wc");

    applets.sort_unstable();
    applets
}

fn install_applets(args: &[String]) -> i32 {
    let mut symlink_mode = false;
    let mut force = false;
    let mut target_dir: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "-s" | "--symlink" => symlink_mode = true,
            "-f" | "--force" => force = true,
            value if value.starts_with('-') => {
                eprintln!("proteus: --install: invalid option '{value}'");
                return 2;
            }
            value => target_dir = Some(value),
        }
    }

    let Some(target_dir) = target_dir else {
        eprintln!("proteus: --install: missing target directory");
        return 2;
    };

    let target_path = Path::new(target_dir);
    if !target_path.exists() {
        eprintln!("proteus: --install: directory does not exist: {target_dir}");
        return 1;
    }
    if !target_path.is_dir() {
        eprintln!("proteus: --install: target is not a directory: {target_dir}");
        return 1;
    }

    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("proteus: --install: unable to locate current executable: {error}");
            return 1;
        }
    };

    let mut had_error = false;
    for applet in available_applets() {
        let link_path = target_path.join(applet);
        if link_path.exists() || link_path.is_symlink() {
            if force {
                if let Err(error) = std::fs::remove_file(&link_path) {
                    eprintln!("proteus: --install: failed to remove {}: {error}", link_path.display());
                    had_error = true;
                    continue;
                }
            } else {
                eprintln!("proteus: --install: target exists, use --force: {}", link_path.display());
                had_error = true;
                continue;
            }
        }

        let result = if symlink_mode {
            std::os::unix::fs::symlink(&exe_path, &link_path)
        } else {
            std::fs::hard_link(&exe_path, &link_path)
        };

        if let Err(error) = result {
            eprintln!("proteus: --install: failed to create {}: {error}", link_path.display());
            had_error = true;
        }
    }

    if had_error { 1 } else { 0 }
}

fn uninstall_applets(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("proteus: --uninstall: missing target directory");
        return 2;
    }

    let target_path = PathBuf::from(&args[0]);
    if !target_path.exists() {
        eprintln!("proteus: --uninstall: directory does not exist: {}", target_path.display());
        return 1;
    }
    if !target_path.is_dir() {
        eprintln!("proteus: --uninstall: target is not a directory: {}", target_path.display());
        return 1;
    }

    let mut had_error = false;
    for applet in available_applets() {
        let link_path = target_path.join(applet);
        if link_path.exists() || link_path.is_symlink() {
            if let Err(error) = std::fs::remove_file(&link_path) {
                eprintln!("proteus: --uninstall: failed to remove {}: {error}", link_path.display());
                had_error = true;
            }
        }
    }

    if had_error { 1 } else { 0 }
}

fn print_help() {
    println!("proteus {VERSION}");
    println!("Shape-shifting Unix toolkit — one binary, every tool.");
    println!();
    println!("USAGE:");
    println!("    proteus <applet> [args...]");
    println!("    proteus --list");
    println!("    proteus --install [-s|--symlink] [-f|--force] <directory>");
    println!("    proteus --uninstall <directory>");
    println!("    proteus --version");
    println!("    proteus --help");
}
