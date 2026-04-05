use crate::ast::{Node, BuiltInUnaryOp, BuiltInBinaryOp, BuiltInTernaryOp, BuiltInNullaryOp, SourcePosition};
use crate::lexer::Token;
use std::fmt;

/// Error types that can occur during parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    UnexpectedEOF {
        expected: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => write!(
                f,
                "Unexpected token: expected '{}', found '{}' at position {}",
                expected, found, position
            ),
            ParseError::UnexpectedEOF { expected } => {
                write!(f, "Unexpected end of input: expected '{}'", expected)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// The Parser consumes tokens and produces an AST.
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    /// Original source text for position lookups
    source: String,
    /// Byte offset of each token in the source
    token_positions: Vec<usize>,
}

impl Parser {
    /// Create a new Parser from a vector of tokens.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            source: String::new(),
            token_positions: Vec::new(),
        }
    }

    /// Create a new Parser with source text for position tracking.
    pub fn new_with_source(tokens: Vec<Token>, source: String) -> Self {
        // Build approximate byte positions for each token by scanning the source
        let mut token_positions = Vec::with_capacity(tokens.len());
        let mut search_from = 0;
        for token in &tokens {
            // Find the token's text in the source starting from search_from
            let pos = find_token_position(token, &source, search_from);
            token_positions.push(pos);
            search_from = pos;
        }
        Parser {
            tokens,
            position: 0,
            source,
            token_positions,
        }
    }

    /// Get the source position (line, column, line_content) for the current token.
    fn get_source_position(&self) -> (usize, usize, String) {
        let byte_pos = if self.position < self.token_positions.len() {
            self.token_positions[self.position]
        } else {
            self.source.len().saturating_sub(1)
        };
        get_line_info(&self.source, byte_pos)
    }

    /// Peek at the current token without advancing.
    fn peek(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &Token::EOF
        }
    }

    /// Advance to the next token and return the current one.
    fn advance(&mut self) -> Token {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            token
        } else {
            Token::EOF
        }
    }

    /// Skip any newline tokens (statement separators).
    fn skip_newlines(&mut self) {
        while let Token::Newline = self.peek() {
            self.advance();
        }
    }

    /// Check if the current token is a specific keyword.
    fn is_keyword(&self, keyword: &str) -> bool {
        match self.peek() {
            Token::Keyword(k) => k == keyword,
            _ => false,
        }
    }

    /// Expect and consume a specific keyword, or return an error.
    fn expect_keyword(&mut self, keyword: &str) -> Result<(), ParseError> {
        match self.peek() {
            Token::Keyword(k) if k == keyword => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::UnexpectedToken {
                expected: format!("keyword '{}'", keyword),
                found: format!("{}", other),
                position: self.position,
            }),
        }
    }

    /// Expect and consume an identifier, returning its name.
    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{}", other),
                position: self.position,
            }),
        }
    }

    /// Expect and consume a specific word, which may be either a keyword or an identifier.
    fn expect_word(&mut self, word: &str) -> Result<(), ParseError> {
        match self.peek() {
            Token::Keyword(k) if k == word => {
                self.advance();
                Ok(())
            }
            Token::Identifier(name) if name == word => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::UnexpectedToken {
                expected: format!("'{}'", word),
                found: format!("{}", other),
                position: self.position,
            }),
        }
    }

    /// Check if the current token is a specific word (keyword or identifier).
    fn is_word(&self, word: &str) -> bool {
        match self.peek() {
            Token::Keyword(k) if k == word => true,
            Token::Identifier(name) if name == word => true,
            _ => false,
        }
    }

    /// Parse the entire token stream into a Program node.
    pub fn parse_program(&mut self) -> Result<Node, ParseError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while *self.peek() != Token::EOF {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }
        Ok(Node::Program(statements))
    }

    /// Parse a single statement based on the current token.
    fn parse_statement(&mut self) -> Result<Node, ParseError> {
        self.skip_newlines();
        if *self.peek() == Token::EOF {
            return Err(ParseError::UnexpectedEOF {
                expected: "statement".to_string(),
            });
        }

        if self.is_keyword("set") {
            self.parse_set()
        } else if self.is_keyword("show") || self.is_keyword("display") || self.is_keyword("print") {
            self.parse_show()
        } else if self.is_keyword("ask") || self.is_keyword("input") {
            self.parse_ask()
        } else if self.is_keyword("create") {
            self.parse_create_file()
        } else if self.is_keyword("delete") {
            self.parse_delete_file()
        } else if self.is_keyword("read") {
            self.parse_read_file()
        } else if self.is_keyword("write") {
            self.parse_write_file()
        } else if self.is_keyword("append") {
            self.parse_append_file()
        } else if self.is_keyword("copy") {
            self.parse_copy_file()
        } else if self.is_keyword("rename") || self.is_keyword("move") {
            self.parse_rename_file()
        } else if self.is_keyword("if") {
            self.parse_if()
        } else if self.is_keyword("repeat") {
            self.parse_repeat()
        } else if self.is_keyword("while") || self.is_keyword("until") {
            self.parse_while()
        } else if self.is_keyword("define") || self.is_keyword("function") {
            self.parse_define()
        } else if self.is_keyword("call") {
            self.parse_call()
        } else if self.is_keyword("return") || self.is_keyword("give") {
            self.parse_return()
        } else if self.is_keyword("exit") {
            self.parse_exit()
        } else if self.is_keyword("break") {
            self.parse_break()
        } else if self.is_keyword("continue") {
            self.parse_continue()
        } else if self.is_keyword("try") {
            self.parse_try()
        } else if self.is_keyword("test") {
            self.parse_test()
        } else if self.is_keyword("expect") {
            self.parse_expect()
        } else {
            // Try parsing as an expression statement
            let expr = self.parse_expression()?;
            Ok(expr)
        }
    }

    /// Parse: set <identifier> to <expression>
    fn parse_set(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("set")?;
        let name = self.expect_identifier()?;
        // Accept "to", "is", or "=" as assignment operator
        if self.is_keyword("to") || self.is_keyword("is") || self.is_keyword("as") {
            self.advance();
        } else if let Token::Operator(op) = self.peek() {
            if op == "=" {
                self.advance();
            }
        }

        // Parse the value expression, handling `funcname with args` as a function call
        let value = self.parse_expression_with_call()?;

        Ok(Node::Set {
            name,
            value: Box::new(value),
            pos: None,
        })
    }

    /// Parse: show <expression>
    fn parse_show(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("show")?;
        let expr = self.parse_expression_with_call()?;
        Ok(Node::Show(Box::new(expr)))
    }

    /// Parse: ask <expression> into <identifier>
    fn parse_ask(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("ask")?;
        let prompt = self.parse_expression()?;
        self.expect_word("into")?;
        let into = self.expect_identifier()?;
        Ok(Node::Ask {
            prompt: Box::new(prompt),
            into,
        })
    }

    /// Parse: create <expression>
    fn parse_create_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("create")?;
        let path = self.parse_expression()?;
        Ok(Node::CreateFile(Box::new(path)))
    }

    /// Parse: delete <expression>
    fn parse_delete_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("delete")?;
        let path = self.parse_expression()?;
        Ok(Node::DeleteFile(Box::new(path)))
    }

    /// Parse: read <expression> into <identifier>
    fn parse_read_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("read")?;
        let path = self.parse_expression()?;
        self.expect_word("into")?;
        let into = self.expect_identifier()?;
        Ok(Node::ReadFile {
            path: Box::new(path),
            into,
        })
    }

    /// Parse: write <expression> content <expression>
    fn parse_write_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("write")?;
        let path = self.parse_expression()?;
        self.expect_word("content")?;
        let content = self.parse_expression()?;
        Ok(Node::WriteFile {
            path: Box::new(path),
            content: Box::new(content),
        })
    }

    /// Parse: append <expression> content <expression>
    fn parse_append_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("append")?;
        let path = self.parse_expression()?;
        self.expect_word("content")?;
        let content = self.parse_expression()?;
        Ok(Node::AppendFile {
            path: Box::new(path),
            content: Box::new(content),
        })
    }

    /// Parse: copy <expression> to <expression>
    fn parse_copy_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("copy")?;
        let from = self.parse_expression()?;
        self.expect_keyword("to")?;
        let to = self.parse_expression()?;
        Ok(Node::CopyFile {
            from: Box::new(from),
            to: Box::new(to),
        })
    }

    /// Parse: rename <expression> to <expression>
    fn parse_rename_file(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("rename")?;
        let from = self.parse_expression()?;
        self.expect_keyword("to")?;
        let to = self.parse_expression()?;
        Ok(Node::RenameFile {
            from: Box::new(from),
            to: Box::new(to),
        })
    }

    /// Parse: if <expression> then <statements>* [otherwise <statements>*] end
    fn parse_if(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("if")?;
        let condition = self.parse_expression()?;
        self.expect_keyword("then")?;
        let body = self.parse_block_until_keywords(&["otherwise", "else", "end"])?;
        let otherwise = if self.is_keyword("otherwise") || self.is_keyword("else") {
            self.advance();
            Some(self.parse_block_until_keywords(&["end"])?)
        } else {
            None
        };
        // Consume "end" if present
        if self.is_keyword("end") {
            self.advance();
        }
        Ok(Node::If {
            condition: Box::new(condition),
            body,
            otherwise,
        })
    }

    /// Parse: repeat <expression> times { <statements>* }
    ///   or: repeat from <expr> to <expr> as <var> { <statements>* }
    fn parse_repeat(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("repeat")?;
        
        // Check for range syntax: repeat from X to Y as i { }
        if self.is_keyword("from") {
            self.advance(); // consume "from"
            let start = self.parse_primary_no_type_conv()?;
            self.expect_keyword("to")?;
            let end = self.parse_primary_no_type_conv()?;
            let mut var_name = "i".to_string();
            if self.is_keyword("as") {
                self.advance();
                var_name = self.expect_identifier()?;
            }
            // Expect opening brace
            match self.peek() {
                Token::LeftBrace => {
                    self.advance();
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "{".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            }
            let body = self.parse_block_until_keywords(&[])?;
            // Consume closing brace
            if let Token::RightBrace = self.peek() {
                self.advance();
            }
            return Ok(Node::RepeatRange {
                start: Box::new(start),
                end: Box::new(end),
                var_name,
                body,
            });
        }
        
        let times = self.parse_expression()?;
        self.expect_keyword("times")?;
        // Check for optional `as <var>` for named loop variable
        let mut var_name: Option<String> = None;
        if self.is_keyword("as") {
            self.advance();
            var_name = Some(self.expect_identifier()?);
        }
        // Expect opening brace
        match self.peek() {
            Token::LeftBrace => {
                self.advance();
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "{".to_string(),
                    found: format!("{}", other),
                    position: self.position,
                });
            }
        }
        let body = self.parse_block_until_keywords(&[])?;
        // Consume closing brace
        if let Token::RightBrace = self.peek() {
            self.advance();
        }
        Ok(Node::Repeat {
            times: Box::new(times),
            body,
            var_name,
        })
    }

    /// Parse: while <expression> [do] <statements>* end
    fn parse_while(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("while")?;
        let condition = self.parse_expression()?;
        // "do" is optional (may be keyword or identifier)
        if self.is_word("do") {
            self.advance();
        }
        let body = self.parse_block_until_keywords(&["end"])?;
        if self.is_keyword("end") {
            self.advance();
        }
        Ok(Node::While {
            condition: Box::new(condition),
            body,
        })
    }

    /// Parse: define <identifier> [with <params>*] <statements>* end
    fn parse_define(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("define")?;
        let name = self.expect_word_as_identifier()?;
        let mut params = Vec::new();
        if self.is_word("with") {
            self.advance();
            // Parse comma-separated identifiers (or words)
            params.push(self.expect_word_as_identifier()?);
            while let Token::Comma = self.peek() {
                self.advance();
                params.push(self.expect_word_as_identifier()?);
            }
        }
        let body = self.parse_block_until_keywords(&["end"])?;
        if self.is_keyword("end") {
            self.advance();
        }
        Ok(Node::Define {
            name,
            params,
            body,
        })
    }

    /// Parse: call <identifier> [with <args>*]
    fn parse_call(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("call")?;
        let name = self.expect_identifier()?;
        let mut args = Vec::new();
        if self.is_word("with") {
            self.advance();
            args.push(self.parse_expression()?);
            while let Token::Comma = self.peek() {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }
        Ok(Node::Call { name, args })
    }

    /// Parse: return <expression>
    fn parse_return(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("return")?;
        let expr = self.parse_expression_with_call()?;
        Ok(Node::Return(Box::new(expr)))
    }

    /// Parse: exit
    fn parse_exit(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("exit")?;
        Ok(Node::Exit)
    }

    /// Parse: break
    fn parse_break(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("break")?;
        Ok(Node::Break)
    }

    /// Parse: continue
    fn parse_continue(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("continue")?;
        Ok(Node::Continue)
    }

    /// Parse: try <statements>* catch <var> <statements>* end
    fn parse_try(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("try")?;
        let body = self.parse_block_until_keywords(&["catch"])?;
        self.expect_keyword("catch")?;
        let catch_var = self.expect_identifier()?;
        let catch_body = self.parse_block_until_keywords(&["end"])?;
        if self.is_keyword("end") {
            self.advance();
        }
        Ok(Node::Try {
            body,
            catch_var,
            catch_body,
        })
    }

    /// Parse: test "name" <statements>* end
    fn parse_test(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("test")?;
        // Expect a string literal for the test name
        let name = match self.peek() {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            other => {
                return Err(ParseError::UnexpectedToken {
                    expected: "test name (string)".to_string(),
                    found: format!("{}", other),
                    position: self.position,
                });
            }
        };
        let body = self.parse_block_until_keywords(&["end"])?;
        if self.is_keyword("end") {
            self.advance();
        }
        Ok(Node::TestBlock { name, body })
    }

    /// Parse: expect <expression> is <expression>
    fn parse_expect(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("expect")?;
        // Use parse_term to avoid consuming 'is' (which comparison handles as ==)
        let left = self.parse_term()?;
        // Expect 'is' keyword
        self.expect_keyword("is")?;
        let right = self.parse_term()?;
        // Build SourcePosition from token position
        let (line_num, col, line_content) = self.get_source_position();
        Ok(Node::Expect {
            left: Box::new(left),
            right: Box::new(right),
            pos: Some(SourcePosition::new(line_num, col, line_content)),
        })
    }

    /// Parse statements until we hit one of the terminator keywords.
    fn parse_block_until_keywords(
        &mut self,
        terminators: &[&str],
    ) -> Result<Vec<Node>, ParseError> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while *self.peek() != Token::EOF {
            // Check for any terminator keyword
            if let Token::Keyword(k) = self.peek() {
                if terminators.iter().any(|t| t == &k.as_str()) {
                    break;
                }
            }
            // For brace-terminated blocks, check for RightBrace
            if matches!(self.peek(), Token::RightBrace) {
                break;
            }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }
        Ok(statements)
    }

    /// Parse an expression with operator precedence.
    /// Handles: or, and, comparison, term, factor, unary, primary
    fn parse_expression(&mut self) -> Result<Node, ParseError> {
        self.parse_or()
    }

    /// Parse an expression that may start with `identifier with args` as a function call.
    /// This is used at statement level (show, return, set x to, etc.) where the expression
    /// starts fresh and an identifier followed by 'with' should be treated as a function call.
    fn parse_expression_with_call(&mut self) -> Result<Node, ParseError> {
        // Check if the expression starts with `identifier with args`
        if let Token::Identifier(_) = self.peek() {
            let is_func_with = self.position + 1 < self.tokens.len()
                && matches!(&self.tokens[self.position + 1], Token::Keyword(k) if k == "with");
            if is_func_with {
                let Token::Identifier(func_name)  = self.peek() else {
                    unreachable!()
                };
                let func_name = func_name.clone();
                self.advance(); // consume identifier
                self.advance(); // consume 'with'
                let mut args = Vec::new();
                args.push(self.parse_expression()?);
                while let Token::Comma = self.peek() {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
                return Ok(Node::Call {
                    name: func_name,
                    args,
                });
            }
        }
        // Fall through to normal expression parsing
        self.parse_or()
    }

    /// Parse: or expression (lowest precedence)
    fn parse_or(&mut self) -> Result<Node, ParseError> {
        let mut left = self.parse_and()?;
        loop {
            if self.is_keyword("or") {
                self.advance();
                let right = self.parse_and()?;
                left = Node::BinaryOp {
                    left: Box::new(left),
                    op: "or".to_string(),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse: and expression
    fn parse_and(&mut self) -> Result<Node, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.is_keyword("and") {
                self.advance();
                let right = self.parse_comparison()?;
                left = Node::BinaryOp {
                    left: Box::new(left),
                    op: "and".to_string(),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse: comparison (is, !=, <, >, <=, >=, contains)
    fn parse_comparison(&mut self) -> Result<Node, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            let op = if self.is_keyword("is") {
                Some("==".to_string())
            } else if self.is_keyword("contains") {
                // "x contains y" - check if x contains y (string or list)
                self.advance();
                let right = self.parse_term()?;
                left = Node::BuiltInBinary {
                    op: BuiltInBinaryOp::Contains,
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            } else if let Token::Operator(op) = self.peek() {
                if op == "!=" || op == "<" || op == ">" || op == "<=" || op == ">=" || op == "==" {
                    Some(op.clone())
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_term()?;
                left = Node::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse: term (add, subtract)
    fn parse_term(&mut self) -> Result<Node, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            let op = if self.is_keyword("add") {
                Some("+".to_string())
            } else if self.is_keyword("subtract") {
                Some("-".to_string())
            } else {
                None
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_factor()?;
                left = Node::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse: factor (multiply, divide)
    fn parse_factor(&mut self) -> Result<Node, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.is_keyword("multiply") {
                Some("*".to_string())
            } else if self.is_keyword("divide") {
                Some("/".to_string())
            } else {
                None
            };
            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary()?;
                left = Node::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse: unary (not, built-in prefix operations)
    fn parse_unary(&mut self) -> Result<Node, ParseError> {
        if self.is_keyword("not") {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Node::BinaryOp {
                left: Box::new(Node::BoolLit(true)),
                op: "not".to_string(),
                right: Box::new(operand),
            });
        }

        // Built-in prefix operations
        if self.is_keyword("uppercase")
            || self.is_keyword("lowercase")
            || self.is_keyword("length")
            || self.is_keyword("trim")
            || self.is_keyword("reverse")
            || self.is_keyword("count")
            || self.is_keyword("first")
            || self.is_keyword("last")
            || self.is_keyword("sort")
        {
            return self.parse_builtin_unary_of();
        }

        if self.is_keyword("round") || self.is_keyword("floor") || self.is_keyword("ceiling") {
            return self.parse_builtin_unary_direct();
        }

        if self.is_keyword("absolute") {
            return self.parse_builtin_absolute();
        }

        if self.is_keyword("split") {
            return self.parse_builtin_split();
        }

        if self.is_keyword("square") {
            return self.parse_builtin_sqrt();
        }

        if self.is_keyword("push") {
            return self.parse_builtin_push();
        }

        if self.is_keyword("pop") {
            return self.parse_builtin_pop();
        }

        if self.is_keyword("random") {
            return self.parse_builtin_random();
        }

        if self.is_keyword("join") {
            return self.parse_builtin_join();
        }

        if self.is_keyword("replace") {
            return self.parse_builtin_replace();
        }

        if self.is_keyword("file") {
            return self.parse_builtin_file_exists();
        }

        if self.is_keyword("list") {
            // Peek ahead: if next token after "list" is "files", parse as list files
            let is_list_files = self.position + 1 < self.tokens.len() 
                && matches!(&self.tokens[self.position + 1], Token::Keyword(k) if k == "files");
            
            if is_list_files {
                return self.parse_builtin_list_files();
            }
            // Otherwise, fall through to parse_primary which handles list literals
        }

        if self.is_keyword("files") {
            return self.parse_builtin_list_files();
        }

        if self.is_keyword("current") {
            return self.parse_builtin_current();
        }

        if self.is_keyword("power") {
            return self.parse_builtin_power();
        }

        self.parse_primary()
    }

    /// Parse: <keyword> of <expression>
    fn parse_builtin_unary_of(&mut self) -> Result<Node, ParseError> {
        let op = match self.peek() {
            Token::Keyword(k) => k.clone(),
            _ => return self.parse_primary(),
        };
        self.advance();
        self.expect_keyword("of")?;
        let operand = self.parse_unary()?;

        let built_in_op = match op.as_str() {
            "uppercase" => BuiltInUnaryOp::Uppercase,
            "lowercase" => BuiltInUnaryOp::Lowercase,
            "length" => BuiltInUnaryOp::Length,
            "trim" => BuiltInUnaryOp::Trim,
            "reverse" => BuiltInUnaryOp::Reverse,
            "count" => BuiltInUnaryOp::Count,
            "first" => BuiltInUnaryOp::First,
            "last" => BuiltInUnaryOp::Last,
            "sort" => BuiltInUnaryOp::Sort,
            _ => return Err(ParseError::UnexpectedToken {
                expected: "built-in operation".to_string(),
                found: op,
                position: self.position,
            }),
        };

        Ok(Node::BuiltInUnary {
            op: built_in_op,
            operand: Box::new(operand),
        })
    }

    /// Parse: <keyword> <expression> (for round, floor, ceiling)
    fn parse_builtin_unary_direct(&mut self) -> Result<Node, ParseError> {
        let op = match self.peek() {
            Token::Keyword(k) => k.clone(),
            _ => return self.parse_primary(),
        };
        self.advance();
        let operand = self.parse_unary()?;

        let built_in_op = match op.as_str() {
            "round" => BuiltInUnaryOp::Round,
            "floor" => BuiltInUnaryOp::Floor,
            "ceiling" => BuiltInUnaryOp::Ceiling,
            _ => return Err(ParseError::UnexpectedToken {
                expected: "built-in operation".to_string(),
                found: op,
                position: self.position,
            }),
        };

        Ok(Node::BuiltInUnary {
            op: built_in_op,
            operand: Box::new(operand),
        })
    }

    /// Parse: absolute of <expression>
    fn parse_builtin_absolute(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("absolute")?;
        self.expect_keyword("of")?;
        let operand = self.parse_unary()?;
        Ok(Node::BuiltInUnary {
            op: BuiltInUnaryOp::Absolute,
            operand: Box::new(operand),
        })
    }

    /// Parse: push <expression> to <expression>
    fn parse_builtin_push(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("push")?;
        let value = self.parse_expression()?;
        self.expect_keyword("to")?;
        let list = self.parse_expression()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::PushTo,
            left: Box::new(value),
            right: Box::new(list),
        })
    }

    /// Parse: pop from <expression>
    fn parse_builtin_pop(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("pop")?;
        self.expect_keyword("from")?;
        let list = self.parse_expression()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::PopFrom,
            left: Box::new(list),
            right: Box::new(Node::NumberLit(0.0)), // placeholder
        })
    }

    /// Parse: random between <expression> and <expression>
    fn parse_builtin_random(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("random")?;
        self.expect_keyword("between")?;
        // Use parse_comparison instead of parse_expression to avoid consuming 'and' keyword
        let low = self.parse_comparison()?;
        self.expect_keyword("and")?;
        let high = self.parse_comparison()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::RandomBetween,
            left: Box::new(low),
            right: Box::new(high),
        })
    }

    /// Parse: replace <expression> with <expression> in <expression>
    fn parse_builtin_replace(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("replace")?;
        let old = self.parse_expression()?;
        self.expect_keyword("with")?;
        let new = self.parse_expression()?;
        self.expect_keyword("in")?;
        let target = self.parse_expression()?;
        Ok(Node::BuiltInTernary {
            op: BuiltInTernaryOp::ReplaceIn,
            first: Box::new(old),
            second: Box::new(new),
            third: Box::new(target),
        })
    }

    /// Parse: split <expression> by <expression>
    fn parse_builtin_split(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("split")?;
        let text = self.parse_expression()?;
        self.expect_keyword("by")?;
        let delimiter = self.parse_expression()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::Split,
            left: Box::new(text),
            right: Box::new(delimiter),
        })
    }

    /// Parse: square root of <expression>
    fn parse_builtin_sqrt(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("square")?;
        self.expect_keyword("root")?;
        self.expect_keyword("of")?;
        let operand = self.parse_unary()?;
        Ok(Node::BuiltInUnary {
            op: BuiltInUnaryOp::Sqrt,
            operand: Box::new(operand),
        })
    }

    /// Parse: join <expression> with <expression>
    fn parse_builtin_join(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("join")?;
        let list = self.parse_expression()?;
        self.expect_keyword("with")?;
        let separator = self.parse_expression()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::JoinWith,
            left: Box::new(list),
            right: Box::new(separator),
        })
    }

    /// Parse: file exists <expression>
    fn parse_builtin_file_exists(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("file")?;
        self.expect_keyword("exists")?;
        let path = self.parse_expression()?;
        Ok(Node::BuiltInUnary {
            op: BuiltInUnaryOp::FileExists,
            operand: Box::new(path),
        })
    }

    /// Parse: list files in <expression>
    fn parse_builtin_list_files(&mut self) -> Result<Node, ParseError> {
        // Handle both "list files in <path>" and "files in <path>"
        if self.is_keyword("list") {
            self.advance();
        }
        self.expect_keyword("files")?;
        self.expect_keyword("in")?;
        let path = self.parse_expression()?;
        Ok(Node::BuiltInUnary {
            op: BuiltInUnaryOp::ListFiles,
            operand: Box::new(path),
        })
    }

    /// Parse: current date or current time
    fn parse_builtin_current(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("current")?;
        if self.is_keyword("date") {
            self.advance();
            Ok(Node::BuiltInNullary {
                op: BuiltInNullaryOp::CurrentDate,
            })
        } else if self.is_keyword("time") {
            self.advance();
            Ok(Node::BuiltInNullary {
                op: BuiltInNullaryOp::CurrentTime,
            })
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "date or time".to_string(),
                found: format!("{}", self.peek()),
                position: self.position,
            })
        }
    }

    /// Parse: power <expression> by <expression>
    fn parse_builtin_power(&mut self) -> Result<Node, ParseError> {
        self.expect_keyword("power")?;
        let base = self.parse_expression()?;
        self.expect_keyword("by")?;
        let exponent = self.parse_expression()?;
        Ok(Node::BuiltInBinary {
            op: BuiltInBinaryOp::Power,
            left: Box::new(base),
            right: Box::new(exponent),
        })
    }

    /// Parse: primary expression (literals, identifiers, parenthesized expressions, lists)
    /// Also handles: <expr> as text/number/truth
    fn parse_primary(&mut self) -> Result<Node, ParseError> {
        // Handle interpolated strings: sequence of StringLiteral + StringInterpolation tokens
        if let Token::StringLiteral(_) = self.peek() {
            return self.parse_interpolated_string();
        }

        // Handle dictionary literal: { key: value, ... }
        if let Token::LeftBrace = self.peek() {
            return self.parse_dict_literal();
        }

        let result = match self.peek() {
            Token::NumberLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Node::NumberLit(n))
            }
            Token::Keyword(k) if k == "true" => {
                self.advance();
                Ok(Node::BoolLit(true))
            }
            Token::Keyword(k) if k == "false" => {
                self.advance();
                Ok(Node::BoolLit(false))
            }
            Token::Keyword(k) if k == "list" => {
                self.advance();
                self.parse_list_literal()
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                // Check if this is a function call: name(...)
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RightParen) {
                        args.push(self.parse_expression()?);
                        while let Token::Comma = self.peek() {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect_right_paren()?;
                    Ok(Node::Call { name, args })
                } else {
                    Ok(Node::Identifier(name))
                }
            }
            // Handle specific keywords used as identifiers (e.g., `text`, `number`, `truth` as variable names)
            // Only allow keywords that make sense as variable/function names, not structural keywords
            Token::Keyword(kw) if kw == "text" || kw == "number" || kw == "truth"
                || kw == "uppercase" || kw == "lowercase" || kw == "length" || kw == "trim"
                || kw == "reverse" || kw == "round" || kw == "floor" || kw == "ceiling"
                || kw == "absolute" || kw == "random" || kw == "between"
                || kw == "count" || kw == "first" || kw == "last" || kw == "sort"
                || kw == "push" || kw == "pop" || kw == "from"
                || kw == "replace" || kw == "with" || kw == "in" || kw == "of"
                || kw == "contains" =>
            {
                let name = kw.clone();
                self.advance();
                // Check if this is a function call: name(...)
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance(); // consume (
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RightParen) {
                        args.push(self.parse_expression()?);
                        while let Token::Comma = self.peek() {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect_right_paren()?;
                    Ok(Node::Call { name, args })
                } else {
                    Ok(Node::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance(); // consume (
                let expr = self.parse_expression()?;
                self.expect_right_paren()?;
                Ok(expr)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{}", other),
                position: self.position,
            }),
        }?;

        // Check for type conversion: <expr> as text/number/truth
        if self.is_keyword("as") {
            self.advance();
            let type_keyword = match self.peek() {
                Token::Keyword(k) if k == "text" || k == "number" || k == "truth" => k.clone(),
                Token::Identifier(name) if name == "text" || name == "number" || name == "truth" => {
                    name.clone()
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "text, number, or truth".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            };
            self.advance();

            let op = match type_keyword.as_str() {
                "text" => BuiltInUnaryOp::AsText,
                "number" => BuiltInUnaryOp::AsNumber,
                "truth" => BuiltInUnaryOp::AsTruth,
                _ => unreachable!(),
            };

            return Ok(Node::BuiltInUnary {
                op,
                operand: Box::new(result),
            });
        }

        // Check for dictionary field access: expr.field
        if matches!(self.peek(), Token::Dot) {
            self.advance(); // consume .
            let field = match self.peek() {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "field name".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            };
            return Ok(Node::DictAccess {
                dict: Box::new(result),
                field,
            });
        }

        Ok(result)
    }

    /// Parse a primary expression WITHOUT type conversion (for range loop bounds).
    fn parse_primary_no_type_conv(&mut self) -> Result<Node, ParseError> {
        // Handle interpolated strings
        if let Token::StringLiteral(_) = self.peek() {
            return self.parse_interpolated_string();
        }

        // Handle dictionary literal
        if let Token::LeftBrace = self.peek() {
            return self.parse_dict_literal();
        }

        let result = match self.peek() {
            Token::NumberLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Node::NumberLit(n))
            }
            Token::Keyword(k) if k == "true" => {
                self.advance();
                Ok(Node::BoolLit(true))
            }
            Token::Keyword(k) if k == "false" => {
                self.advance();
                Ok(Node::BoolLit(false))
            }
            Token::Keyword(k) if k == "list" => {
                self.advance();
                self.parse_list_literal()
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RightParen) {
                        args.push(self.parse_expression()?);
                        while let Token::Comma = self.peek() {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect_right_paren()?;
                    Ok(Node::Call { name, args })
                } else {
                    Ok(Node::Identifier(name))
                }
            }
            Token::Keyword(kw) if kw == "text" || kw == "number" || kw == "truth"
                || kw == "uppercase" || kw == "lowercase" || kw == "length" || kw == "trim"
                || kw == "reverse" || kw == "round" || kw == "floor" || kw == "ceiling"
                || kw == "absolute" || kw == "random" || kw == "between"
                || kw == "count" || kw == "first" || kw == "last" || kw == "sort"
                || kw == "push" || kw == "pop" || kw == "from"
                || kw == "replace" || kw == "with" || kw == "in" || kw == "of"
                || kw == "contains" =>
            {
                let name = kw.clone();
                self.advance();
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RightParen) {
                        args.push(self.parse_expression()?);
                        while let Token::Comma = self.peek() {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect_right_paren()?;
                    Ok(Node::Call { name, args })
                } else {
                    Ok(Node::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_right_paren()?;
                Ok(expr)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: format!("{}", other),
                position: self.position,
            }),
        }?;

        // NO type conversion check here - that's the key difference from parse_primary

        // Check for dictionary field access: expr.field
        if matches!(self.peek(), Token::Dot) {
            self.advance();
            let field = match self.peek() {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "field name".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            };
            return Ok(Node::DictAccess {
                dict: Box::new(result),
                field,
            });
        }

        Ok(result)
    }

    /// Parse an interpolated string: sequence of StringLiteral and StringInterpolation tokens.
    fn parse_interpolated_string(&mut self) -> Result<Node, ParseError> {
        let mut parts: Vec<crate::ast::InterpPart> = Vec::new();

        loop {
            match self.peek() {
                Token::StringLiteral(s) => {
                    let s = s.clone();
                    self.advance();
                    if !s.is_empty() {
                        parts.push(crate::ast::InterpPart::Lit(s));
                    }
                }
                Token::StringInterpolation(var) => {
                    let var = var.clone();
                    self.advance();
                    parts.push(crate::ast::InterpPart::Var(var));
                }
                _ => break,
            }
        }

        // If there's only one Lit part and no Var parts, return as StringLit for backwards compat
        if parts.len() == 1 {
            if let crate::ast::InterpPart::Lit(s) = &parts[0] {
                return Ok(Node::StringLit(s.clone()));
            }
        }

        Ok(Node::InterpolatedString(parts))
    }

    /// Parse a dictionary literal: { key: value, key2: value2, ... }
    fn parse_dict_literal(&mut self) -> Result<Node, ParseError> {
        self.advance(); // consume {
        let mut entries = Vec::new();

        // Parse comma-separated key: value pairs
        loop {
            // Check for closing brace
            if matches!(self.peek(), Token::RightBrace) {
                self.advance();
                break;
            }

            // Parse key (must be an identifier or string)
            let key = match self.peek() {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                Token::StringLiteral(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "dictionary key (identifier or string)".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            };

            // Expect colon
            match self.peek() {
                Token::Colon => {
                    self.advance();
                }
                other => {
                    return Err(ParseError::UnexpectedToken {
                        expected: ":".to_string(),
                        found: format!("{}", other),
                        position: self.position,
                    });
                }
            }

            // Parse value
            let value = self.parse_expression()?;
            entries.push((key, value));

            // Check for comma or end
            if matches!(self.peek(), Token::Comma) {
                self.advance();
            }
        }

        Ok(Node::DictLiteral(entries))
    }

    /// Parse a list literal: [ expr, expr, ... ]
    fn parse_list_literal(&mut self) -> Result<Node, ParseError> {
        // Check for opening bracket - we use keyword "list" so next should be [ or expression
        // Actually, let's use parentheses for list: list(expr, expr)
        // Or we can just parse comma-separated expressions after "list"
        let mut items = Vec::new();
        // Check if next is not a keyword that would start a new statement
        if !matches!(
            self.peek(),
            Token::Keyword(_) | Token::Newline | Token::EOF | Token::RightBrace
        ) {
            items.push(self.parse_expression()?);
            while let Token::Comma = self.peek() {
                self.advance();
                items.push(self.parse_expression()?);
            }
        }
        Ok(Node::ListLiteral(items))
    }

    /// Expect and consume a right parenthesis.
    fn expect_right_paren(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Token::RightParen => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::UnexpectedToken {
                expected: ")".to_string(),
                found: format!("{}", other),
                position: self.position,
            }),
        }
    }

    /// Expect and consume a word as an identifier (accepts both Identifier and Keyword tokens).
    fn expect_word_as_identifier(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Token::Keyword(k) => {
                // Accept keywords as parameter names
                let name = k.clone();
                self.advance();
                Ok(name)
            }
            other => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: format!("{}", other),
                position: self.position,
            }),
        }
    }
}

