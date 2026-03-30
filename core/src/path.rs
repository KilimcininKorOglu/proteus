use std::path::{Path, PathBuf};

pub fn base_name(path: &str) -> &str {
    let p = Path::new(path);
    p.file_name()
        .map(|n| n.to_str().unwrap_or(""))
        .unwrap_or("")
}

pub fn dir_name(path: &str) -> &str {
    let p = Path::new(path);
    p.parent()
        .map(|d| d.to_str().unwrap_or("."))
        .unwrap_or(".")
}

pub fn join(base: &str, rest: &str) -> String {
    let base_path = Path::new(base);
    base_path.join(rest).to_string_lossy().into_owned()
}

pub fn normalize(path: &str) -> String {
    let p = PathBuf::from(path);
    let mut components = Vec::new();

    for comp in p.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                } else {
                    components.push("..".to_string());
                }
            }
            std::path::Component::Normal(s) => {
                components.push(s.to_string_lossy().into_owned());
            }
            std::path::Component::RootDir => {
                components.clear();
                components.push(String::new());
            }
            std::path::Component::Prefix(p) => {
                components.clear();
                components.push(p.as_os_str().to_string_lossy().into_owned());
            }
        }
    }

    let result = components.join("/");
    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, &path[1..]);
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }
    path.to_string()
}
