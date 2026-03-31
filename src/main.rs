use std::path::{Path, PathBuf};

use proteus_core::compliance::{AppletCategory, AppletHelp, AppletMetadata, AppletOption, PosixLevel};
use proteus_core::platform::current_platform;
use proteus_core::sandbox::{apply_sandbox_policy, sandbox_report_for, SandboxMode, SandboxReport};
use proteus_core::ProteusResult;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
struct RuntimeOptions {
    sandbox_mode: SandboxMode,
}

#[derive(Debug, Clone)]
struct RuntimeContext {
    sandbox_report: Option<SandboxReport>,
    platform_name: &'static str,
    page_size: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let argv0 = args
        .first()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "proteus".into());

    let exit_code = if argv0 == "proteus" {
        dispatch_multi_call(&args[1..])
    } else {
        dispatch_applet(
            &argv0,
            &args[1..],
            &RuntimeOptions {
                sandbox_mode: SandboxMode::Strict,
            },
        )
    };

    std::process::exit(exit_code);
}

fn dispatch_multi_call(args: &[String]) -> i32 {
    let (runtime_options, command_args) = match parse_runtime_options(args) {
        Ok(result) => result,
        Err(code) => return code,
    };

    if command_args.is_empty() {
        eprintln!("proteus: no applet specified");
        eprintln!("Try 'proteus --list' for a list of available applets.");
        return 1;
    }

    match command_args[0].as_str() {
        "--list" => {
            list_applets();
            0
        }
        "--list-full" => {
            list_applets_full();
            0
        }
        "--posix-info" => print_posix_info(&command_args[1..]),
        "--sandbox-info" => print_sandbox_info(&command_args[1..], runtime_options.sandbox_mode),
        "--version" | "-V" => {
            println!("proteus {VERSION}");
            0
        }
        "--help" | "-h" => {
            print_help();
            0
        }
        "--install" => install_applets(&command_args[1..]),
        "--uninstall" => uninstall_applets(&command_args[1..]),
        applet_name => {
            let applet_args = &command_args[1..];
            dispatch_applet(applet_name, applet_args, &runtime_options)
        }
    }
}

fn dispatch_applet(name: &str, args: &[String], runtime_options: &RuntimeOptions) -> i32 {
    if let Some(help_mode) = parse_applet_help_mode(args) {
        return print_applet_help(name, help_mode);
    }

    let runtime_context = match prepare_runtime_context(name, runtime_options) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("proteus: {name}: {error}");
            return 1;
        }
    };

    match dispatch(name, args, &runtime_context) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("proteus: {name}: {error}");
            1
        }
    }
}

fn dispatch(name: &str, args: &[String], runtime_context: &RuntimeContext) -> ProteusResult<i32> {
    let _ = runtime_context.page_size;
    let _ = runtime_context.platform_name;
    let _ = &runtime_context.sandbox_report;

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
        #[cfg(feature = "sed")]
        "sed" => proteus_applets::textutils::sed::run(args),
        #[cfg(feature = "sort")]
        "sort" => proteus_applets::textutils::sort::run(args),
        #[cfg(feature = "cut")]
        "cut" => proteus_applets::textutils::cut::run(args),
        #[cfg(feature = "tr")]
        "tr" => proteus_applets::textutils::tr::run(args),
        #[cfg(feature = "uniq")]
        "uniq" => proteus_applets::textutils::uniq::run(args),
        #[cfg(feature = "awk")]
        "awk" => proteus_applets::textutils::awk::run(args),
        #[cfg(feature = "find")]
        "find" => proteus_applets::fileutils::find::run(args),
        #[cfg(feature = "xargs")]
        "xargs" => proteus_applets::fileutils::xargs::run(args),
        #[cfg(feature = "tar")]
        "tar" => proteus_applets::fileutils::tar::run(args),
        #[cfg(feature = "gzip")]
        "gzip" => proteus_applets::fileutils::gzip::run(args),
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

fn parse_runtime_options(args: &[String]) -> Result<(RuntimeOptions, &[String]), i32> {
    let mut sandbox_mode = SandboxMode::Strict;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--no-sandbox" => {
                sandbox_mode = SandboxMode::Off;
                index += 1;
            }
            value if value.starts_with("--sandbox=") => {
                let mode_value = &value[10..];
                let Some(mode) = SandboxMode::parse(mode_value) else {
                    eprintln!("proteus: invalid sandbox mode '{mode_value}'");
                    return Err(2);
                };
                sandbox_mode = mode;
                index += 1;
            }
            "--sandbox" => {
                index += 1;
                if index >= args.len() {
                    eprintln!("proteus: --sandbox requires a mode");
                    return Err(2);
                }
                let mode_value = &args[index];
                let Some(mode) = SandboxMode::parse(mode_value) else {
                    eprintln!("proteus: invalid sandbox mode '{mode_value}'");
                    return Err(2);
                };
                sandbox_mode = mode;
                index += 1;
            }
            _ => break,
        }
    }

    Ok((RuntimeOptions { sandbox_mode }, &args[index..]))
}

