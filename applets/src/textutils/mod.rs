use std::fs::File;
use std::io::{self, BufRead, BufReader};

use proteus_core::regex::{Regex, RegexSyntax};
use proteus_core::{ProteusError, ProteusResult};

#[cfg(feature = "grep")]
pub mod grep;
#[cfg(feature = "egrep")]
pub mod egrep;
#[cfg(feature = "fgrep")]
pub mod fgrep;
#[cfg(feature = "sed")]
pub mod sed;
#[cfg(feature = "sort")]
pub mod sort;
#[cfg(feature = "cut")]
pub mod cut;
#[cfg(feature = "tr")]
pub mod tr;
#[cfg(feature = "uniq")]
pub mod uniq;
#[cfg(feature = "awk")]
pub mod awk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    Regex(RegexSyntax),
    Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepOptions {
    pub mode: MatchMode,
    pub pattern: String,
    pub invert_match: bool,
    pub count_only: bool,
    pub list_files: bool,
    pub line_number: bool,
    pub quiet: bool,
    pub files: Vec<String>,
}

impl GrepOptions {
    pub fn parse(args: &[String], default_mode: MatchMode) -> Result<Self, i32> {
        let mut mode = default_mode;
        let mut invert_match = false;
        let mut count_only = false;
        let mut list_files = false;
        let mut line_number = false;
        let mut quiet = false;
        let mut pattern: Option<String> = None;
        let mut files = Vec::new();
        let mut literal_mode = false;

        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-v" => invert_match = true,
                "-c" => count_only = true,
                "-l" => list_files = true,
                "-n" => line_number = true,
                "-q" => quiet = true,
                "-F" => {
                    mode = MatchMode::Fixed;
                    literal_mode = true;
                }
                "-E" => {
                    mode = MatchMode::Regex(RegexSyntax::Extended);
                    literal_mode = true;
                }
                "-G" => {
                    mode = MatchMode::Regex(RegexSyntax::Basic);
                    literal_mode = true;
                }
                "-e" => {
                    let Some(value) = iter.next() else {
                        eprintln!("grep: option requires an argument -- 'e'");
                        return Err(2);
                    };
                    pattern = Some(value.clone());
                }
                "--" => {
                    if pattern.is_none() {
                        let Some(value) = iter.next() else {
                            eprintln!("grep: missing pattern");
                            return Err(2);
                        };
                        pattern = Some(value.clone());
                    }
                    files.extend(iter.cloned());
                    break;
                }
                value if value.starts_with('-') && value.len() > 1 && pattern.is_none() => {
                    for option in value[1..].chars() {
                        match option {
                            'v' => invert_match = true,
                            'c' => count_only = true,
                            'l' => list_files = true,
                            'n' => line_number = true,
                            'q' => quiet = true,
                            'F' => {
                                mode = MatchMode::Fixed;
                                literal_mode = true;
                            }
                            'E' => {
                                mode = MatchMode::Regex(RegexSyntax::Extended);
                                literal_mode = true;
                            }
                            'G' => {
                                mode = MatchMode::Regex(RegexSyntax::Basic);
                                literal_mode = true;
                            }
                            _ => {
                                eprintln!("grep: invalid option -- '{option}'");
                                return Err(2);
                            }
                        }
                    }
                }
                value if pattern.is_none() => pattern = Some(value.to_string()),
                value => files.push(value.to_string()),
            }
        }

        let Some(pattern) = pattern else {
            eprintln!("grep: missing pattern");
            return Err(2);
        };

        if !literal_mode {
            mode = default_mode;
        }

        Ok(Self {
            mode,
            pattern,
            invert_match,
            count_only,
            list_files,
            line_number,
            quiet,
            files,
        })
    }
}