/// Find the byte position of a token in source text, starting search from `search_from`.
fn find_token_position(token: &Token, source: &str, search_from: usize) -> usize {
    let needle = match token {
        Token::Keyword(s) => s.as_str(),
        Token::Identifier(s) => s.as_str(),
        Token::StringLiteral(s) => {
            // Search for the quoted string
            let quoted = format!("\"{}\"", s);
            if let Some(pos) = source[search_from..].find(&quoted) {
                return search_from + pos;
            }
            // Fallback: try without quotes
            s.as_str()
        }
        Token::NumberLiteral(n) => {
            let _s = if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            };
            // Use a leaky string for static - or just search
            // We'll scan manually
            let search = &source[search_from..];
            for (i, _) in search.char_indices() {
                if let Ok(num) = search[i..].parse::<f64>() {
                    if (num - n).abs() < f64::EPSILON {
                        return search_from + i;
                    }
                }
            }
            return search_from;
        }
        Token::Operator(s) => s.as_str(),
        Token::Newline => {
            // Find next newline
            if let Some(pos) = source[search_from..].find('\n') {
                return search_from + pos;
            }
            return search_from;
        }
        Token::EOF => return source.len().saturating_sub(1),
        Token::LeftParen => "(",
        Token::RightParen => ")",
        Token::LeftBrace => "{",
        Token::RightBrace => "}",
        Token::Comma => ",",
        Token::Dot => ".",
        Token::Colon => ":",
        Token::LeftBracket => "[",
        Token::RightBracket => "]",
        Token::StringInterpolation(s) => s.as_str(),
    };

    if let Some(pos) = source[search_from..].find(needle) {
        search_from + pos
    } else {
        search_from
    }
}

/// Get line info for a byte position.
fn get_line_info(source: &str, byte_pos: usize) -> (usize, usize, String) {
    let up_to_pos = &source[..byte_pos.min(source.len())];
    let line_num = up_to_pos.lines().count();
    let line_content = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("").to_string();
    let line_start = source.lines().take(line_num.saturating_sub(1)).map(|l| l.len() + 1).sum::<usize>();
    let col = byte_pos.saturating_sub(line_start) + 1;
    (line_num, col, line_content)
}

#[cfg(test)]
mod tests;