fn prepare_runtime_context(
    applet_name: &str,
    runtime_options: &RuntimeOptions,
) -> Result<RuntimeContext, proteus_core::ProteusError> {
    let platform = current_platform();
    let sandbox_report = if default_sandboxed_applet(applet_name) {
        Some(
            apply_sandbox_policy(applet_name, runtime_options.sandbox_mode)
                .map_err(|error| proteus_core::ProteusError::Other(error.to_string()))?,
        )
    } else {
        None
    };

    Ok(RuntimeContext {
        sandbox_report,
        platform_name: platform.name(),
        page_size: platform.page_size(),
    })
}

fn default_sandboxed_applet(applet_name: &str) -> bool {
    matches!(
        applet_name,
        "cat"
            | "ls"
            | "cp"
            | "mv"
            | "rm"
            | "echo"
            | "head"
            | "tail"
            | "wc"
            | "pwd"
            | "mkdir"
            | "rmdir"
            | "touch"
            | "chmod"
            | "chown"
            | "chgrp"
            | "ln"
            | "basename"
            | "dirname"
            | "true"
            | "false"
            | "grep"
            | "egrep"
            | "fgrep"
            | "sed"
            | "sort"
            | "cut"
            | "tr"
            | "uniq"
            | "awk"
            | "find"
            | "xargs"
            | "tar"
            | "gzip"
            | "sh"
    )
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
            let help = applet_help(applet_name);
            for line in metadata.to_report_lines(help.as_ref()) {
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

fn print_sandbox_info(args: &[String], mode: SandboxMode) -> i32 {
    let Some(applet_name) = args.first() else {
        eprintln!("proteus: --sandbox-info: missing applet name");
        return 2;
    };

    match sandbox_report_for(applet_name, mode) {
        Ok(report) => {
            println!("applet: {}", report.applet);
            println!("mode: {}", report.mode);
            println!("backend: {:?}", report.backend);
            println!("applied: {}", if report.applied { "yes" } else { "no" });
            println!("degraded: {}", if report.degraded { "yes" } else { "no" });
            println!("supports_strict: {}", if report.policy.supports_strict { "yes" } else { "no" });
            println!("syscalls: {}", report.policy.syscall_classes_as_strings().join(", "));
            println!("capabilities: {}", report.policy.capabilities_as_strings().join(", "));
            if !report.notes.is_empty() {
                println!("notes: {}", report.notes.join(" | "));
            }
            0
        }
        Err(error) => {
            eprintln!("proteus: --sandbox-info: {error}");
            1
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HelpMode {
    Short,
    Full,
}

fn parse_applet_help_mode(args: &[String]) -> Option<HelpMode> {
    if args.iter().any(|arg| arg == "--help-full") {
        Some(HelpMode::Full)
    } else if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        Some(HelpMode::Short)
    } else {
        None
    }
}

fn print_applet_help(name: &str, help_mode: HelpMode) -> i32 {
    let Some(metadata) = applet_metadata(name) else {
        eprintln!("proteus: applet '{name}' not found");
        return 127;
    };
    let Some(help) = applet_help(name) else {
        eprintln!("proteus: {name}: no help available");
        return 1;
    };

    for line in metadata.to_help_lines(&help, matches!(help_mode, HelpMode::Full)) {
        println!("{line}");
    }
    0
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

fn applet_help(name: &str) -> Option<AppletHelp> {
    let help = match name {
        "cat" => AppletHelp::new(
            "proteus cat [-nbsETAtet] [FILE...]",
            vec![
                AppletOption::new("-n", "number output lines"),
                AppletOption::new("-b", "number nonblank lines"),
                AppletOption::new("-s", "suppress repeated blank lines"),
                AppletOption::new("-E", "display $ at end of each line"),
                AppletOption::new("-T", "display TAB as ^I"),
            ],
            vec!["Reads stdin when no FILE is provided.".to_string()],
        ),
        "cp" => AppletHelp::new(
            "proteus cp [-rRfFi] SOURCE... DEST",
            vec![
                AppletOption::new("-r, -R", "copy directories recursively"),
                AppletOption::new("-f", "force overwriting existing destinations"),
                AppletOption::new("-i", "prompt before overwrite"),
            ],
            vec!["Multiple sources require DEST to be a directory.".to_string()],
        ),
        "cut" => AppletHelp::new(
            "proteus cut (-c LIST | -f LIST [-d DELIM]) [FILE...]",
            vec![
                AppletOption::new("-c LIST", "select character positions"),
                AppletOption::new("-f LIST", "select field positions"),
                AppletOption::new("-d DELIM", "set field delimiter for -f mode"),
            ],
            vec!["Character mode is UTF-8 aware through the shared core helpers.".to_string()],
        ),
        "grep" => AppletHelp::new(
            "proteus grep [-EFGclnqv] PATTERN [FILE...]",
            vec![
                AppletOption::new("-E", "use extended regular expressions"),
                AppletOption::new("-F", "match fixed strings"),
                AppletOption::new("-c", "print match counts"),
                AppletOption::new("-l", "print matching file names only"),
                AppletOption::new("-n", "print line numbers"),
                AppletOption::new("-q", "exit on first match"),
                AppletOption::new("-v", "invert match sense"),
            ],
            vec!["Use egrep/fgrep aliases for extended and fixed matching defaults.".to_string()],
        ),
        "egrep" => AppletHelp::new(
            "proteus egrep [-clnqv] PATTERN [FILE...]",
            vec![AppletOption::new("-c|-l|-n|-q|-v", "same filtering controls as grep")],
            vec!["Equivalent to grep with extended regular expressions enabled.".to_string()],
        ),
        "fgrep" => AppletHelp::new(
            "proteus fgrep [-clnqv] PATTERN [FILE...]",
            vec![AppletOption::new("-c|-l|-n|-q|-v", "same filtering controls as grep")],
            vec!["Equivalent to grep with fixed-string matching enabled.".to_string()],
        ),
        "ls" => AppletHelp::new(
            "proteus ls [-a l R] [PATH...]",
            vec![
                AppletOption::new("-a", "include dotfiles"),
                AppletOption::new("-l", "use long listing format"),
                AppletOption::new("-R", "recurse into subdirectories"),
            ],
            vec!["Defaults to the current directory when no PATH is given.".to_string()],
        ),
        "sed" => AppletHelp::new(
            "proteus sed [-e SCRIPT] SCRIPT [FILE...]",
            vec![
                AppletOption::new("-e SCRIPT", "provide an explicit editing script"),
                AppletOption::new("s/old/new/[g]", "apply a basic substitute command"),
            ],
            vec!["Current implementation supports simple substitute scripts only.".to_string()],
        ),
        "sort" => AppletHelp::new(
            "proteus sort [-ru] [FILE...]",
            vec![
                AppletOption::new("-r", "reverse the sorted output"),
                AppletOption::new("-u", "suppress duplicate output lines"),
            ],
            vec!["Sorts the full input set in memory.".to_string()],
        ),
        "find" => AppletHelp::new(
            "proteus find [PATH...] [-name PATTERN] [-type f|d]",
            vec![
                AppletOption::new("-name PATTERN", "filter by simple prefix/suffix/exact pattern"),
                AppletOption::new("-type f|d", "filter by file or directory type"),
            ],
            vec!["Defaults to the current directory when no PATH is provided.".to_string()],
        ),
        "xargs" => AppletHelp::new(
            "proteus xargs [-0] [COMMAND [ARG...]]",
            vec![
                AppletOption::new("-0", "read NUL-delimited input items"),
            ],
            vec!["Executes the given command once with all collected input items.".to_string()],
        ),
        "tar" => AppletHelp::new(
            "proteus tar (-c|-x) -f ARCHIVE [PATH...]",
            vec![
                AppletOption::new("-c", "create an archive"),
                AppletOption::new("-x", "extract an archive"),
                AppletOption::new("-f ARCHIVE", "use the given tar file"),
            ],
            vec!["Implements a ustar-style archive for regular files.".to_string()],
        ),
        "gzip" => AppletHelp::new(
            "proteus gzip [-cd] [FILE]",
            vec![
                AppletOption::new("-c", "write compressed or decompressed data to stdout"),
                AppletOption::new("-d", "decompress instead of compressing"),
            ],
            vec!["Uses miniz_oxide for deflate/inflate operations.".to_string()],
        ),
        "awk" => AppletHelp::new(
            "proteus awk '{print | print $N | print NR}' [FILE...]",
            vec![
                AppletOption::new("{print}", "print the entire current line"),
                AppletOption::new("{print $N}", "print the Nth whitespace-delimited field"),
                AppletOption::new("{print NR}", "print the current record number"),
            ],
            vec!["Current implementation supports a minimal print-oriented awk subset.".to_string()],
        ),
        "tr" => AppletHelp::new(
            "proteus tr [-d] SET1 [SET2] [FILE...]",
            vec![
                AppletOption::new("-d", "delete characters from SET1 instead of translating"),
                AppletOption::new("[:lower:]", "ASCII lowercase class"),
                AppletOption::new("[:upper:]", "ASCII uppercase class"),
            ],
            vec!["Supports basic translate and delete flows with UTF-8 aware case helpers.".to_string()],
        ),
        "uniq" => AppletHelp::new(
            "proteus uniq [-cdu] [FILE...]",
            vec![
                AppletOption::new("-c", "prefix each line with occurrence count"),
                AppletOption::new("-d", "print only repeated lines"),
                AppletOption::new("-u", "print only unique lines"),
            ],
            vec!["Only adjacent duplicate lines are coalesced.".to_string()],
        ),
        _ => {
            let metadata = applet_metadata(name)?;
            AppletHelp::new(
                format!("proteus {} [args...]", metadata.name),
                Vec::new(),
                vec!["Detailed option coverage is not documented yet for this applet.".to_string()],
            )
        }
    };

    Some(help)
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
    #[cfg(feature = "awk")]
    applets.push(AppletMetadata::new(
        "awk",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Run a minimal awk print subset",
        "awk",
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
    #[cfg(feature = "gzip")]
    applets.push(AppletMetadata::new(
        "gzip",
        AppletCategory::FileUtilities,
        PosixLevel::None,
        "Compress or decompress gzip streams",
        "gzip",
        true,
    ));
    #[cfg(feature = "cut")]
    applets.push(AppletMetadata::new(
        "cut",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Select portions of each input line",
        "cut",
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
    #[cfg(feature = "find")]
    applets.push(AppletMetadata::new(
        "find",
        AppletCategory::FileUtilities,
        PosixLevel::Partial,
        "Walk directory trees and filter paths",
        "find",
        true,
    ));
    #[cfg(feature = "sed")]
    applets.push(AppletMetadata::new(
        "sed",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Apply stream editing substitutions",
        "sed",
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
    #[cfg(feature = "sort")]
    applets.push(AppletMetadata::new(
        "sort",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Sort input lines",
        "sort",
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
    #[cfg(feature = "tar")]
    applets.push(AppletMetadata::new(
        "tar",
        AppletCategory::FileUtilities,
        PosixLevel::Partial,
        "Create or extract tar archives",
        "tar",
        true,
    ));
    #[cfg(feature = "tr")]
    applets.push(AppletMetadata::new(
        "tr",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Translate or delete characters",
        "tr",
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
    #[cfg(feature = "uniq")]
    applets.push(AppletMetadata::new(
        "uniq",
        AppletCategory::TextProcessing,
        PosixLevel::Partial,
        "Filter adjacent repeated lines",
        "uniq",
        true,
    ));
    #[cfg(feature = "xargs")]
    applets.push(AppletMetadata::new(
        "xargs",
        AppletCategory::FileUtilities,
        PosixLevel::Partial,
        "Build command lines from standard input",
        "xargs",
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
    println!("    proteus [--sandbox <strict|permissive|off>] <applet> [args...]");
    println!("    proteus [--no-sandbox] <applet> [args...]");
    println!("    proteus --list");
    println!("    proteus --list-full");
    println!("    proteus --posix-info <applet>");
    println!("    proteus --sandbox-info <applet>");
    println!("    proteus --install [-s|--symlink] [-f|--force] <directory>");
    println!("    proteus --uninstall <directory>");
    println!("    proteus --version");
    println!("    proteus --help");
}