pub fn run_grep(args: &[String], default_mode: MatchMode, applet_name: &str) -> ProteusResult<i32> {
    let options = match GrepOptions::parse(args, default_mode) {
        Ok(options) => options,
        Err(code) => return Ok(code),
    };

    let matcher = build_matcher(&options).map_err(ProteusError::Other)?;
    let input_files = if options.files.is_empty() {
        vec!["-".to_string()]
    } else {
        options.files.clone()
    };
    let multi_file = input_files.len() > 1;

    let mut had_error = false;
    let mut matched_any = false;

    for file in &input_files {
        let result = if file == "-" {
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            process_reader(&mut reader, &matcher, &options, multi_file, None)
        } else {
            match File::open(file) {
                Ok(handle) => {
                    let mut reader = BufReader::new(handle);
                    process_reader(&mut reader, &matcher, &options, multi_file, Some(file.as_str()))
                }
                Err(error) => {
                    eprintln!("{applet_name}: {file}: {error}");
                    had_error = true;
                    continue;
                }
            }
        };

        match result {
            Ok(file_matched) => {
                if file_matched {
                    matched_any = true;
                    if options.quiet {
                        return Ok(0);
                    }
                }
            }
            Err(error) => {
                eprintln!("{applet_name}: {error}");
                had_error = true;
            }
        }
    }

    if had_error {
        Ok(2)
    } else if matched_any {
        Ok(0)
    } else {
        Ok(1)
    }
}

pub fn for_each_input<F>(files: &[String], mut callback: F) -> ProteusResult<()>
where
    F: FnMut(&mut dyn BufRead, Option<&str>) -> ProteusResult<()>,
{
    if files.is_empty() {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        callback(&mut reader, None)?;
        return Ok(());
    }

    for file in files {
        if file == "-" {
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            callback(&mut reader, None)?;
            continue;
        }

        let handle = File::open(file).map_err(|error| ProteusError::Other(format!("{file}: {error}")))?;
        let mut reader = BufReader::new(handle);
        callback(&mut reader, Some(file.as_str()))?;
    }

    Ok(())
}

pub fn read_all_lines(files: &[String]) -> ProteusResult<Vec<String>> {
    let mut lines = Vec::new();
    for_each_input(files, |reader, _file_name| {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            lines.push(strip_line_ending(&line).to_string());
        }
        Ok(())
    })?;
    Ok(lines)
}

pub fn read_all_text(files: &[String]) -> ProteusResult<String> {
    let mut text = String::new();
    for_each_input(files, |reader, _file_name| {
        reader.read_to_string(&mut text)?;
        Ok(())
    })?;
    Ok(text)
}

pub fn strip_line_ending(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

enum Matcher {
    Regex(Regex),
    Fixed(String),
}

impl Matcher {
    fn is_match(&self, line: &str) -> bool {
        match self {
            Matcher::Regex(regex) => regex.is_match(line),
            Matcher::Fixed(pattern) => line.contains(pattern),
        }
    }
}

fn build_matcher(options: &GrepOptions) -> Result<Matcher, String> {
    match options.mode {
        MatchMode::Regex(syntax) => Regex::new(&options.pattern, syntax)
            .map(Matcher::Regex)
            .map_err(|error| format!("invalid pattern: {error:?}")),
        MatchMode::Fixed => Ok(Matcher::Fixed(options.pattern.clone())),
    }
}

fn process_reader<R: BufRead>(
    reader: &mut R,
    matcher: &Matcher,
    options: &GrepOptions,
    multi_file: bool,
    file_name: Option<&str>,
) -> io::Result<bool> {
    let mut line = String::new();
    let mut line_index = 0usize;
    let mut match_count = 0usize;
    let mut listed_file = false;

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        line_index += 1;

        let printable_line = line.strip_suffix('\n').unwrap_or(&line);
        let is_match = matcher.is_match(printable_line);
        let selected = if options.invert_match { !is_match } else { is_match };

        if !selected {
            continue;
        }

        match_count += 1;

        if options.quiet {
            return Ok(true);
        }

        if options.list_files {
            if !listed_file {
                println!("{}", file_name.unwrap_or("-"));
                listed_file = true;
            }
            continue;
        }

        if options.count_only {
            continue;
        }

        if multi_file {
            print!("{}:", file_name.unwrap_or("-"));
        }
        if options.line_number {
            print!("{line_index}:");
        }
        print!("{line}");
        if !line.ends_with('\n') {
            println!();
        }
    }

    if options.count_only {
        if multi_file {
            print!("{}:", file_name.unwrap_or("-"));
        }
        println!("{match_count}");
    }

    Ok(match_count > 0)
}
