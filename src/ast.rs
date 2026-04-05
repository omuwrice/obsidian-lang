use std::fmt;

/// Source position tracking for error messages.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourcePosition {
    /// 1-based line number
    pub line: usize,
    /// 1-based column number
    pub column: usize,
    /// The full line of source code
    pub line_content: String,
}

impl SourcePosition {
    pub fn new(line: usize, column: usize, line_content: String) -> Self {
        Self { line, column, line_content }
    }
}

/// Unary built-in operations (single operand).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInUnaryOp {
    // Text operations
    Uppercase,
    Lowercase,
    Length,
    Trim,
    Reverse,
    // Number operations
    Round,
    Floor,
    Ceiling,
    Absolute,
    Sqrt,
    // Date/Time operations
    CurrentDate,
    CurrentTime,
    // File operations
    FileExists,
    ListFiles,
    // List operations
    Count,
    First,
    Last,
    Sort,
    // Type conversions
    AsText,
    AsNumber,
    AsTruth,
}

/// Binary built-in operations (two operands).
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInBinaryOp {
    // "x contains y" - check if x contains y
    Contains,
    // "push x to y" - append x to list y
    PushTo,
    // "pop from x" - remove last element from list x
    PopFrom,
    // "random between x and y" - random number in range
    RandomBetween,
    // "join x with y" - join list x with separator y
    JoinWith,
    // "power x by y" - x raised to power y
    Power,
    // "split x by y" - split string x by delimiter y
    Split,
}

/// Ternary built-in operations (three operands).
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInTernaryOp {
    // "replace x with y in z" - string replacement
    ReplaceIn,
}

/// Nullary built-in operations (no operands).
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInNullaryOp {
    CurrentDate,
    CurrentTime,
}

/// Represents a node in the Abstract Syntax Tree (AST).
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Program(Vec<Node>),
    Set {
        name: String,
        value: Box<Node>,
        pos: Option<SourcePosition>,
    },
    Show(Box<Node>),
    Ask {
        prompt: Box<Node>,
        into: String,
    },
    CreateFile(Box<Node>),
    DeleteFile(Box<Node>),
    ReadFile {
        path: Box<Node>,
        into: String,
    },
    WriteFile {
        path: Box<Node>,
        content: Box<Node>,
    },
    AppendFile {
        path: Box<Node>,
        content: Box<Node>,
    },
    CopyFile {
        from: Box<Node>,
        to: Box<Node>,
    },
    RenameFile {
        from: Box<Node>,
        to: Box<Node>,
    },
    If {
        condition: Box<Node>,
        body: Vec<Node>,
        otherwise: Option<Vec<Node>>,
    },
    Repeat {
        times: Box<Node>,
        body: Vec<Node>,
        var_name: Option<String>,
    },
    RepeatRange {
        start: Box<Node>,
        end: Box<Node>,
        var_name: String,
        body: Vec<Node>,
    },
    While {
        condition: Box<Node>,
        body: Vec<Node>,
    },
    Define {
        name: String,
        params: Vec<String>,
        body: Vec<Node>,
    },
    Call {
        name: String,
        args: Vec<Node>,
    },
    Return(Box<Node>),
    BinaryOp {
        left: Box<Node>,
        op: String,
        right: Box<Node>,
    },
    /// Unary built-in: `uppercase of x`, `round x`, `x as text`
    BuiltInUnary {
        op: BuiltInUnaryOp,
        operand: Box<Node>,
    },
    /// Binary built-in: `x contains y`, `push x to y`, `random between x and y`
    BuiltInBinary {
        op: BuiltInBinaryOp,
        left: Box<Node>,
        right: Box<Node>,
    },
    /// Ternary built-in: `replace x with y in z`
    BuiltInTernary {
        op: BuiltInTernaryOp,
        first: Box<Node>,
        second: Box<Node>,
        third: Box<Node>,
    },
    /// Nullary built-in: `current date`, `current time`
    BuiltInNullary {
        op: BuiltInNullaryOp,
    },
    Identifier(String),
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    ListLiteral(Vec<Node>),
    /// String interpolation: "Hello {name}, you are {age}"
    InterpolatedString(Vec<InterpPart>),
    /// Dictionary literal: { name: "sidd", age: 20 }
    DictLiteral(Vec<(String, Node)>),
    /// Dictionary field access: person.name
    DictAccess {
        dict: Box<Node>,
        field: String,
    },
    Exit,
    /// Break out of a loop
    Break,
    /// Continue to next iteration of a loop
    Continue,
    /// Try/catch error handling
    Try {
        body: Vec<Node>,
        catch_var: String,
        catch_body: Vec<Node>,
    },
    /// Return value from a node (used internally for control flow)
    #[allow(dead_code)]
    ReturnVal(Box<Node>),
    /// Test block: test "name" ... end
    TestBlock {
        name: String,
        body: Vec<Node>,
    },
    /// Expect assertion: expect <expr> is <expr>
    Expect {
        left: Box<Node>,
        right: Box<Node>,
        pos: Option<SourcePosition>,
    },
}

