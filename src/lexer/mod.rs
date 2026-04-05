use std::collections::HashSet;
use std::fmt;

/// Represents all possible token types produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    Operator(String),
    Newline,
    EOF,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Colon,
    LeftBracket,
    RightBracket,
    // String interpolation token
    StringInterpolation(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(s) => write!(f, "Keyword({})", s),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::StringLiteral(s) => write!(f, "StringLiteral({})", s),
            Token::NumberLiteral(n) => write!(f, "NumberLiteral({})", n),
            Token::Operator(s) => write!(f, "Operator({})", s),
            Token::Newline => write!(f, "Newline"),
            Token::EOF => write!(f, "EOF"),
            Token::LeftParen => write!(f, "LeftParen"),
            Token::RightParen => write!(f, "RightParen"),
            Token::LeftBrace => write!(f, "LeftBrace"),
            Token::RightBrace => write!(f, "RightBrace"),
            Token::Comma => write!(f, "Comma"),
            Token::Dot => write!(f, "Dot"),
            Token::Colon => write!(f, "Colon"),
            Token::LeftBracket => write!(f, "LeftBracket"),
            Token::RightBracket => write!(f, "RightBracket"),
            Token::StringInterpolation(s) => write!(f, "StringInterpolation({})", s),
        }
    }
}

/// Error types that can occur during lexing.
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { ch: char, position: usize },
    UnterminatedString { position: usize },
    InvalidNumber { value: String, position: usize },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedCharacter { ch, position } => {
                write!(f, "Unexpected character '{}' at position {}", ch, position)
            }
            LexError::UnterminatedString { position } => {
                write!(f, "Unterminated string starting at position {}", position)
            }
            LexError::InvalidNumber { value, position } => {
                write!(f, "Invalid number '{}' at position {}", value, position)
            }
        }
    }
}

impl std::error::Error for LexError {}

/// Set of all reserved keywords (case-insensitive).
fn keywords() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    // File operations
    set.extend(&[
        "create", "delete", "open", "read", "write", "append", "rename", "copy", "move",
        // I/O operations
        "show", "display", "print", "ask", "input",
        // Assignment
        "set", "to", "is", "as",
        // Control flow
        "if", "otherwise", "else", "then", "end", "repeat", "times", "while", "until", "stop",
        // Loop control
        "break", "continue",
        // Functions
        "define", "function", "call", "return", "give", "back",
        // Math operations
        "add", "subtract", "multiply", "divide", "by", "from",
        // Time operations
        "wait", "seconds", "milliseconds",
        // Logic
        "exists", "not", "and", "or",
        // List operations
        "list", "empty", "contains", "count", "length", "at", "index", "push", "pop", "remove",
        // Network
        "connect", "fetch", "send", "request",
        // System
        "run", "exit", "clear",
        // Literals
        "true", "false",
        // Built-in text functions
        "uppercase", "lowercase", "trim", "reverse", "replace", "with", "in",
        "split", "join",
        // Built-in number functions
        "round", "floor", "ceiling", "absolute", "random", "between",
        "power", "square", "root",
        // Built-in list functions
        "first", "last", "sort",
        // Type conversion
        "text", "number", "truth",
        // Structural keywords for built-in syntax
        "of",
        // Error handling
        "try", "catch",
        // Testing
        "test", "expect",
        // File operations
        "exists", "files", "file",
        // Date/Time
        "current", "date", "time",
    ]);
    set
}

/// The Lexer reads source text character by character and produces tokens.
pub struct Lexer {
    chars: Vec<char>,
    position: usize,
    current_char: Option<char>,
    pending_interpolation: Option<String>,
    /// When Some, we're in the middle of scanning a string and should resume
    string_resume: Option<String>,
}

impl Lexer {
    /// Create a new Lexer from source text.
    pub fn new(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let current_char = if chars.is_empty() { None } else { Some(chars[0]) };
        Lexer {
            chars,
            position: 0,
            current_char,
            pending_interpolation: None,
            string_resume: None,
        }
    }

    /// Advance the position by one and update current_char.
    fn advance(&mut self) {
        self.position += 1;
        if self.position >= self.chars.len() {
            self.current_char = None;
        } else {
            self.current_char = Some(self.chars[self.position]);
        }
    }

    /// Peek at the next character without advancing.
    fn peek(&self) -> Option<char> {
        let next_pos = self.position + 1;
        if next_pos < self.chars.len() {
            Some(self.chars[next_pos])
        } else {
            None
        }
    }

    /// Check if a character is alphabetic or underscore.
    fn is_alpha(ch: Option<char>) -> bool {
        match ch {
            Some(c) => c.is_alphabetic() || c == '_',
            None => false,
        }
    }

    /// Check if a character is a digit.
    fn is_digit(ch: Option<char>) -> bool {
        match ch {
            Some(c) => c.is_ascii_digit(),
            None => false,
        }
    }

