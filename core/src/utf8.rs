pub struct Utf8Chars<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Utf8Chars<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Utf8Chars { bytes: input, pos: 0 }
    }

    pub fn next_char(&mut self) -> Option<char> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        let first = self.bytes[self.pos];
        let (codepoint, len) = if first < 0x80 {
            (first as u32, 1)
        } else if first < 0xE0 {
            if self.pos + 1 >= self.bytes.len() {
                self.pos = self.bytes.len();
                return Some('\u{FFFD}');
            }
            let cp = ((first & 0x1F) as u32) << 6
                | ((self.bytes[self.pos + 1] & 0x3F) as u32);
            (cp, 2)
        } else if first < 0xF0 {
            if self.pos + 2 >= self.bytes.len() {
                self.pos = self.bytes.len();
                return Some('\u{FFFD}');
            }
            let cp = ((first & 0x0F) as u32) << 12
                | ((self.bytes[self.pos + 1] & 0x3F) as u32) << 6
                | ((self.bytes[self.pos + 2] & 0x3F) as u32);
            (cp, 3)
        } else {
            if self.pos + 3 >= self.bytes.len() {
                self.pos = self.bytes.len();
                return Some('\u{FFFD}');
            }
            let cp = ((first & 0x07) as u32) << 18
                | ((self.bytes[self.pos + 1] & 0x3F) as u32) << 12
                | ((self.bytes[self.pos + 2] & 0x3F) as u32) << 6
                | ((self.bytes[self.pos + 3] & 0x3F) as u32);
            (cp, 4)
        };

        self.pos += len;

        if let Some(c) = char::from_u32(codepoint) {
            Some(c)
        } else {
            Some('\u{FFFD}')
        }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }
}

pub fn validate(input: &[u8]) -> Result<(), usize> {
    let mut pos = 0;
    while pos < input.len() {
        let first = input[pos];
        let expected_len = if first < 0x80 {
            1
        } else if first < 0xC0 {
            return Err(pos);
        } else if first < 0xE0 {
            2
        } else if first < 0xF0 {
            3
        } else if first < 0xF8 {
            4
        } else {
            return Err(pos);
        };

        if pos + expected_len > input.len() {
            return Err(pos);
        }

        for i in 1..expected_len {
            if input[pos + i] & 0xC0 != 0x80 {
                return Err(pos);
            }
        }

        pos += expected_len;
    }
    Ok(())
}

pub fn to_uppercase(c: char) -> char {
    // Basic ASCII + common Latin
    if c.is_ascii_lowercase() {
        return (c as u8 - b'a' + b'A') as char;
    }
    // Turkish-specific
    match c {
        'i' => 'I',
        '\u{131}' => 'I', // dotless i -> I
        _ => c.to_uppercase().next().unwrap_or(c),
    }
}

pub fn to_lowercase(c: char) -> char {
    if c.is_ascii_uppercase() {
        return (c as u8 - b'A' + b'a') as char;
    }
    match c {
        'I' => 'i',
        '\u{130}' => 'i', // dotted I -> i
        _ => c.to_lowercase().next().unwrap_or(c),
    }
}

pub fn char_count(input: &[u8]) -> usize {
    let mut count = 0;
    let mut pos = 0;
    while pos < input.len() {
        let first = input[pos];
        if first < 0x80 {
            pos += 1;
        } else if first < 0xE0 {
            pos += 2;
        } else if first < 0xF0 {
            pos += 3;
        } else {
            pos += 4;
        }
        count += 1;
    }
    count
}