/// A part of an interpolated string.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpPart {
    /// A literal string segment
    Lit(String),
    /// A variable to interpolate
    Var(String),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Program(nodes) => {
                for node in nodes {
                    writeln!(f, "{}", node)?;
                }
                Ok(())
            }
            Node::Set { name, value, .. } => write!(f, "Set({}, {})", name, value),
            Node::Show(expr) => write!(f, "Show({})", expr),
            Node::Ask { prompt, into } => write!(f, "Ask({}, {})", prompt, into),
            Node::CreateFile(path) => write!(f, "CreateFile({})", path),
            Node::DeleteFile(path) => write!(f, "DeleteFile({})", path),
            Node::ReadFile { path, into } => write!(f, "ReadFile({}, {})", path, into),
            Node::WriteFile { path, content } => write!(f, "WriteFile({}, {})", path, content),
            Node::AppendFile { path, content } => write!(f, "AppendFile({}, {})", path, content),
            Node::CopyFile { from, to } => write!(f, "CopyFile({}, {})", from, to),
            Node::RenameFile { from, to } => write!(f, "RenameFile({}, {})", from, to),
            Node::If {
                condition,
                body,
                otherwise,
            } => {
                write!(f, "If({}, [", condition)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "], ")?;
                match otherwise {
                    Some(nodes) => {
                        write!(f, "[")?;
                        for node in nodes {
                            write!(f, "{}, ", node)?;
                        }
                        write!(f, "]")?;
                    }
                    None => write!(f, "None")?,
                }
                write!(f, ")")
            }
            Node::Repeat { times, body, var_name } => {
                write!(f, "Repeat({}, {:?}, [", times, var_name)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::RepeatRange { start, end, var_name, body } => {
                write!(f, "RepeatRange({}, {}, {}, [", start, end, var_name)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::While { condition, body } => {
                write!(f, "While({}, [", condition)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::Define {
                name,
                params,
                body,
            } => {
                write!(f, "Define({}, {:?}, [", name, params)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::Call { name, args } => {
                write!(f, "Call({}, [", name)?;
                for node in args {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::Return(expr) => write!(f, "Return({})", expr),
            Node::BinaryOp { left, op, right } => {
                write!(f, "BinaryOp({}, {}, {})", left, op, right)
            }
            Node::BuiltInUnary { op, operand } => {
                write!(f, "BuiltInUnary({:?}, {})", op, operand)
            }
            Node::BuiltInBinary { op, left, right } => {
                write!(f, "BuiltInBinary({:?}, {}, {})", op, left, right)
            }
            Node::BuiltInTernary { op, first, second, third } => {
                write!(f, "BuiltInTernary({:?}, {}, {}, {})", op, first, second, third)
            }
            Node::BuiltInNullary { op } => {
                write!(f, "BuiltInNullary({:?})", op)
            }
            Node::Identifier(s) => write!(f, "Identifier({})", s),
            Node::StringLit(s) => write!(f, "StringLit({})", s),
            Node::NumberLit(n) => write!(f, "NumberLit({})", n),
            Node::BoolLit(b) => write!(f, "BoolLit({})", b),
            Node::ListLiteral(nodes) => {
                write!(f, "ListLiteral([")?;
                for node in nodes {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::InterpolatedString(parts) => {
                write!(f, "InterpolatedString([")?;
                for part in parts {
                    match part {
                        InterpPart::Lit(s) => write!(f, "Lit({}), ", s)?,
                        InterpPart::Var(v) => write!(f, "Var({}), ", v)?,
                    }
                }
                write!(f, "])")
            }
            Node::DictLiteral(entries) => {
                write!(f, "DictLiteral([")?;
                for (k, v) in entries {
                    write!(f, "{}: {}, ", k, v)?;
                }
                write!(f, "])")
            }
            Node::DictAccess { dict, field } => {
                write!(f, "DictAccess({}, {})", dict, field)
            }
            Node::Exit => write!(f, "Exit"),
            Node::Break => write!(f, "Break"),
            Node::Continue => write!(f, "Continue"),
            Node::Try { body, catch_var, catch_body } => {
                write!(f, "Try([", )?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "], {}, [", catch_var)?;
                for node in catch_body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::ReturnVal(expr) => write!(f, "ReturnVal({})", expr),
            Node::TestBlock { name, body } => {
                write!(f, "TestBlock({}, [", name)?;
                for node in body {
                    write!(f, "{}, ", node)?;
                }
                write!(f, "])")
            }
            Node::Expect { left, right, .. } => {
                write!(f, "Expect({}, {})", left, right)
            }
        }
    }
}