    /// Check if a character is alphanumeric or underscore.
    fn is_alphanumeric(ch: Option<char>) -> bool {
        match ch {
            Some(c) => c.is_alphanumeric() || c == '_',
            None => false,
        }
    }
    /// Skip horizontal whitespace. Returns true if a newline was encountered.
    fn skip_whitespace(&mut self) -> bool {
        let mut found_newline = false;
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    found_newline = true;
                    self.advance();
                }
                _ => break,
            }
        }
        found_newline
    }

    /// Skip a single-line comment starting with //.
    fn skip_comment(&mut self) {
        if self.current_char == Some('/') && self.peek() == Some('/') {
            while let Some(ch) = self.current_char {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    /// Scan an identifier or keyword.
    fn scan_identifier(&mut self) -> Token {
        let _start = self.position;
        let mut result = String::new();

        while Self::is_alphanumeric(self.current_char) {
            if let Some(ch) = self.current_char {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check if it's a keyword (case-insensitive)
        let lower = result.to_lowercase();
        if keywords().contains(lower.as_str()) {
            Token::Keyword(lower)
        } else {
            Token::Identifier(result)
        }
    }

    /// Scan a string literal enclosed in double quotes, resuming from a previous scan if `resume_from` is set.
    fn scan_string(&mut self, resume_from: Option<String>) -> Result<Token, LexError> {
        let is_resuming = resume_from.is_some();
        let mut result = resume_from.unwrap_or_else(String::new);
        // If resuming, we're already past the opening quote
        if !is_resuming {
            self.advance(); // skip opening quote
        }

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // skip closing quote
                return Ok(Token::StringLiteral(result));
            } else if ch == '{' {
                // Start of interpolation - return what we've accumulated so far
                let token = Token::StringLiteral(result);
                self.advance(); // skip opening brace
                let mut interpolated = String::new();
                while let Some(ch) = self.current_char {
                    if ch == '}' {
                        self.advance(); // skip closing brace
                        break;
                    } else {
                        interpolated.push(ch);
                        self.advance();
                    }
                }
                // Store state to resume string scanning after returning interpolation
                self.pending_interpolation = Some(interpolated);
                self.string_resume = Some(String::new()); // Will continue from current position
                return Ok(token);
            } else if ch == '\\' {
                // Handle escape sequences
                self.advance();
                match self.current_char {
                    Some('n') => {
                        result.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        result.push('\t');
                        self.advance();
                    }
                    Some('\\') => {
                        result.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        result.push('"');
                        self.advance();
                    }
                    Some('r') => {
                        result.push('\r');
                        self.advance();
                    }
                    Some(other) => {
                        // Unknown escape sequence, keep as-is
                        result.push('\\');
                        result.push(other);
                        self.advance();
                    }
                    None => {
                        return Err(LexError::UnterminatedString { position: self.position });
                    }
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        // Reached EOF without closing quote
        Err(LexError::UnterminatedString { position: self.position })
    }

    /// Scan a number literal (integer or float).
    fn scan_number(&mut self) -> Result<Token, LexError> {
        let start = self.position;
        let mut result = String::new();

        // Scan integer part
        while Self::is_digit(self.current_char) {
            if let Some(ch) = self.current_char {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Scan decimal part if present
        if self.current_char == Some('.') && Self::is_digit(self.peek()) {
            result.push('.');
            self.advance();
            while Self::is_digit(self.current_char) {
                if let Some(ch) = self.current_char {
                    result.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        result
            .parse::<f64>()
            .map(Token::NumberLiteral)
            .map_err(|_| LexError::InvalidNumber {
                value: result,
                position: start,
            })
    }

    /// Scan an operator or multi-character symbol.
    fn scan_operator(&mut self) -> Token {
        let mut op = String::new();

        if let Some(ch) = self.current_char {
            op.push(ch);
            self.advance();

            // Check for two-character operators
            if let Some(next) = self.current_char {
                match (op.as_str(), next) {
                    ("=", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("!", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("<", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    (">", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("+", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("-", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("*", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("/", '=') => {
                        op.push(next);
                        self.advance();
                    }
                    ("&", '&') => {
                        op.push(next);
                        self.advance();
                    }
                    ("|", '|') => {
                        op.push(next);
                        self.advance();
                    }
                    _ => {}
                }
            }
        }

        Token::Operator(op)
    }

    /// Get the next token from the source.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Check if we have a pending interpolation token from a string scan
        if let Some(var_name) = self.pending_interpolation.take() {
            return Ok(Token::StringInterpolation(var_name));
        }

        // Check if we need to resume string scanning
        if let Some(accumulated) = self.string_resume.take() {
            return self.scan_string(Some(accumulated));
        }

        loop {
            // Skip whitespace and track newlines
            let found_newline = self.skip_whitespace();

            // Skip comments
            if self.current_char == Some('/') && self.peek() == Some('/') {
                self.skip_comment();
                continue;
            }

            // If we found a newline, emit it first
            if found_newline {
                return Ok(Token::Newline);
            }

            // Now scan the actual token
            match self.current_char {
                None => return Ok(Token::EOF),

                // Identifiers and keywords
                Some(ch) if Self::is_alpha(Some(ch)) => {
                    return Ok(self.scan_identifier());
                }

                // Number literals
                Some(ch) if Self::is_digit(Some(ch)) => {
                    return self.scan_number();
                }

                // String literals
                Some('"') => {
                    return self.scan_string(None);
                }

                // Single-character tokens
                Some('(') => {
                    self.advance();
                    return Ok(Token::LeftParen);
                }
                Some(')') => {
                    self.advance();
                    return Ok(Token::RightParen);
                }
                Some('{') => {
                    self.advance();
                    return Ok(Token::LeftBrace);
                }
                Some('}') => {
                    self.advance();
                    return Ok(Token::RightBrace);
                }
                Some(',') => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                Some('.') => {
                    self.advance();
                    return Ok(Token::Dot);
                }
                Some(':') => {
                    self.advance();
                    return Ok(Token::Colon);
                }
                Some('[') => {
                    self.advance();
                    return Ok(Token::LeftBracket);
                }
                Some(']') => {
                    self.advance();
                    return Ok(Token::RightBracket);
                }

                // Operators
                Some(ch) if "+-*/%=<>!&|^~".contains(ch) => {
                    return Ok(self.scan_operator());
                }

                // Unexpected character
                Some(ch) => {
                    return Err(LexError::UnexpectedCharacter {
                        ch,
                        position: self.position,
                    });
                }
            }
        }
    }

    /// Tokenize the entire source and return all tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token == Token::EOF;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests;
