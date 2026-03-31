use crate::error::{ProteusError, ProteusResult};

pub const USER_READ: u32 = 0o400;
pub const USER_WRITE: u32 = 0o200;
pub const USER_EXECUTE: u32 = 0o100;
pub const GROUP_READ: u32 = 0o040;
pub const GROUP_WRITE: u32 = 0o020;
pub const GROUP_EXECUTE: u32 = 0o010;
pub const OTHER_READ: u32 = 0o004;
pub const OTHER_WRITE: u32 = 0o002;
pub const OTHER_EXECUTE: u32 = 0o001;
pub const SET_UID: u32 = 0o4000;
pub const SET_GID: u32 = 0o2000;
pub const STICKY: u32 = 0o1000;
pub const ALL_PERMISSIONS: u32 = 0o7777;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionTarget {
    User,
    Group,
    Other,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOp {
    Add,
    Remove,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PermissionMask {
    pub mode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolicClause {
    targets: u8,
    op: PermissionOp,
    perms: u32,
    conditional_execute: bool,
}

impl PermissionMask {
    pub fn from_mode(mode: u32) -> Self {
        Self {
            mode: mode & ALL_PERMISSIONS,
        }
    }

    pub fn parse_octal(input: &str) -> ProteusResult<Self> {
        if input.is_empty() || input.len() > 4 {
            return Err(ProteusError::InvalidArgument(format!(
                "invalid octal mode: {input}"
            )));
        }

        if !input.chars().all(|ch| ('0'..='7').contains(&ch)) {
            return Err(ProteusError::InvalidArgument(format!(
                "invalid octal mode: {input}"
            )));
        }

        let mode = u32::from_str_radix(input, 8)
            .map_err(|_| ProteusError::InvalidArgument(format!("invalid octal mode: {input}")))?;

        Ok(Self::from_mode(mode))
    }

    pub fn parse_symbolic(input: &str, current_mode: u32) -> ProteusResult<Self> {
        let mut mode = current_mode & ALL_PERMISSIONS;

        for raw_clause in input.split(',') {
            let clause = parse_clause(raw_clause)?;
            mode = apply_clause(mode, clause);
        }

        Ok(Self::from_mode(mode))
    }

    pub fn parse(input: &str, current_mode: u32) -> ProteusResult<Self> {
        if input.chars().all(|ch| ('0'..='7').contains(&ch)) {
            Self::parse_octal(input)
        } else {
            Self::parse_symbolic(input, current_mode)
        }
    }

    pub fn to_octal_string(self) -> String {
        format!("{:04o}", self.mode & ALL_PERMISSIONS)
    }

    pub fn to_symbolic_string(self) -> String {
        let mode = self.mode & ALL_PERMISSIONS;
        let mut chars = ['-'; 9];

        if mode & USER_READ != 0 {
            chars[0] = 'r';
        }
        if mode & USER_WRITE != 0 {
            chars[1] = 'w';
        }
        if mode & USER_EXECUTE != 0 {
            chars[2] = if mode & SET_UID != 0 { 's' } else { 'x' };
        } else if mode & SET_UID != 0 {
            chars[2] = 'S';
        }

        if mode & GROUP_READ != 0 {
            chars[3] = 'r';
        }
        if mode & GROUP_WRITE != 0 {
            chars[4] = 'w';
        }
        if mode & GROUP_EXECUTE != 0 {
            chars[5] = if mode & SET_GID != 0 { 's' } else { 'x' };
        } else if mode & SET_GID != 0 {
            chars[5] = 'S';
        }

        if mode & OTHER_READ != 0 {
            chars[6] = 'r';
        }
        if mode & OTHER_WRITE != 0 {
            chars[7] = 'w';
        }
        if mode & OTHER_EXECUTE != 0 {
            chars[8] = if mode & STICKY != 0 { 't' } else { 'x' };
        } else if mode & STICKY != 0 {
            chars[8] = 'T';
        }

        chars.iter().collect()
    }
}

fn parse_clause(input: &str) -> ProteusResult<SymbolicClause> {
    if input.is_empty() {
        return Err(ProteusError::InvalidArgument(
            "empty symbolic mode clause".to_string(),
        ));
    }

    let bytes = input.as_bytes();
    let mut index = 0;
    let mut targets = 0u8;

    while index < bytes.len() {
        match bytes[index] as char {
            'u' => targets |= 0b001,
            'g' => targets |= 0b010,
            'o' => targets |= 0b100,
            'a' => targets |= 0b111,
            '+' | '-' | '=' => break,
            _ => break,
        }
        index += 1;
    }

    if targets == 0 {
        targets = 0b111;
    }

    if index >= bytes.len() {
        return Err(ProteusError::InvalidArgument(format!(
            "missing symbolic operator in mode clause: {input}"
        )));
    }

    let op = match bytes[index] as char {
        '+' => PermissionOp::Add,
        '-' => PermissionOp::Remove,
        '=' => PermissionOp::Set,
        _ => {
            return Err(ProteusError::InvalidArgument(format!(
                "invalid symbolic operator in mode clause: {input}"
            )))
        }
    };
    index += 1;

    if index >= bytes.len() {
        return Err(ProteusError::InvalidArgument(format!(
            "missing symbolic permissions in mode clause: {input}"
        )));
    }

    let mut perms = 0u32;
    let mut conditional_execute = false;

    for ch in input[index..].chars() {
        match ch {
            'r' => {
                if targets & 0b001 != 0 {
                    perms |= USER_READ;
                }
                if targets & 0b010 != 0 {
                    perms |= GROUP_READ;
                }
                if targets & 0b100 != 0 {
                    perms |= OTHER_READ;
                }
            }
            'w' => {
                if targets & 0b001 != 0 {
                    perms |= USER_WRITE;
                }
                if targets & 0b010 != 0 {
                    perms |= GROUP_WRITE;
                }
                if targets & 0b100 != 0 {
                    perms |= OTHER_WRITE;
                }
            }
            'x' => {
                if targets & 0b001 != 0 {
                    perms |= USER_EXECUTE;
                }
                if targets & 0b010 != 0 {
                    perms |= GROUP_EXECUTE;
                }
                if targets & 0b100 != 0 {
                    perms |= OTHER_EXECUTE;
                }
            }
            'X' => conditional_execute = true,
            's' => {
                if targets & 0b001 != 0 {
                    perms |= SET_UID;
                }
                if targets & 0b010 != 0 {
                    perms |= SET_GID;
                }
            }
            't' => perms |= STICKY,
            _ => {
                return Err(ProteusError::InvalidArgument(format!(
                    "invalid symbolic permission in mode clause: {input}"
                )))
            }
        }
    }

    Ok(SymbolicClause {
        targets,
        op,
        perms,
        conditional_execute,
    })
}

fn apply_clause(current_mode: u32, clause: SymbolicClause) -> u32 {
    let mut mode = current_mode & ALL_PERMISSIONS;
    let mut perms = clause.perms;

    if clause.conditional_execute && has_any_execute_bit(current_mode) {
        if clause.targets & 0b001 != 0 {
            perms |= USER_EXECUTE;
        }
        if clause.targets & 0b010 != 0 {
            perms |= GROUP_EXECUTE;
        }
        if clause.targets & 0b100 != 0 {
            perms |= OTHER_EXECUTE;
        }
    }

    let clear_mask = target_clear_mask(clause.targets);

    match clause.op {
        PermissionOp::Add => mode |= perms,
        PermissionOp::Remove => mode &= !perms,
        PermissionOp::Set => {
            mode &= !clear_mask;
            mode |= perms;
        }
    }

    mode & ALL_PERMISSIONS
}

fn target_clear_mask(targets: u8) -> u32 {
    let mut mask = 0u32;
    if targets & 0b001 != 0 {
        mask |= USER_READ | USER_WRITE | USER_EXECUTE | SET_UID;
    }
    if targets & 0b010 != 0 {
        mask |= GROUP_READ | GROUP_WRITE | GROUP_EXECUTE | SET_GID;
    }
    if targets & 0b100 != 0 {
        mask |= OTHER_READ | OTHER_WRITE | OTHER_EXECUTE | STICKY;
    }
    mask
}

fn has_any_execute_bit(mode: u32) -> bool {
    mode & (USER_EXECUTE | GROUP_EXECUTE | OTHER_EXECUTE) != 0
}

pub fn parse_mode(input: &str, current_mode: u32) -> ProteusResult<u32> {
    PermissionMask::parse(input, current_mode).map(|mask| mask.mode)
}

pub fn format_mode(mode: u32) -> String {
    PermissionMask::from_mode(mode).to_symbolic_string()
}

#[cfg(test)]
mod tests {
    use super::{format_mode, parse_mode, PermissionMask};

    #[test]
    fn parses_octal_modes() {
        assert_eq!(PermissionMask::parse_octal("755").unwrap().mode, 0o755);
        assert_eq!(PermissionMask::parse_octal("0644").unwrap().mode, 0o644);
    }

    #[test]
    fn rejects_invalid_octal_modes() {
        assert!(PermissionMask::parse_octal("888").is_err());
        assert!(PermissionMask::parse_octal("12345").is_err());
        assert!(PermissionMask::parse_octal("").is_err());
    }

    #[test]
    fn formats_symbolic_string() {
        assert_eq!(PermissionMask::from_mode(0o755).to_symbolic_string(), "rwxr-xr-x");
        assert_eq!(PermissionMask::from_mode(0o4755).to_symbolic_string(), "rwsr-xr-x");
        assert_eq!(PermissionMask::from_mode(0o1777).to_symbolic_string(), "rwxrwxrwt");
    }

    #[test]
    fn applies_symbolic_add_clause() {
        assert_eq!(parse_mode("u+x", 0o644).unwrap(), 0o744);
        assert_eq!(parse_mode("go-w", 0o776).unwrap(), 0o754);
    }

    #[test]
    fn applies_symbolic_set_clause() {
        assert_eq!(parse_mode("u=rw,g=r,o=", 0o777).unwrap(), 0o640);
    }

    #[test]
    fn applies_conditional_execute() {
        assert_eq!(parse_mode("a+X", 0o644).unwrap(), 0o644);
        assert_eq!(parse_mode("a+X", 0o755).unwrap(), 0o755);
        assert_eq!(parse_mode("g+X", 0o744).unwrap(), 0o754);
    }

    #[test]
    fn formats_octal_string() {
        assert_eq!(PermissionMask::from_mode(0o755).to_octal_string(), "0755");
    }

    #[test]
    fn format_mode_helper_matches_symbolic() {
        assert_eq!(format_mode(0o640), "rw-r-----");
    }
}
