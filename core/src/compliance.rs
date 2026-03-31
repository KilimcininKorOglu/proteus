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

    pub fn to_report_lines(self) -> [String; 6] {
        [
            format!("name: {}", self.name),
            format!("category: {}", self.category.as_str()),
            format!("posix_level: {}", self.posix_level.as_str()),
            format!("feature_flag: {}", self.feature_flag),
            format!("multi_call: {}", if self.multi_call { "yes" } else { "no" }),
            format!("description: {}", self.description),
        ]
    }
}
