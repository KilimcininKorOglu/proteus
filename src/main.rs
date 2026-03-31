use std::path::{Path, PathBuf};

use proteus_core::compliance::{AppletCategory, AppletMetadata, PosixLevel};
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
        "--list-full" => {
            list_applets_full();
            0
        }
        "--posix-info" => print_posix_info(&args[1..]),
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
        #[cfg(feature = "grep")]
        "grep" => proteus_applets::textutils::grep::run(args),
        #[cfg(feature = "egrep")]
        "egrep" => proteus_applets::textutils::egrep::run(args),
        #[cfg(feature = "fgrep")]
        "fgrep" => proteus_applets::textutils::fgrep::run(args),
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

fn list_applets_full() {
    for metadata in available_applet_metadata() {
        println!(
            "{:<12} {:<16} {:<12} {}",
            metadata.name,
            metadata.category.as_str(),
            metadata.posix_level.as_str(),
            metadata.description
        );
    }
}

fn print_posix_info(args: &[String]) -> i32 {
    let Some(applet_name) = args.first() else {
        eprintln!("proteus: --posix-info: missing applet name");
        return 2;
    };

    match applet_metadata(applet_name) {
        Some(metadata) => {
            for line in metadata.to_report_lines() {
                println!("{line}");
            }
            0
        }
        None => {
            eprintln!("proteus: --posix-info: unknown applet '{applet_name}'");
            1
        }
    }
}

fn available_applets() -> Vec<&'static str> {
    available_applet_metadata()
        .iter()
        .map(|metadata| metadata.name)
        .collect()
}

fn applet_metadata(name: &str) -> Option<AppletMetadata> {
    available_applet_metadata()
        .into_iter()
        .find(|metadata| metadata.name == name)
}

fn available_applet_metadata() -> Vec<AppletMetadata> {
    let mut applets: Vec<AppletMetadata> = Vec::new();

    #[cfg(feature = "basename")]
    applets.push(AppletMetadata::new(
        "basename",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Strip directory components from a path",
        "basename",
        true,
    ));
    #[cfg(feature = "cat")]
    applets.push(AppletMetadata::new(
        "cat",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Concatenate files to standard output",
        "cat",
        true,
    ));
    #[cfg(feature = "chgrp")]
    applets.push(AppletMetadata::new(
        "chgrp",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "Change file group ownership",
        "chgrp",
        true,
    ));
    #[cfg(feature = "chmod")]
    applets.push(AppletMetadata::new(
        "chmod",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "Change file mode bits",
        "chmod",
        true,
    ));
    #[cfg(feature = "chown")]
    applets.push(AppletMetadata::new(
        "chown",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "Change file owner and group",
        "chown",
        true,
    ));
    #[cfg(feature = "cp")]
    applets.push(AppletMetadata::new(
        "cp",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "Copy files and directories",
        "cp",
        true,
    ));
    #[cfg(feature = "dirname")]
    applets.push(AppletMetadata::new(
        "dirname",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Strip last path component",
        "dirname",
        true,
    ));
    #[cfg(feature = "echo")]
    applets.push(AppletMetadata::new(
        "echo",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "Write arguments to standard output",
        "echo",
        true,
    ));
    #[cfg(feature = "false")]
    applets.push(AppletMetadata::new(
        "false",
        AppletCategory::Coreutils,
        PosixLevel::Full,
        "Return a non-zero exit status",
        "false",
        true,
    ));
    #[cfg(feature = "egrep")]
    applets.push(AppletMetadata::new(
        "egrep",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Search files using extended regular expressions",
        "egrep",
        true,
    ));
    #[cfg(feature = "fgrep")]
    applets.push(AppletMetadata::new(
        "fgrep",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Search files using fixed-string patterns",
        "fgrep",
        true,
    ));
    #[cfg(feature = "grep")]
    applets.push(AppletMetadata::new(
        "grep",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Search files using basic regular expressions",
        "grep",
        true,
    ));
    #[cfg(feature = "head")]
    applets.push(AppletMetadata::new(
        "head",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Print the first lines of files",
        "head",
        true,
    ));
    #[cfg(feature = "ln")]
    applets.push(AppletMetadata::new(
        "ln",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Create file links",
        "ln",
        true,
    ));
    #[cfg(feature = "ls")]
    applets.push(AppletMetadata::new(
        "ls",
        AppletCategory::Coreutils,
        PosixLevel::Partial,
        "List directory contents",
        "ls",
        true,
    ));
    #[cfg(feature = "mkdir")]
    applets.push(AppletMetadata::new(
        "mkdir",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Create directories",
        "mkdir",
        true,
    ));
    #[cfg(feature = "mv")]
    applets.push(AppletMetadata::new(
        "mv",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Move or rename files",
        "mv",
        true,
    ));
    #[cfg(feature = "pwd")]
    applets.push(AppletMetadata::new(
        "pwd",
        AppletCategory::Coreutils,
        PosixLevel::Full,
        "Print working directory",
        "pwd",
        true,
    ));
    #[cfg(feature = "rm")]
    applets.push(AppletMetadata::new(
        "rm",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Remove files or directories",
        "rm",
        true,
    ));
    #[cfg(feature = "rmdir")]
    applets.push(AppletMetadata::new(
        "rmdir",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Remove empty directories",
        "rmdir",
        true,
    ));
    #[cfg(feature = "sh")]
    applets.push(AppletMetadata::new(
        "sh",
        AppletCategory::Shell,
        PosixLevel::Partial,
        "Run the Nereus POSIX shell subset",
        "sh",
        true,
    ));
    #[cfg(feature = "tail")]
    applets.push(AppletMetadata::new(
        "tail",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Print the last lines of files",
        "tail",
        true,
    ));
    #[cfg(feature = "touch")]
    applets.push(AppletMetadata::new(
        "touch",
        AppletCategory::Coreutils,
        PosixLevel::Substantial,
        "Update file timestamps",
        "touch",
        true,
    ));
    #[cfg(feature = "true")]
    applets.push(AppletMetadata::new(
        "true",
        AppletCategory::Coreutils,
        PosixLevel::Full,
        "Return a zero exit status",
        "true",
        true,
    ));
    #[cfg(feature = "wc")]
    applets.push(AppletMetadata::new(
        "wc",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Count lines, words, and bytes",
        "wc",
        true,
    ));

    applets.sort_unstable_by_key(|metadata| metadata.name);
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
    println!("    proteus --list-full");
    println!("    proteus --posix-info <applet>");
    println!("    proteus --install [-s|--symlink] [-f|--force] <directory>");
    println!("    proteus --uninstall <directory>");
    println!("    proteus --version");
    println!("    proteus --help");
}
