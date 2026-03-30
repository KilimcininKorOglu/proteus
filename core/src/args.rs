use std::ffi::OsString;

pub struct ArgParser {
    options: Vec<OptDef>,
    positional: Vec<String>,
}

pub struct OptDef {
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub takes_value: bool,
    pub description: &'static str,
}

pub struct ParsedArgs {
    pub options: Vec<(char, Option<String>)>,
    pub positional: Vec<String>,
}

impl ArgParser {
    pub fn new() -> Self {
        ArgParser {
            options: Vec::new(),
            positional: Vec::new(),
        }
    }

    pub fn add_opt(mut self, short: char, long: &'static str, takes_value: bool, description: &'static str) -> Self {
        self.options.push(OptDef {
            short: Some(short),
            long: Some(long),
            takes_value,
            description,
        });
        self
    }

    pub fn parse(&self, args: &[OsString]) -> Result<ParsedArgs, String> {
        let mut parsed_opts: Vec<(char, Option<String>)> = Vec::new();
        let mut positional = Vec::new();
        let mut past_separator = false;
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            let arg_str = arg.to_string_lossy().into_owned();

            if past_separator {
                positional.push(arg_str);
                continue;
            }

            if arg_str == "--" {
                past_separator = true;
                continue;
            }

            if arg_str == "-" {
                positional.push(arg_str);
                continue;
            }

            if arg_str.starts_with("--") {
                let long_arg = &arg_str[2..];
                if let Some(eq_pos) = long_arg.find('=') {
                    let name = &long_arg[..eq_pos];
                    let value = &long_arg[eq_pos + 1..];
                    let short = self.find_short_for_long(name)
                        .ok_or_else(|| format!("unknown option: --{name}"))?;
                    parsed_opts.push((short, Some(value.to_string())));
                } else if let Some((short, takes_value)) = self.find_long(long_arg) {
                    if takes_value {
                        let value = iter.next()
                            .ok_or_else(|| format!("option --{long_arg} requires an argument"))?
                            .to_string_lossy()
                            .into_owned();
                        parsed_opts.push((short, Some(value)));
                    } else {
                        parsed_opts.push((short, None));
                    }
                } else {
                    return Err(format!("unknown option: --{long_arg}"));
                }
            } else if arg_str.starts_with('-') && arg_str.len() > 1 {
                let chars: Vec<char> = arg_str[1..].chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    let c = chars[i];
                    if let Some(takes_value) = self.find_short(c) {
                        if takes_value {
                            let remaining: String = chars[i + 1..].iter().collect();
                            if remaining.is_empty() {
                                let value = iter.next()
                                    .ok_or_else(|| format!("option -{c} requires an argument"))?
                                    .to_string_lossy()
                                    .into_owned();
                                parsed_opts.push((c, Some(value)));
                            } else {
                                parsed_opts.push((c, Some(remaining)));
                            }
                            break;
                        } else {
                            parsed_opts.push((c, None));
                        }
                    } else {
                        return Err(format!("unknown option: -{c}"));
                    }
                    i += 1;
                }
            } else {
                positional.push(arg_str);
            }
        }

        Ok(ParsedArgs {
            options: parsed_opts,
            positional,
        })
    }

    fn find_short(&self, c: char) -> Option<bool> {
        self.options.iter()
            .find(|o| o.short == Some(c))
            .map(|o| o.takes_value)
    }

    fn find_long(&self, name: &str) -> Option<(char, bool)> {
        self.options.iter()
            .find(|o| o.long == Some(name))
            .map(|o| (o.short.unwrap_or('?'), o.takes_value))
    }

    fn find_short_for_long(&self, name: &str) -> Option<char> {
        self.options.iter()
            .find(|o| o.long == Some(name))
            .and_then(|o| o.short)
    }
}

impl ParsedArgs {
    pub fn has_opt(&self, short: char) -> bool {
        self.options.iter().any(|(c, _)| *c == short)
    }

    pub fn opt_value(&self, short: char) -> Option<&str> {
        self.options.iter()
            .find(|(c, _)| *c == short)
            .and_then(|(_, v)| v.as_deref())
    }

    pub fn opt_count(&self, short: char) -> usize {
        self.options.iter().filter(|(c, _)| *c == short).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parser() -> ArgParser {
        ArgParser::new()
            .add_opt('a', "all", false, "show all")
            .add_opt('l', "long", false, "long format")
            .add_opt('n', "lines", true, "number of lines")
            .add_opt('f', "file", true, "input file")
    }

    fn parse_strs(parser: &ArgParser, args: &[&str]) -> Result<ParsedArgs, String> {
        let os_args: Vec<OsString> = args.iter().map(|s| OsString::from(s)).collect();
        parser.parse(&os_args)
    }

    #[test]
    fn test_short_flags_combined() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-al"]).unwrap();
        assert!(result.has_opt('a'));
        assert!(result.has_opt('l'));
        assert_eq!(result.positional.len(), 0);
    }

    #[test]
    fn test_short_flag_with_value() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-n", "10"]).unwrap();
        assert_eq!(result.opt_value('n'), Some("10"));
    }

    #[test]
    fn test_long_flag() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["--all"]).unwrap();
        assert!(result.has_opt('a'));
    }

    #[test]
    fn test_long_flag_with_equals() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["--lines=20"]).unwrap();
        assert_eq!(result.opt_value('n'), Some("20"));
    }

    #[test]
    fn test_positional_args() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-a", "file1.txt", "file2.txt"]).unwrap();
        assert!(result.has_opt('a'));
        assert_eq!(result.positional, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_separator() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-a", "--", "-n", "10"]).unwrap();
        assert!(result.has_opt('a'));
        assert_eq!(result.positional, vec!["-n", "10"]);
    }

    #[test]
    fn test_unknown_option() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-z"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_opt_count() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-a", "-a", "-a"]).unwrap();
        assert_eq!(result.opt_count('a'), 3);
    }

    #[test]
    fn test_short_value_combined() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-n10"]).unwrap();
        assert_eq!(result.opt_value('n'), Some("10"));
    }

    #[test]
    fn test_dash_as_positional() {
        let parser = make_parser();
        let result = parse_strs(&parser, &["-"]).unwrap();
        assert_eq!(result.positional, vec!["-"]);
    }
}
