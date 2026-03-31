use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexSyntax {
    Basic,
    Extended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegexError {
    EmptyPattern,
    UnexpectedEnd,
    UnclosedGroup,
    UnclosedClass,
    InvalidEscape(char),
    InvalidRange(char, char),
    UnexpectedToken(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Empty,
    Literal(char),
    Any,
    Class(CharClass),
    Concat(Vec<Ast>),
    Alternate(Vec<Ast>),
    Repeat {
        node: Box<Ast>,
        min: usize,
        max: Option<usize>,
    },
    AnchorStart,
    AnchorEnd,
    Group(Box<Ast>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharClass {
    negated: bool,
    items: Vec<ClassItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassItem {
    Single(char),
    Range(char, char),
    Digit,
    Alpha,
    Alnum,
    Space,
    Lower,
    Upper,
    Word,
}

#[derive(Debug, Clone)]
pub struct Regex {
    ast: Ast,
    syntax: RegexSyntax,
}

impl Regex {
    pub fn new(pattern: &str, syntax: RegexSyntax) -> Result<Self, RegexError> {
        if pattern.is_empty() {
            return Err(RegexError::EmptyPattern);
        }
        let mut parser = Parser::new(pattern, syntax);
        let ast = parser.parse()?;
        Ok(Self { ast, syntax })
    }

    pub fn syntax(&self) -> RegexSyntax {
        self.syntax
    }

    pub fn is_match(&self, input: &str) -> bool {
        let chars: Vec<char> = input.chars().collect();
        let anchored_start = starts_with_anchor(&self.ast);
        let mut starts = Vec::new();

        if anchored_start {
            starts.push(0);
        } else {
            starts.extend(0..=chars.len());
        }

        for start in starts {
            let states = match_ast(&self.ast, &chars, start);
            if states.iter().any(|&pos| pos <= chars.len()) {
                return true;
            }
        }

        false
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }
}

fn starts_with_anchor(ast: &Ast) -> bool {
    match ast {
        Ast::AnchorStart => true,
        Ast::Concat(nodes) => nodes.first().is_some_and(starts_with_anchor),
        Ast::Group(inner) => starts_with_anchor(inner),
        _ => false,
    }
}

fn match_ast(ast: &Ast, chars: &[char], pos: usize) -> HashSet<usize> {
    match ast {
        Ast::Empty => HashSet::from([pos]),
        Ast::Literal(expected) => {
            if chars.get(pos) == Some(expected) {
                HashSet::from([pos + 1])
            } else {
                HashSet::new()
            }
        }
        Ast::Any => {
            if pos < chars.len() {
                HashSet::from([pos + 1])
            } else {
                HashSet::new()
            }
        }
        Ast::Class(class) => {
            if let Some(&ch) = chars.get(pos) {
                if class.matches(ch) {
                    HashSet::from([pos + 1])
                } else {
                    HashSet::new()
                }
            } else {
                HashSet::new()
            }
        }
        Ast::Concat(nodes) => {
            let mut states = HashSet::from([pos]);
            for node in nodes {
                let mut next_states = HashSet::new();
                for state in &states {
                    next_states.extend(match_ast(node, chars, *state));
                }
                if next_states.is_empty() {
                    return HashSet::new();
                }
                states = next_states;
            }
            states
        }
        Ast::Alternate(branches) => {
            let mut states = HashSet::new();
            for branch in branches {
                states.extend(match_ast(branch, chars, pos));
            }
            states
        }
        Ast::Repeat { node, min, max } => match_repeat(node, *min, *max, chars, pos),
        Ast::AnchorStart => {
            if pos == 0 {
                HashSet::from([pos])
            } else {
                HashSet::new()
            }
        }
        Ast::AnchorEnd => {
            if pos == chars.len() {
                HashSet::from([pos])
            } else {
                HashSet::new()
            }
        }
        Ast::Group(inner) => match_ast(inner, chars, pos),
    }
}

fn match_repeat(
    node: &Ast,
    min: usize,
    max: Option<usize>,
    chars: &[char],
    pos: usize,
) -> HashSet<usize> {
    let mut queue = VecDeque::from([(pos, 0usize)]);
    let mut visited = HashSet::from([(pos, 0usize)]);
    let mut results = HashSet::new();

    while let Some((current_pos, count)) = queue.pop_front() {
        if count >= min {
            results.insert(current_pos);
        }

        if max.is_some_and(|limit| count >= limit) {
            continue;
        }

        let next_positions = match_ast(node, chars, current_pos);
        for next_pos in next_positions {
            if next_pos == current_pos {
                continue;
            }
            let state = (next_pos, count + 1);
            if visited.insert(state) {
                queue.push_back(state);
            }
        }
    }

    results
}

impl CharClass {
    pub fn new(negated: bool, items: Vec<ClassItem>) -> Self {
        Self { negated, items }
    }

    pub fn matches(&self, ch: char) -> bool {
        let matched = self.items.iter().any(|item| item.matches(ch));
        if self.negated { !matched } else { matched }
    }
}

impl ClassItem {
    fn matches(&self, ch: char) -> bool {
        match self {
            ClassItem::Single(value) => *value == ch,
            ClassItem::Range(start, end) => (*start..=*end).contains(&ch),
            ClassItem::Digit => ch.is_ascii_digit(),
            ClassItem::Alpha => ch.is_alphabetic(),
            ClassItem::Alnum => ch.is_alphanumeric(),
            ClassItem::Space => ch.is_whitespace(),
            ClassItem::Lower => ch.is_lowercase(),
            ClassItem::Upper => ch.is_uppercase(),
            ClassItem::Word => ch.is_alphanumeric() || ch == '_',
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    syntax: RegexSyntax,
}

impl Parser {
    fn new(pattern: &str, syntax: RegexSyntax) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
            syntax,
        }
    }

    fn parse(&mut self) -> Result<Ast, RegexError> {
        let ast = self.parse_alternation()?;
        if let Some(ch) = self.peek() {
            return Err(RegexError::UnexpectedToken(ch));
        }
        Ok(ast)
    }

    fn parse_alternation(&mut self) -> Result<Ast, RegexError> {
        let mut branches = vec![self.parse_concatenation()?];

        if self.syntax == RegexSyntax::Extended {
            while self.peek() == Some('|') {
                self.advance();
                branches.push(self.parse_concatenation()?);
            }
        }

        Ok(if branches.len() == 1 {
            branches.remove(0)
        } else {
            Ast::Alternate(branches)
        })
    }

    fn parse_concatenation(&mut self) -> Result<Ast, RegexError> {
        let mut nodes = Vec::new();

        while let Some(ch) = self.peek() {
            if self.ends_concatenation(ch) {
                break;
            }
            let atom = self.parse_atom()?;
            nodes.push(self.parse_quantifier(atom)?);
        }

        Ok(match nodes.len() {
            0 => Ast::Empty,
            1 => nodes.remove(0),
            _ => Ast::Concat(nodes),
        })
    }

    fn parse_atom(&mut self) -> Result<Ast, RegexError> {
        let Some(ch) = self.advance() else {
            return Err(RegexError::UnexpectedEnd);
        };

        match ch {
            '.' => Ok(Ast::Any),
            '^' => Ok(Ast::AnchorStart),
            '$' => Ok(Ast::AnchorEnd),
            '[' => self.parse_class(),
            '(' if self.syntax == RegexSyntax::Extended => {
                let inner = self.parse_alternation()?;
                if self.advance() != Some(')') {
                    return Err(RegexError::UnclosedGroup);
                }
                Ok(Ast::Group(Box::new(inner)))
            }
            '\\' => self.parse_escape(),
            '*' | '+' | '?' | '|' | ')' if self.syntax == RegexSyntax::Extended => {
                Err(RegexError::UnexpectedToken(ch))
            }
            _ => Ok(Ast::Literal(ch)),
        }
    }

    fn parse_quantifier(&mut self, node: Ast) -> Result<Ast, RegexError> {
        let Some(ch) = self.peek() else {
            return Ok(node);
        };

        if self.syntax == RegexSyntax::Basic {
            if ch == '\\' {
                if let Some(next) = self.peek_next() {
                    return match next {
                        '*' => {
                            self.advance();
                            self.advance();
                            Ok(Ast::Repeat {
                                node: Box::new(node),
                                min: 0,
                                max: None,
                            })
                        }
                        '+' => {
                            self.advance();
                            self.advance();
                            Ok(Ast::Repeat {
                                node: Box::new(node),
                                min: 1,
                                max: None,
                            })
                        }
                        '?' => {
                            self.advance();
                            self.advance();
                            Ok(Ast::Repeat {
                                node: Box::new(node),
                                min: 0,
                                max: Some(1),
                            })
                        }
                        _ => Ok(node),
                    };
                }
            }
            return Ok(node);
        }

        match ch {
            '*' => {
                self.advance();
                Ok(Ast::Repeat {
                    node: Box::new(node),
                    min: 0,
                    max: None,
                })
            }
            '+' => {
                self.advance();
                Ok(Ast::Repeat {
                    node: Box::new(node),
                    min: 1,
                    max: None,
                })
            }
            '?' => {
                self.advance();
                Ok(Ast::Repeat {
                    node: Box::new(node),
                    min: 0,
                    max: Some(1),
                })
            }
            _ => Ok(node),
        }
    }

    fn parse_class(&mut self) -> Result<Ast, RegexError> {
        let mut negated = false;
        let mut items = Vec::new();

        if matches!(self.peek(), Some('^') | Some('!')) {
            negated = true;
            self.advance();
        }

        let mut first = true;
        while let Some(ch) = self.peek() {
            if ch == ']' && !first {
                self.advance();
                return Ok(Ast::Class(CharClass::new(negated, items)));
            }
            first = false;

            let item = if ch == '[' && self.peek_next() == Some(':') {
                self.advance();
                self.advance();
                self.parse_posix_class()?
            } else {
                self.parse_class_item()?
            };
            items.push(item);
        }

        Err(RegexError::UnclosedClass)
    }

    fn parse_posix_class(&mut self) -> Result<ClassItem, RegexError> {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch == ':' && self.peek_two_ahead() == Some(']') {
                self.advance();
                self.advance();
                return match name.as_str() {
                    "digit" => Ok(ClassItem::Digit),
                    "alpha" => Ok(ClassItem::Alpha),
                    "alnum" => Ok(ClassItem::Alnum),
                    "space" => Ok(ClassItem::Space),
                    "lower" => Ok(ClassItem::Lower),
                    "upper" => Ok(ClassItem::Upper),
                    "word" => Ok(ClassItem::Word),
                    _ => Err(RegexError::UnexpectedToken('[')),
                };
            }
            name.push(ch);
            self.advance();
        }
        Err(RegexError::UnclosedClass)
    }

    fn parse_class_item(&mut self) -> Result<ClassItem, RegexError> {
        let start = self.parse_class_char()?;
        if self.peek() == Some('-') && self.peek_next() != Some(']') {
            self.advance();
            let end = self.parse_class_char()?;
            if start > end {
                return Err(RegexError::InvalidRange(start, end));
            }
            Ok(ClassItem::Range(start, end))
        } else {
            Ok(ClassItem::Single(start))
        }
    }

    fn parse_class_char(&mut self) -> Result<char, RegexError> {
        let Some(ch) = self.advance() else {
            return Err(RegexError::UnclosedClass);
        };

        if ch == '\\' {
            let Some(escaped) = self.advance() else {
                return Err(RegexError::UnexpectedEnd);
            };
            Ok(escaped)
        } else {
            Ok(ch)
        }
    }

    fn parse_escape(&mut self) -> Result<Ast, RegexError> {
        let Some(ch) = self.advance() else {
            return Err(RegexError::UnexpectedEnd);
        };

        match ch {
            'd' => Ok(Ast::Class(CharClass::new(false, vec![ClassItem::Digit]))),
            'w' => Ok(Ast::Class(CharClass::new(false, vec![ClassItem::Word]))),
            's' => Ok(Ast::Class(CharClass::new(false, vec![ClassItem::Space]))),
            '(' if self.syntax == RegexSyntax::Basic => {
                let inner = self.parse_alternation()?;
                if self.advance() != Some('\\') || self.advance() != Some(')') {
                    return Err(RegexError::UnclosedGroup);
                }
                Ok(Ast::Group(Box::new(inner)))
            }
            '|' if self.syntax == RegexSyntax::Basic => Ok(Ast::Literal('|')),
            '+' | '?' | '*' | '.' | '^' | '$' | '[' | ']' | '(' | ')' | '{' | '}' | '|' | '\\' => {
                Ok(Ast::Literal(ch))
            }
            other => Ok(Ast::Literal(other)),
        }
    }

    fn ends_concatenation(&self, ch: char) -> bool {
        match self.syntax {
            RegexSyntax::Basic => ch == ')' || ch == ']',
            RegexSyntax::Extended => ch == '|' || ch == ')' || ch == ']',
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn peek_two_ahead(&self) -> Option<char> {
        self.chars.get(self.pos + 2).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }
}

#[cfg(test)]
mod tests {
    use super::{Ast, Regex, RegexSyntax};

    #[test]
    fn matches_literal() {
        let regex = Regex::new("abc", RegexSyntax::Extended).unwrap();
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("zabcx"));
        assert!(!regex.is_match("abx"));
    }

    #[test]
    fn matches_any_and_repeat() {
        let regex = Regex::new("a.*c", RegexSyntax::Extended).unwrap();
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("axyzc"));
        assert!(!regex.is_match("abx"));
    }

    #[test]
    fn matches_alternation() {
        let regex = Regex::new("cat|dog", RegexSyntax::Extended).unwrap();
        assert!(regex.is_match("cat"));
        assert!(regex.is_match("dog"));
        assert!(!regex.is_match("cow"));
    }

    #[test]
    fn matches_class_and_range() {
        let regex = Regex::new("[a-c][0-9]", RegexSyntax::Extended).unwrap();
        assert!(regex.is_match("a7"));
        assert!(regex.is_match("c3"));
        assert!(!regex.is_match("d3"));
    }

    #[test]
    fn matches_anchor() {
        let regex = Regex::new("^abc$", RegexSyntax::Extended).unwrap();
        assert!(regex.is_match("abc"));
        assert!(!regex.is_match("zabc"));
        assert!(!regex.is_match("abcz"));
    }

    #[test]
    fn matches_basic_group() {
        let regex = Regex::new("\\(ab\\)*c", RegexSyntax::Basic).unwrap();
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("ababc"));
        assert!(regex.is_match("c"));
    }

    #[test]
    fn parses_ast() {
        let regex = Regex::new("ab+", RegexSyntax::Extended).unwrap();
        assert!(matches!(regex.ast(), Ast::Concat(_)));
    }
}
