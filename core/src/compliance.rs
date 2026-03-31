#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixLevel {
    Full,
    Substantial,
    Partial,
    None,
}

impl PosixLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            PosixLevel::Full => "FULL",
            PosixLevel::Substantial => "SUBSTANTIAL",
            PosixLevel::Partial => "PARTIAL",
            PosixLevel::None => "NONE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppletCategory {
    Coreutils,
    Shell,
    TextProcessing,
    FileUtilities,
    Network,
    Process,
    System,
    Editors,
    Misc,
}

impl AppletCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            AppletCategory::Coreutils => "coreutils",
            AppletCategory::Shell => "shell",
            AppletCategory::TextProcessing => "text-processing",
            AppletCategory::FileUtilities => "file-utilities",
            AppletCategory::Network => "network",
            AppletCategory::Process => "process",
            AppletCategory::System => "system",
            AppletCategory::Editors => "editors",
            AppletCategory::Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppletMetadata {
    pub name: &'static str,
    pub category: AppletCategory,
    pub posix_level: PosixLevel,
    pub description: &'static str,
    pub feature_flag: &'static str,
    pub multi_call: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppletOption {
    pub flag: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppletHelp {
    pub usage: String,
    pub options: Vec<AppletOption>,
    pub notes: Vec<String>,
}

impl AppletOption {
    pub fn new(flag: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            flag: flag.into(),
            description: description.into(),
        }
    }
}

impl AppletHelp {
    pub fn new(
        usage: impl Into<String>,
        options: Vec<AppletOption>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            usage: usage.into(),
            options,
            notes,
        }
    }
}

impl AppletMetadata {
    pub const fn new(
        name: &'static str,
        category: AppletCategory,
        posix_level: PosixLevel,
        description: &'static str,
        feature_flag: &'static str,
        multi_call: bool,
    ) -> Self {
        Self {
            name,
            category,
            posix_level,
            description,
            feature_flag,
            multi_call,
        }
    }

    pub fn to_report_lines(self, help: Option<&AppletHelp>) -> Vec<String> {
        let mut lines = vec![
            format!("name: {}", self.name),
            format!("category: {}", self.category.as_str()),
            format!("posix_level: {}", self.posix_level.as_str()),
            format!("feature_flag: {}", self.feature_flag),
            format!("multi_call: {}", if self.multi_call { "yes" } else { "no" }),
            format!("description: {}", self.description),
        ];

        if let Some(help) = help {
            lines.push(format!("usage: {}", help.usage));
            if !help.options.is_empty() {
                lines.push(format!("options: {}", help.options.len()));
            }
        }

        lines
    }

    pub fn to_help_lines(self, help: &AppletHelp, full: bool) -> Vec<String> {
        let mut lines = vec![
            format!("{} - {}", self.name, self.description),
            String::new(),
            "USAGE:".to_string(),
            format!("  {}", help.usage),
            String::new(),
            "DETAILS:".to_string(),
            format!("  category: {}", self.category.as_str()),
            format!("  posix: {}", self.posix_level.as_str()),
            format!("  feature: {}", self.feature_flag),
        ];

        if full && !help.options.is_empty() {
            lines.push(String::new());
            lines.push("OPTIONS:".to_string());
            for option in &help.options {
                lines.push(format!("  {:<18} {}", option.flag, option.description));
            }
        }

        if !help.notes.is_empty() {
            lines.push(String::new());
            lines.push("NOTES:".to_string());
            for note in &help.notes {
                lines.push(format!("  - {}", note));
            }
        }

        if !full {
            lines.push(String::new());
            lines.push("Use --help-full for detailed option coverage.".to_string());
        }

        lines
    }
}
