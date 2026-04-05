use crate::ast::{Node, BuiltInUnaryOp, BuiltInBinaryOp, BuiltInTernaryOp, BuiltInNullaryOp};
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::rc::Rc;

// ============================================================================
// Loop Control Flow
// ============================================================================

/// Control flow signal for loop execution (break/continue).
#[allow(dead_code)]
enum LoopControl {
    None,
    Break,
    Continue,
}

// ============================================================================
// Value Enum
// ============================================================================

/// Represents runtime values in the Obsidian language.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    Bool(bool),
    List(Vec<Value>),
    Dict(HashMap<String, Value>),
    Null,
    Function {
        params: Vec<String>,
        body: Vec<Node>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Text(s) => write!(f, "{}", s),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Dict(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Null => write!(f, "null"),
            Value::Function { .. } => write!(f, "<function>"),
        }
    }
}

// ============================================================================
// ObsidianError
// ============================================================================

/// Friendly error messages for runtime errors.
#[derive(Clone, Debug)]
pub enum ObsidianError {
    UndefinedVariable { name: String },
    TypeMismatch { message: String },
    FileError { message: String },
    RuntimeError { message: String },
    /// Used for control flow - unwinds the call stack with a return value.
    Return { value: Option<Value> },
    /// Used for loop break/continue control flow.
    Break,
    Continue,
}

impl fmt::Display for ObsidianError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObsidianError::UndefinedVariable { name } => {
                write!(f, "I couldn't find a variable named '{}'", name)
            }
            ObsidianError::TypeMismatch { message } => {
                write!(f, "Type error: {}", message)
            }
            ObsidianError::FileError { message } => write!(f, "File error: {}", message),
            ObsidianError::RuntimeError { message } => write!(f, "Error: {}", message),
            ObsidianError::Return { .. } => write!(f, "return"),
            ObsidianError::Break => write!(f, "break"),
            ObsidianError::Continue => write!(f, "continue"),
        }
    }
}


// ============================================================================
// Environment
// ============================================================================

/// An environment with lexical scoping using a parent chain.
#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// Create a new top-level environment.
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    /// Create a child environment with the given parent.
    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Define a variable in the current scope.
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    /// Look up a variable, searching the parent chain.
    pub fn get(&self, name: &str) -> Result<Value, ObsidianError> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        Err(ObsidianError::UndefinedVariable {
            name: name.to_string(),
        })
    }

    /// Assign to an existing variable (searches parent chain).
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), ObsidianError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(ObsidianError::UndefinedVariable {
            name: name.to_string(),
        })
    }
}

// ============================================================================
// Call Frame
// ============================================================================

struct CallFrame;

// ============================================================================
// Interpreter
// ============================================================================

/// The tree-walking interpreter for Obsidian.
pub struct Interpreter {
    pub(crate) environment: Rc<RefCell<Environment>>,
    call_stack: Vec<CallFrame>,
}

impl Interpreter {
    /// Create a new Interpreter with a fresh top-level environment.
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
            call_stack: Vec::new(),
        }
    }

    /// Create a new Interpreter sharing the given environment.
    pub fn with_environment(environment: Rc<RefCell<Environment>>) -> Self {
        Interpreter {
            environment,
            call_stack: Vec::new(),
        }
    }

    /// Execute a Program AST node.
    pub fn execute(&mut self, program: &Node) -> Result<Option<Value>, ObsidianError> {
        match program {
            Node::Program(statements) => {
                let mut last_value = None;
                for stmt in statements {
                    last_value = self.execute_statement(stmt)?;
                }
                Ok(last_value)
            }
            other => self.execute_statement(other),
        }
    }

    /// Execute a single statement node.
    fn execute_statement(&mut self, node: &Node) -> Result<Option<Value>, ObsidianError> {
        match node {
            Node::Set { name, value, .. } => {
                let val = self.evaluate(value)?;
                self.environment.borrow_mut().define(name.clone(), val);
                Ok(None)
            }

            Node::Show(expr) => {
                let val = self.evaluate(expr)?;
                println!("{}", val);
                Ok(None)
            }

            Node::Ask { prompt, into } => {
                let prompt_val = self.evaluate(prompt)?;
                print!("{}", prompt_val);
                io::stdout().flush().map_err(|e| ObsidianError::RuntimeError {
                    message: format!("I couldn't print the prompt: {}", e),
                })?;

                let stdin = io::stdin();
                let mut line = String::new();
                stdin.lock().read_line(&mut line).map_err(|e| {
                    ObsidianError::RuntimeError {
                        message: format!("I couldn't read your input: {}", e),
                    }
                })?;
                // Remove trailing newline
                if line.ends_with('\n') {
                    line.pop();
                }
                if line.ends_with('\r') {
                    line.pop();
                }
                self.environment
                    .borrow_mut()
                    .define(into.clone(), Value::Text(line));
                Ok(None)
            }

            Node::CreateFile(path_node) => {
                let path_val = self.evaluate(path_node)?;
                let path_str = value_to_string(&path_val)?;
                std::fs::File::create(&path_str).map_err(|e| ObsidianError::FileError {
                    message: format!(
                        "I couldn't create a file named '{}': {}",
                        path_str, e
                    ),
                })?;
                Ok(None)
            }

            Node::DeleteFile(path_node) => {
                let path_val = self.evaluate(path_node)?;
                let path_str = value_to_string(&path_val)?;
                std::fs::remove_file(&path_str).map_err(|e| ObsidianError::FileError {
                    message: format!(
                        "I couldn't delete the file named '{}': {}",
                        path_str, e
                    ),
                })?;
                Ok(None)
            }

            Node::ReadFile { path, into } => {
                let path_val = self.evaluate(path)?;
                let path_str = value_to_string(&path_val)?;
                let content = std::fs::read_to_string(&path_str).map_err(|e| {
                    ObsidianError::FileError {
                        message: format!(
                            "I couldn't find a file named '{}': {}",
                            path_str, e
                        ),
                    }
                })?;
                self.environment
                    .borrow_mut()
                    .define(into.clone(), Value::Text(content));
                Ok(None)
            }

            Node::WriteFile { path, content } => {
                let path_val = self.evaluate(path)?;
                let path_str = value_to_string(&path_val)?;
                let content_val = self.evaluate(content)?;
                let content_str = value_to_string(&content_val)?;
                std::fs::write(&path_str, &content_str).map_err(|e| ObsidianError::FileError {
                    message: format!(
                        "I couldn't write to a file named '{}': {}",
                        path_str, e
                    ),
                })?;
                Ok(None)
            }

            Node::AppendFile { path, content } => {
                let path_val = self.evaluate(path)?;
                let path_str = value_to_string(&path_val)?;
                let content_val = self.evaluate(content)?;
                let content_str = value_to_string(&content_val)?;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path_str)
                    .map_err(|e| ObsidianError::FileError {
                        message: format!(
                            "I couldn't open a file named '{}' for appending: {}",
                            path_str, e
                        ),
                    })?;
                file.write_all(content_str.as_bytes())
                    .map_err(|e| ObsidianError::FileError {
                        message: format!(
                            "I couldn't append to a file named '{}': {}",
                            path_str, e
                        ),
                    })?;
                Ok(None)
            }

            Node::CopyFile { from, to } => {
                let from_val = self.evaluate(from)?;
                let from_str = value_to_string(&from_val)?;
                let to_val = self.evaluate(to)?;
                let to_str = value_to_string(&to_val)?;
                std::fs::copy(&from_str, &to_str).map_err(|e| ObsidianError::FileError {
                    message: format!(
                        "I couldn't copy the file '{}' to '{}': {}",
                        from_str, to_str, e
                    ),
                })?;
                Ok(None)
            }

            Node::RenameFile { from, to } => {
                let from_val = self.evaluate(from)?;
                let from_str = value_to_string(&from_val)?;
                let to_val = self.evaluate(to)?;
                let to_str = value_to_string(&to_val)?;
                std::fs::rename(&from_str, &to_str).map_err(|e| ObsidianError::FileError {
                    message: format!(
                        "I couldn't rename '{}' to '{}': {}",
                        from_str, to_str, e
                    ),
                })?;
                Ok(None)
            }

            Node::If {
                condition,
                body,
                otherwise,
            } => {
                let cond_val = self.evaluate(condition)?;
                if is_truthy(&cond_val) {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(otherwise_body) = otherwise {
                    for stmt in otherwise_body {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(None)
            }

            Node::Repeat { times, body, var_name } => {
                let times_val = self.evaluate(times)?;
                let n = value_to_number(&times_val)?;
                let count = n as usize;
                
                for i in 0..count {
                    // If a loop variable name is specified, define it
                    if let Some(var) = var_name {
                        self.environment.borrow_mut().define(var.clone(), Value::Number(i as f64));
                    }
                    
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(ObsidianError::Break) => return Ok(None),
                            Err(ObsidianError::Continue) => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(None)
            }

            Node::RepeatRange { start, end, var_name, body } => {
                let start_val = value_to_number(&self.evaluate(start)?)?;
                let end_val = value_to_number(&self.evaluate(end)?)?;
                let start_i = start_val as i64;
                let end_i = end_val as i64;
                
                for i in start_i..=end_i {
                    self.environment.borrow_mut().define(var_name.clone(), Value::Number(i as f64));
                    
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(ObsidianError::Break) => return Ok(None),
                            Err(ObsidianError::Continue) => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(None)
            }

            Node::While { condition, body } => {
                loop {
                    let cond_val = self.evaluate(condition)?;
                    if !is_truthy(&cond_val) {
                        break;
                    }
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(ObsidianError::Break) => return Ok(None),
                            Err(ObsidianError::Continue) => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(None)
            }

            Node::Define {
                name,
                params,
                body,
            } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.environment
                    .borrow_mut()
                    .define(name.clone(), func);
                Ok(None)
            }

            Node::Call { name, args } => {
                let result = self.call_function(name, args)?;
                Ok(Some(result))
            }

            Node::Return(expr) => {
                let val = self.evaluate(expr)?;
                Err(ObsidianError::Return { value: Some(val) })
            }

            Node::Exit => {
                std::process::exit(0);
            }

            Node::Break => {
                return Err(ObsidianError::Break);
            }

            Node::Continue => {
                return Err(ObsidianError::Continue);
            }

            // TestBlock: skip during normal execution (handled by test runner)
            Node::TestBlock { .. } => {
                Ok(None)
            }

            // Expect assertion: check left == right, error if not equal
            Node::Expect { left, right, .. } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                if left_val != right_val {
                    let msg = format!(
                        "Assertion failed: expected {}, but got {}",
                        right_val, left_val
                    );
                    Err(ObsidianError::RuntimeError { message: msg })
                } else {
                    Ok(None)
                }
            }

            Node::Try { body, catch_var, catch_body } => {
                // Try to execute the body
                let saved_env = Rc::clone(&self.environment);
                let result = self.execute_body(body);
                
                match result {
                    Ok(val) => Ok(val),
                    Err(ObsidianError::FileError { message }) |
                    Err(ObsidianError::RuntimeError { message }) |
                    Err(ObsidianError::TypeMismatch { message }) |
                    Err(ObsidianError::UndefinedVariable { name: message }) => {
                        // Restore environment and set up catch scope
                        self.environment = saved_env;
                        // Define the error variable
                        self.environment.borrow_mut().define(catch_var.clone(), Value::Text(message));
                        // Execute catch body
                        self.execute_body(catch_body)
                    }
                    Err(e) => Err(e),
                }
            }

            // push/pop as statements that modify variables in place
            Node::BuiltInBinary { op: BuiltInBinaryOp::PushTo, left, right } => {
                let value = self.evaluate(left)?;
                let list_val = self.evaluate(right)?;
                match list_val {
                    Value::List(mut items) => {
                        items.push(value);
                        // If the right side is an identifier, update the variable
                        if let Node::Identifier(name) = right.as_ref() {
                            self.environment.borrow_mut().assign(name, Value::List(items))?;
                        }
                        Ok(None)
                    }
                    _ => Err(ObsidianError::TypeMismatch {
                        message: format!("I expected a list, but got {}", value_type_name(&list_val)),
                    }),
                }
            }
            Node::BuiltInBinary { op: BuiltInBinaryOp::PopFrom, left, right: _ } => {
                let list_val = self.evaluate(left)?;
                match list_val {
                    Value::List(mut items) => {
                        if items.is_empty() {
                            return Err(ObsidianError::RuntimeError {
                                message: "I can't pop from an empty list".to_string(),
                            });
                        }
                        items.pop();
                        // If the left side is an identifier, update the variable
                        if let Node::Identifier(name) = left.as_ref() {
                            self.environment.borrow_mut().assign(name, Value::List(items))?;
                        }
                        Ok(None)
                    }
                    _ => Err(ObsidianError::TypeMismatch {
                        message: format!("I expected a list, but got {}", value_type_name(&list_val)),
                    }),
                }
            }

            // Expression nodes can appear as statements
            Node::BinaryOp { .. }
            | Node::Identifier(_)
            | Node::StringLit(_)
            | Node::NumberLit(_)
            | Node::BoolLit(_)
            | Node::ListLiteral(_)
            | Node::InterpolatedString(_)
            | Node::DictLiteral(_)
            | Node::DictAccess { .. }
            | Node::BuiltInUnary { .. }
            | Node::BuiltInBinary { .. }
            | Node::BuiltInTernary { .. }
            | Node::BuiltInNullary { .. } => {
                self.evaluate(node)?;
                Ok(None)
            }

            other => Err(ObsidianError::RuntimeError {
                message: format!("I don't know how to execute: {:?}", other),
            }),
        }
    }

    /// Execute a list of statements, returning the first Return error encountered.
    fn execute_body(&mut self, body: &[Node]) -> Result<Option<Value>, ObsidianError> {
        let mut last_value = None;
        for stmt in body {
            last_value = self.execute_statement(stmt)?;
        }
        Ok(last_value)
    }

    /// Evaluate an expression node to a Value.
    fn evaluate(&mut self, node: &Node) -> Result<Value, ObsidianError> {
        match node {
            Node::StringLit(s) => Ok(Value::Text(s.clone())),
            Node::NumberLit(n) => Ok(Value::Number(*n)),
            Node::BoolLit(b) => Ok(Value::Bool(*b)),
            Node::ListLiteral(items) => {
                let values: Vec<Value> = items
                    .iter()
                    .map(|item| self.evaluate(item))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Value::List(values))
            }
            Node::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        crate::ast::InterpPart::Lit(s) => result.push_str(s),
                        crate::ast::InterpPart::Var(var_name) => {
                            // Support dot-notation field access: "person.name" or "person.address.city"
                            let mut parts_iter = var_name.split('.');
                            let base_name = match parts_iter.next() {
                                Some(n) => n,
                                None => {
                                    return Err(ObsidianError::RuntimeError {
                                        message: "Empty variable name in interpolation".to_string(),
                                    });
                                }
                            };
                            let mut val = self.environment.borrow().get(base_name)?;
                            for field in parts_iter {
                                match val {
                                    Value::Dict(entries) => {
                                        if let Some(v) = entries.get(field) {
                                            val = v.clone();
                                        } else {
                                            return Err(ObsidianError::RuntimeError {
                                                message: format!(
                                                    "I couldn't find a field named '{}' in the dictionary",
                                                    field
                                                ),
                                            });
                                        }
                                    }
                                    _ => {
                                        return Err(ObsidianError::TypeMismatch {
                                            message: format!(
                                                "I expected a dictionary, but got {}",
                                                value_type_name(&val)
                                            ),
                                        });
                                    }
                                }
                            }
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::Text(result))
            }
            Node::DictLiteral(entries) => {
                let mut dict = HashMap::new();
                for (key, value_node) in entries {
                    let value = self.evaluate(value_node)?;
                    dict.insert(key.clone(), value);
                }
                Ok(Value::Dict(dict))
            }
            Node::DictAccess { dict, field } => {
                let dict_val = self.evaluate(dict)?;
                match dict_val {
                    Value::Dict(entries) => {
                        if let Some(value) = entries.get(field) {
                            Ok(value.clone())
                        } else {
                            Err(ObsidianError::RuntimeError {
                                message: format!("I couldn't find a field named '{}' in the dictionary", field),
                            })
                        }
                    }
                    _ => Err(ObsidianError::TypeMismatch {
                        message: format!("I expected a dictionary, but got {}", value_type_name(&dict_val)),
                    }),
                }
            }
            Node::Identifier(name) => self.environment.borrow().get(name),

            Node::BinaryOp { left, op, right } => {
                // Handle 'not' as unary (left is ignored, it's BoolLit(true) from parser)
                if op == "not" {
                    let operand = self.evaluate(right)?;
                    return Ok(Value::Bool(!is_truthy(&operand)));
                }

                // Handle 'and' with short-circuit
                if op == "and" {
                    let left_val = self.evaluate(left)?;
                    if !is_truthy(&left_val) {
                        return Ok(Value::Bool(false));
                    }
                    let right_val = self.evaluate(right)?;
                    return Ok(Value::Bool(is_truthy(&right_val)));
                }

                // Handle 'or' with short-circuit
                if op == "or" {
                    let left_val = self.evaluate(left)?;
                    if is_truthy(&left_val) {
                        return Ok(Value::Bool(true));
                    }
                    let right_val = self.evaluate(right)?;
                    return Ok(Value::Bool(is_truthy(&right_val)));
                }

                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;

                match op.as_str() {
                    "+" => {
                        match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                            (Value::Text(a), Value::Text(b)) => {
                                Ok(Value::Text(format!("{}{}", a, b)))
                            }
                            (Value::Text(a), b) => Ok(Value::Text(format!("{}{}", a, b))),
                            (a, Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!(
                                    "I can't add {} and {}",
                                    value_type_name(&left_val),
                                    value_type_name(&right_val)
                                ),
                            }),
                        }
                    }
                    "-" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Number(a - b))
                    }
                    "*" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Number(a * b))
                    }
                    "/" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        if b == 0.0 {
                            return Err(ObsidianError::RuntimeError {
                                message: "I can't divide by zero".to_string(),
                            });
                        }
                        Ok(Value::Number(a / b))
                    }
                    "==" | "is" => Ok(Value::Bool(left_val == right_val)),
                    "!=" => Ok(Value::Bool(left_val != right_val)),
                    ">" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Bool(a > b))
                    }
                    "<" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Bool(a < b))
                    }
                    ">=" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Bool(a >= b))
                    }
                    "<=" => {
                        let a = value_to_number(&left_val)?;
                        let b = value_to_number(&right_val)?;
                        Ok(Value::Bool(a <= b))
                    }
                    _ => Err(ObsidianError::RuntimeError {
                        message: format!("I don't know how to use the '{}' operator", op),
                    }),
                }
            }

            Node::Call { name, args } => self.call_function(name, args),

            // Built-in unary operations
            Node::BuiltInUnary { op, operand } => {
                let val = self.evaluate(operand)?;
                match op {
                    BuiltInUnaryOp::Uppercase => {
                        let s = value_to_string(&val)?;
                        Ok(Value::Text(s.to_uppercase()))
                    }
                    BuiltInUnaryOp::Lowercase => {
                        let s = value_to_string(&val)?;
                        Ok(Value::Text(s.to_lowercase()))
                    }
                    BuiltInUnaryOp::Length => {
                        match &val {
                            Value::Text(s) => Ok(Value::Number(s.len() as f64)),
                            Value::List(items) => Ok(Value::Number(items.len() as f64)),
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected text or a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::Trim => {
                        let s = value_to_string(&val)?;
                        Ok(Value::Text(s.trim().to_string()))
                    }
                    BuiltInUnaryOp::Reverse => {
                        match &val {
                            Value::Text(s) => {
                                let reversed: String = s.chars().rev().collect();
                                Ok(Value::Text(reversed))
                            }
                            Value::List(items) => {
                                let mut reversed = items.clone();
                                reversed.reverse();
                                Ok(Value::List(reversed))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected text or a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::Round => {
                        let n = value_to_number(&val)?;
                        Ok(Value::Number(n.round()))
                    }
                    BuiltInUnaryOp::Floor => {
                        let n = value_to_number(&val)?;
                        Ok(Value::Number(n.floor()))
                    }
                    BuiltInUnaryOp::Ceiling => {
                        let n = value_to_number(&val)?;
                        Ok(Value::Number(n.ceil()))
                    }
                    BuiltInUnaryOp::Absolute => {
                        let n = value_to_number(&val)?;
                        Ok(Value::Number(n.abs()))
                    }
                    BuiltInUnaryOp::Count => {
                        match &val {
                            Value::List(items) => Ok(Value::Number(items.len() as f64)),
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::First => {
                        match &val {
                            Value::List(items) => {
                                if items.is_empty() {
                                    return Err(ObsidianError::RuntimeError {
                                        message: "I can't get the first element of an empty list".to_string(),
                                    });
                                }
                                Ok(items[0].clone())
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::Last => {
                        match &val {
                            Value::List(items) => {
                                if items.is_empty() {
                                    return Err(ObsidianError::RuntimeError {
                                        message: "I can't get the last element of an empty list".to_string(),
                                    });
                                }
                                Ok(items[items.len() - 1].clone())
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::Sort => {
                        match &val {
                            Value::List(items) => {
                                let mut sorted = items.clone();
                                sorted.sort_by(|a, b| value_cmp(a, b));
                                Ok(Value::List(sorted))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::AsText => {
                        Ok(Value::Text(val.to_string()))
                    }
                    BuiltInUnaryOp::AsNumber => {
                        match &val {
                            Value::Number(n) => Ok(Value::Number(*n)),
                            Value::Text(s) => {
                                s.parse::<f64>().map(Value::Number).map_err(|_| ObsidianError::TypeMismatch {
                                    message: format!("I couldn't convert '{}' to a number", s),
                                })
                            }
                            Value::Bool(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I couldn't convert {} to a number", value_type_name(&val)),
                            }),
                        }
                    }
                    BuiltInUnaryOp::AsTruth => {
                        Ok(Value::Bool(is_truthy(&val)))
                    }
                    BuiltInUnaryOp::Sqrt => {
                        let n = value_to_number(&val)?;
                        if n < 0.0 {
                            return Err(ObsidianError::RuntimeError {
                                message: "I can't take the square root of a negative number".to_string(),
                            });
                        }
                        Ok(Value::Number(n.sqrt()))
                    }
                    BuiltInUnaryOp::FileExists => {
                        let path = value_to_string(&val)?;
                        Ok(Value::Bool(std::path::Path::new(&path).exists()))
                    }
                    BuiltInUnaryOp::ListFiles => {
                        let path = value_to_string(&val)?;
                        let entries = std::fs::read_dir(&path).map_err(|e| ObsidianError::FileError {
                            message: format!("I couldn't list files in '{}': {}", path, e),
                        })?;
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                files.push(Value::Text(name.to_string()));
                            }
                        }
                        Ok(Value::List(files))
                    }
                    BuiltInUnaryOp::CurrentDate => {
                        // This shouldn't be reached - CurrentDate is now a Nullary op
                        // But handle it gracefully
                        let now = chrono::Local::now();
                        Ok(Value::Text(now.format("%Y-%m-%d").to_string()))
                    }
                    BuiltInUnaryOp::CurrentTime => {
                        let now = chrono::Local::now();
                        Ok(Value::Text(now.format("%H:%M:%S").to_string()))
                    }
                }
            }

            // Built-in binary operations
            Node::BuiltInBinary { op, left, right } => {
                match op {
                    BuiltInBinaryOp::Contains => {
                        let container = self.evaluate(left)?;
                        let item = self.evaluate(right)?;
                        match &container {
                            Value::Text(s) => {
                                let search = value_to_string(&item)?;
                                Ok(Value::Bool(s.contains(&search)))
                            }
                            Value::List(items) => {
                                Ok(Value::Bool(items.contains(&item)))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected text or a list, but got {}", value_type_name(&container)),
                            }),
                        }
                    }
                    BuiltInBinaryOp::PushTo => {
                        let value = self.evaluate(left)?;
                        let list_val = self.evaluate(right)?;
                        match list_val {
                            Value::List(mut items) => {
                                items.push(value);
                                Ok(Value::List(items))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&list_val)),
                            }),
                        }
                    }
                    BuiltInBinaryOp::PopFrom => {
                        let list_val = self.evaluate(left)?;
                        match list_val {
                            Value::List(mut items) => {
                                if items.is_empty() {
                                    return Err(ObsidianError::RuntimeError {
                                        message: "I can't pop from an empty list".to_string(),
                                    });
                                }
                                items.pop();
                                Ok(Value::List(items))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list, but got {}", value_type_name(&list_val)),
                            }),
                        }
                    }
                    BuiltInBinaryOp::RandomBetween => {
                        let low = value_to_number(&self.evaluate(left)?)?;
                        let high = value_to_number(&self.evaluate(right)?)?;
                        let mut rng = rand::thread_rng();
                        if low <= high {
                            let val = rng.gen_range(low..=high);
                            // Return integer if both bounds are integers
                            if low.fract() == 0.0 && high.fract() == 0.0 {
                                Ok(Value::Number(val as f64))
                            } else {
                                Ok(Value::Number(val))
                            }
                        } else {
                            let val = rng.gen_range(high..=low);
                            if low.fract() == 0.0 && high.fract() == 0.0 {
                                Ok(Value::Number(val as f64))
                            } else {
                                Ok(Value::Number(val))
                            }
                        }
                    }
                    BuiltInBinaryOp::Split => {
                        let text = value_to_string(&self.evaluate(left)?)?;
                        let delimiter = value_to_string(&self.evaluate(right)?)?;
                        let parts: Vec<Value> = text
                            .split(&delimiter)
                            .map(|s| Value::Text(s.to_string()))
                            .collect();
                        Ok(Value::List(parts))
                    }
                    BuiltInBinaryOp::JoinWith => {
                        let list_val = self.evaluate(left)?;
                        let separator = value_to_string(&self.evaluate(right)?)?;
                        match list_val {
                            Value::List(items) => {
                                let strings: Result<Vec<String>, _> = items
                                    .iter()
                                    .map(|item| value_to_string(item))
                                    .collect();
                                let strings = strings?;
                                Ok(Value::Text(strings.join(&separator)))
                            }
                            _ => Err(ObsidianError::TypeMismatch {
                                message: format!("I expected a list to join, but got {}", value_type_name(&list_val)),
                            }),
                        }
                    }
                    BuiltInBinaryOp::Power => {
                        let base = value_to_number(&self.evaluate(left)?)?;
                        let exponent = value_to_number(&self.evaluate(right)?)?;
                        Ok(Value::Number(base.powf(exponent)))
                    }
                }
            }

            // Built-in ternary operations
            Node::BuiltInTernary { op, first, second, third } => {
                match op {
                    BuiltInTernaryOp::ReplaceIn => {
                        let old = value_to_string(&self.evaluate(first)?)?;
                        let new = value_to_string(&self.evaluate(second)?)?;
                        let target = value_to_string(&self.evaluate(third)?)?;
                        Ok(Value::Text(target.replace(&old, &new)))
                    }
                }
            }

            // Built-in nullary operations
            Node::BuiltInNullary { op } => {
                match op {
                    BuiltInNullaryOp::CurrentDate => {
                        let now = chrono::Local::now();
                        Ok(Value::Text(now.format("%Y-%m-%d").to_string()))
                    }
                    BuiltInNullaryOp::CurrentTime => {
                        let now = chrono::Local::now();
                        Ok(Value::Text(now.format("%H:%M:%S").to_string()))
                    }
                }
            }

            other => Err(ObsidianError::RuntimeError {
                message: format!("I don't know how to evaluate: {:?}", other),
            }),
        }
    }

    /// Call a function by name with the given arguments.
    fn call_function(&mut self, name: &str, args: &[Node]) -> Result<Value, ObsidianError> {
        let func_val = self.environment.borrow().get(name)?;
        match func_val {
            Value::Function { params, body } => {
                if params.len() != args.len() {
                    return Err(ObsidianError::RuntimeError {
                        message: format!(
                            "I expected {} arguments for '{}', but got {}",
                            params.len(),
                            name,
                            args.len()
                        ),
                    });
                }

                // Evaluate arguments in the current scope
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                // Create a new scope for the function call
                let parent = Rc::clone(&self.environment);
                let mut child_env = Environment::new_child(parent);
                for (param, arg_val) in params.iter().zip(arg_values) {
                    child_env.define(param.clone(), arg_val);
                }

                // Push call frame
                self.call_stack.push(CallFrame);

                // Save current environment, switch to child
                let saved_env = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(child_env));

                // Execute body
                let return_val = loop {
                    match self.execute_body(&body) {
                        Ok(val) => break Ok(val),
                        Err(ObsidianError::Return { value }) => break Ok(value),
                        Err(e) => break Err(e),
                    }
                };

                // Restore environment
                self.environment = saved_env;
                self.call_stack.pop();

                Ok(return_val?.unwrap_or(Value::Null))
            }
            _ => Err(ObsidianError::TypeMismatch {
                message: format!("I tried to call '{}', but it's not a function", name),
            }),
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a Value to a bool (truthy/falsy).
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Text(s) => !s.is_empty(),
        Value::List(items) => !items.is_empty(),
        Value::Dict(entries) => !entries.is_empty(),
        Value::Null => false,
        Value::Function { .. } => true,
    }
}

/// Convert a Value to f64, or return a type error.
fn value_to_number(val: &Value) -> Result<f64, ObsidianError> {
    match val {
        Value::Number(n) => Ok(*n),
        Value::Text(s) => s.parse::<f64>().map_err(|_| ObsidianError::TypeMismatch {
            message: format!("I expected a number, but got '{}'", s),
        }),
        _ => Err(ObsidianError::TypeMismatch {
            message: format!(
                "I expected a number, but got {}",
                value_type_name(val)
            ),
        }),
    }
}

/// Convert a Value to String, or return a type error.
fn value_to_string(val: &Value) -> Result<String, ObsidianError> {
    match val {
        Value::Text(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        Value::Dict(_) => Ok(val.to_string()),
        _ => Err(ObsidianError::TypeMismatch {
            message: format!(
                "I expected text, but got {}",
                value_type_name(val)
            ),
        }),
    }
}

/// Get the type name of a Value for error messages.
fn value_type_name(val: &Value) -> &'static str {
    match val {
        Value::Text(_) => "text",
        Value::Number(_) => "number",
        Value::Bool(_) => "bool",
        Value::List(_) => "list",
        Value::Dict(_) => "dictionary",
        Value::Null => "null",
        Value::Function { .. } => "function",
    }
}

/// Compare two Values for sorting.
fn value_cmp(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        // Mixed types: compare type names, then use Display
        _ => value_type_name(a).cmp(value_type_name(b)).then_with(|| a.to_string().cmp(&b.to_string())),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to run a program and get the final result.
    fn run_program(program: Node) -> Result<Option<Value>, ObsidianError> {
        let mut interp = Interpreter::new();
        interp.execute(&program)
    }

    // ---- Variable Tests ----

    #[test]
    fn test_set_and_get_variable() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(42.0)),

pos: None,
            },
            Node::Show(Box::new(Node::Identifier("x".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_variable_error() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::Identifier(
            "nonexistent".to_string(),
        )))]);
        let result = run_program(program);
        assert!(result.is_err());
        match result.unwrap_err() {
            ObsidianError::UndefinedVariable { name } => {
                assert_eq!(name, "nonexistent");
            }
            e => panic!("Expected UndefinedVariable, got: {}", e),
        }
    }

    #[test]
    fn test_variable_shadowing() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(1.0)),

pos: None,
            },
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(2.0)),

pos: None,
            },
            Node::Show(Box::new(Node::Identifier("x".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Arithmetic Tests ----

    #[test]
    fn test_addition() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(3.0)),
            op: "+".to_string(),
            right: Box::new(Node::NumberLit(4.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_subtraction() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(10.0)),
            op: "-".to_string(),
            right: Box::new(Node::NumberLit(3.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiplication() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(5.0)),
            op: "*".to_string(),
            right: Box::new(Node::NumberLit(6.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_division() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(20.0)),
            op: "/".to_string(),
            right: Box::new(Node::NumberLit(4.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_division_by_zero() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(1.0)),
            op: "/".to_string(),
            right: Box::new(Node::NumberLit(0.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_concatenation() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::StringLit("Hello, ".to_string())),
            op: "+".to_string(),
            right: Box::new(Node::StringLit("World!".to_string())),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Boolean Logic Tests ----

    #[test]
    fn test_equality() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(5.0)),
            op: "==".to_string(),
            right: Box::new(Node::NumberLit(5.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_operator() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::BoolLit(true)),
            op: "not".to_string(),
            right: Box::new(Node::BoolLit(false)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_and_operator() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::BoolLit(true)),
            op: "and".to_string(),
            right: Box::new(Node::BoolLit(true)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_or_operator() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::BoolLit(false)),
            op: "or".to_string(),
            right: Box::new(Node::BoolLit(true)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_greater_than() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(10.0)),
            op: ">".to_string(),
            right: Box::new(Node::NumberLit(5.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_less_than() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::NumberLit(3.0)),
            op: "<".to_string(),
            right: Box::new(Node::NumberLit(7.0)),
        }))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Conditional Tests ----

    #[test]
    fn test_if_true_branch() {
        let program = Node::Program(vec![
            Node::If {
                condition: Box::new(Node::BoolLit(true)),
                body: vec![Node::Set {
                    name: "result".to_string(),
                    value: Box::new(Node::StringLit("yes".to_string())),

pos: None,
                }],
                otherwise: None,
            },
            Node::Show(Box::new(Node::Identifier("result".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_false_with_otherwise() {
        let program = Node::Program(vec![
            Node::If {
                condition: Box::new(Node::BoolLit(false)),
                body: vec![Node::Set {
                    name: "result".to_string(),
                    value: Box::new(Node::StringLit("yes".to_string())),

pos: None,
                }],
                otherwise: Some(vec![Node::Set {
                    name: "result".to_string(),
                    value: Box::new(Node::StringLit("no".to_string())),

pos: None,
                }]),
            },
            Node::Show(Box::new(Node::Identifier("result".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Loop Tests ----

    #[test]
    fn test_repeat_loop() {
        let program = Node::Program(vec![
            Node::Set {
                name: "counter".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::Repeat {
                times: Box::new(Node::NumberLit(3.0)),
                body: vec![Node::Set {
                    name: "counter".to_string(),
                    value: Box::new(Node::BinaryOp {

                        left: Box::new(Node::Identifier("counter".to_string())),
                        op: "+".to_string(),
                        right: Box::new(Node::NumberLit(1.0)),
                    }),
                    pos: None,
                }],
                var_name: None,
            },
            Node::Show(Box::new(Node::Identifier("counter".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_while_loop() {
        let program = Node::Program(vec![
            Node::Set {
                name: "i".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::While {
                condition: Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("i".to_string())),
                    op: "<".to_string(),
                    right: Box::new(Node::NumberLit(5.0)),
                }),
                body: vec![Node::Set {
                    name: "i".to_string(),
                    value: Box::new(Node::BinaryOp {

                        left: Box::new(Node::Identifier("i".to_string())),
                        op: "+".to_string(),
                        right: Box::new(Node::NumberLit(1.0)),
                    }),
                    pos: None,
                }],
            },
            Node::Show(Box::new(Node::Identifier("i".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Function Tests ----

    #[test]
    fn test_define_and_call_function() {
        let program = Node::Program(vec![
            Node::Define {
                name: "greet".to_string(),
                params: vec!["name".to_string()],
                body: vec![Node::Show(Box::new(Node::Identifier(
                    "name".to_string(),
                )))],
            },
            Node::Call {
                name: "greet".to_string(),
                args: vec![Node::StringLit("Alice".to_string())],
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_return() {
        let program = Node::Program(vec![
            Node::Define {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![Node::Return(Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("a".to_string())),
                    op: "+".to_string(),
                    right: Box::new(Node::Identifier("b".to_string())),
                }))],
            },
            Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::Call {

                    name: "add".to_string(),
                    args: vec![Node::NumberLit(3.0), Node::NumberLit(4.0)],
                }),
                pos: None,
            },
            Node::Show(Box::new(Node::Identifier("result".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_scope_isolation() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(10.0)),

pos: None,
            },
            Node::Define {
                name: "test_fn".to_string(),
                params: vec![],
                body: vec![Node::Set {
                    name: "x".to_string(),
                    value: Box::new(Node::NumberLit(20.0)),

pos: None,
                }],
            },
            Node::Call {
                name: "test_fn".to_string(),
                args: vec![],
            },
            // x should still be 10 in the outer scope
            Node::Show(Box::new(Node::Identifier("x".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Function Call in Expression Tests ----

    #[test]
    fn test_show_with_function_call() {
        // Test: show funcname with arg (the main bug fix)
        let program = Node::Program(vec![
            Node::Define {
                name: "double".to_string(),
                params: vec!["x".to_string()],
                body: vec![Node::Return(Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("x".to_string())),
                    op: "*".to_string(),
                    right: Box::new(Node::NumberLit(2.0)),
                }))],
            },
            Node::Show(Box::new(Node::Call {
                name: "double".to_string(),
                args: vec![Node::NumberLit(21.0)],
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_with_function_call() {
        // Test: return funcname with arg
        let program = Node::Program(vec![
            Node::Define {
                name: "add1".to_string(),
                params: vec!["x".to_string()],
                body: vec![Node::Return(Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("x".to_string())),
                    op: "+".to_string(),
                    right: Box::new(Node::NumberLit(1.0)),
                }))],
            },
            Node::Define {
                name: "wrapper".to_string(),
                params: vec!["y".to_string()],
                body: vec![Node::Return(Box::new(Node::Call {
                    name: "add1".to_string(),
                    args: vec![Node::Identifier("y".to_string())],
                }))],
            },
            Node::Show(Box::new(Node::Call {
                name: "wrapper".to_string(),
                args: vec![Node::NumberLit(10.0)],
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- List Tests ----

    #[test]
    fn test_list_literal() {
        let program = Node::Program(vec![Node::Set {
            name: "my_list".to_string(),
            value: Box::new(Node::ListLiteral(vec![
                Node::NumberLit(1.0),
                Node::NumberLit(2.0),
                Node::NumberLit(3.0),
            ])),

pos: None,
        }]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- File Operation Tests ----

    #[test]
    fn test_create_and_delete_file() {
        let test_file = "test_obsidian_file.txt";
        let program = Node::Program(vec![
            Node::CreateFile(Box::new(Node::StringLit(test_file.to_string()))),
            Node::DeleteFile(Box::new(Node::StringLit(test_file.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_and_read_file() {
        let test_file = "test_obsidian_write.txt";
        let program = Node::Program(vec![
            Node::WriteFile {
                path: Box::new(Node::StringLit(test_file.to_string())),
                content: Box::new(Node::StringLit("Hello, Obsidian!".to_string())),
            },
            Node::ReadFile {
                path: Box::new(Node::StringLit(test_file.to_string())),
                into: "content".to_string(),
            },
            Node::Show(Box::new(Node::Identifier("content".to_string()))),
            // Cleanup
            Node::DeleteFile(Box::new(Node::StringLit(test_file.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_append_file() {
        let test_file = "test_obsidian_append.txt";
        let program = Node::Program(vec![
            Node::WriteFile {
                path: Box::new(Node::StringLit(test_file.to_string())),
                content: Box::new(Node::StringLit("First line\n".to_string())),
            },
            Node::AppendFile {
                path: Box::new(Node::StringLit(test_file.to_string())),
                content: Box::new(Node::StringLit("Second line\n".to_string())),
            },
            Node::ReadFile {
                path: Box::new(Node::StringLit(test_file.to_string())),
                into: "content".to_string(),
            },
            Node::Show(Box::new(Node::Identifier("content".to_string()))),
            // Cleanup
            Node::DeleteFile(Box::new(Node::StringLit(test_file.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_file() {
        let test_file1 = "test_obsidian_copy_src.txt";
        let test_file2 = "test_obsidian_copy_dst.txt";
        let program = Node::Program(vec![
            Node::WriteFile {
                path: Box::new(Node::StringLit(test_file1.to_string())),
                content: Box::new(Node::StringLit("Copy me!".to_string())),
            },
            Node::CopyFile {
                from: Box::new(Node::StringLit(test_file1.to_string())),
                to: Box::new(Node::StringLit(test_file2.to_string())),
            },
            Node::DeleteFile(Box::new(Node::StringLit(test_file1.to_string()))),
            Node::DeleteFile(Box::new(Node::StringLit(test_file2.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_file() {
        let test_file1 = "test_obsidian_rename_old.txt";
        let test_file2 = "test_obsidian_rename_new.txt";
        let program = Node::Program(vec![
            Node::CreateFile(Box::new(Node::StringLit(test_file1.to_string()))),
            Node::RenameFile {
                from: Box::new(Node::StringLit(test_file1.to_string())),
                to: Box::new(Node::StringLit(test_file2.to_string())),
            },
            Node::DeleteFile(Box::new(Node::StringLit(test_file2.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_not_found_error() {
        let program = Node::Program(vec![Node::DeleteFile(Box::new(Node::StringLit(
            "nonexistent_file.txt".to_string(),
        )))]);
        let result = run_program(program);
        assert!(result.is_err());
        match result.unwrap_err() {
            ObsidianError::FileError { message } => {
                assert!(message.contains("nonexistent_file.txt"));
            }
            e => panic!("Expected FileError, got: {}", e),
        }
    }

    // ---- Value Display Tests ----

    #[test]
    fn test_display_number() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::NumberLit(42.0)))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_string() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::StringLit(
            "Hello!".to_string(),
        )))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_bool() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BoolLit(true)))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_list() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::ListLiteral(vec![
            Node::NumberLit(1.0),
            Node::StringLit("two".to_string()),
            Node::BoolLit(true),
        ])))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_null() {
        let program = Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::NumberLit(0.0)),

pos: None,
        }]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Error Message Tests ----

    #[test]
    fn test_undefined_variable_error_message() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::Identifier(
            "foo".to_string(),
        )))]);
        let err = run_program(program).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("foo"));
        assert!(msg.contains("couldn't find"));
    }

    #[test]
    fn test_type_mismatch_error_message() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::BinaryOp {
            left: Box::new(Node::StringLit("hello".to_string())),
            op: "-".to_string(),
            right: Box::new(Node::NumberLit(5.0)),
        }))]);
        let err = run_program(program).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("number"));
    }

    // ---- Nested Function Tests ----

    #[test]
    fn test_nested_function_calls() {
        let program = Node::Program(vec![
            Node::Define {
                name: "double".to_string(),
                params: vec!["x".to_string()],
                body: vec![Node::Return(Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("x".to_string())),
                    op: "*".to_string(),
                    right: Box::new(Node::NumberLit(2.0)),
                }))],
            },
            Node::Define {
                name: "add_one".to_string(),
                params: vec!["x".to_string()],
                body: vec![Node::Return(Box::new(Node::BinaryOp {
                    left: Box::new(Node::Identifier("x".to_string())),
                    op: "+".to_string(),
                    right: Box::new(Node::NumberLit(1.0)),
                }))],
            },
            // double(add_one(5)) = double(6) = 12
            Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::Call {

                    name: "double".to_string(),
                    args: vec![Node::Call {
                        name: "add_one".to_string(),
                        args: vec![Node::NumberLit(5.0)],
                    }],
                }),
                pos: None,
            },
            Node::Show(Box::new(Node::Identifier("result".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Truthy/Falsy Tests ----

    #[test]
    fn test_truthy_number() {
        let program = Node::Program(vec![Node::If {
            condition: Box::new(Node::NumberLit(1.0)),
            body: vec![Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::StringLit("truthy".to_string())),

pos: None,
            }],
            otherwise: Some(vec![Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::StringLit("falsy".to_string())),

pos: None,
            }]),
        }]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_falsy_zero() {
        let program = Node::Program(vec![Node::If {
            condition: Box::new(Node::NumberLit(0.0)),
            body: vec![Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::StringLit("truthy".to_string())),

pos: None,
            }],
            otherwise: Some(vec![Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::StringLit("falsy".to_string())),

pos: None,
            }]),
        }]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_falsy_null() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::If {
                condition: Box::new(Node::Identifier("x".to_string())),
                body: vec![Node::Set {
                    name: "result".to_string(),
                    value: Box::new(Node::StringLit("truthy".to_string())),

pos: None,
                }],
                otherwise: Some(vec![Node::Set {
                    name: "result".to_string(),
                    value: Box::new(Node::StringLit("falsy".to_string())),

pos: None,
                }]),
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- String Interpolation Tests ----

    #[test]
    fn test_string_interpolation_single_var() {
        let program = Node::Program(vec![
            Node::Set {
                name: "name".to_string(),
                value: Box::new(Node::StringLit("Alice".to_string())),

pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("Hello, ".to_string()),
                crate::ast::InterpPart::Var("name".to_string()),
                crate::ast::InterpPart::Lit("!".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_interpolation_multiple_vars() {
        let program = Node::Program(vec![
            Node::Set {
                name: "name".to_string(),
                value: Box::new(Node::StringLit("Bob".to_string())),

pos: None,
            },
            Node::Set {
                name: "age".to_string(),
                value: Box::new(Node::NumberLit(30.0)),

pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("Name: ".to_string()),
                crate::ast::InterpPart::Var("name".to_string()),
                crate::ast::InterpPart::Lit(", Age: ".to_string()),
                crate::ast::InterpPart::Var("age".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_interpolation_dict_field_access() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Alice".to_string())),
                    ("age".to_string(), Node::NumberLit(30.0)),
                ])),
                pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("Name is ".to_string()),
                crate::ast::InterpPart::Var("person.name".to_string()),
                crate::ast::InterpPart::Lit(" and age is ".to_string()),
                crate::ast::InterpPart::Var("person.age".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_interpolation_nested_dict_field_access() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    (
                        "address".to_string(),
                        Node::DictLiteral(vec![
                            ("city".to_string(), Node::StringLit("Wonderland".to_string())),
                            ("zip".to_string(), Node::StringLit("12345".to_string())),
                        ]),
                    ),
                    ("name".to_string(), Node::StringLit("Bob".to_string())),
                ])),
                pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("Name: ".to_string()),
                crate::ast::InterpPart::Var("person.name".to_string()),
                crate::ast::InterpPart::Lit(", City: ".to_string()),
                crate::ast::InterpPart::Var("person.address.city".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_interpolation_dict_field_missing() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Alice".to_string())),
                ])),
                pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("Age: ".to_string()),
                crate::ast::InterpPart::Var("person.age".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_interpolation_mixed_simple_and_field_access() {
        let program = Node::Program(vec![
            Node::Set {
                name: "greeting".to_string(),
                value: Box::new(Node::StringLit("Hello".to_string())),
                pos: None,
            },
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Charlie".to_string())),
                ])),
                pos: None,
            },
            Node::Show(Box::new(Node::InterpolatedString(vec![
                crate::ast::InterpPart::Lit("".to_string()),
                crate::ast::InterpPart::Var("greeting".to_string()),
                crate::ast::InterpPart::Lit(", ".to_string()),
                crate::ast::InterpPart::Var("person.name".to_string()),
                crate::ast::InterpPart::Lit("!".to_string()),
            ]))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Dictionary Tests ----

    #[test]
    fn test_dict_literal_create() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Sidd".to_string())),
                    ("age".to_string(), Node::NumberLit(20.0)),
                ])),

pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dict_field_access() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Sidd".to_string())),
                    ("age".to_string(), Node::NumberLit(20.0)),
                ])),

pos: None,
            },
            Node::Show(Box::new(Node::DictAccess {
                dict: Box::new(Node::Identifier("person".to_string())),
                field: "name".to_string(),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dict_field_access_missing() {
        let program = Node::Program(vec![
            Node::Set {
                name: "person".to_string(),
                value: Box::new(Node::DictLiteral(vec![
                    ("name".to_string(), Node::StringLit("Sidd".to_string())),
                ])),

pos: None,
            },
            Node::Show(Box::new(Node::DictAccess {
                dict: Box::new(Node::Identifier("person".to_string())),
                field: "missing_field".to_string(),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_err());
    }

    // ---- Break/Continue Tests ----

    #[test]
    fn test_break_in_repeat() {
        let program = Node::Program(vec![
            Node::Set {
                name: "counter".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::Repeat {
                times: Box::new(Node::NumberLit(10.0)),
                body: vec![
                    Node::Set {
                        name: "counter".to_string(),
                        value: Box::new(Node::BinaryOp {

                            left: Box::new(Node::Identifier("counter".to_string())),
                            op: "+".to_string(),
                            right: Box::new(Node::NumberLit(1.0)),
                        }),
                        pos: None,
                    },
                    Node::If {
                        condition: Box::new(Node::BinaryOp {
                            left: Box::new(Node::Identifier("counter".to_string())),
                            op: "==".to_string(),
                            right: Box::new(Node::NumberLit(3.0)),
                        }),
                        body: vec![Node::Break],
                        otherwise: None,
                    },
                ],
                var_name: None,
            },
            Node::Show(Box::new(Node::Identifier("counter".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_in_repeat() {
        let program = Node::Program(vec![
            Node::Set {
                name: "sum".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::Set {
                name: "i".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::Repeat {
                times: Box::new(Node::NumberLit(5.0)),
                body: vec![
                    Node::Set {
                        name: "i".to_string(),
                        value: Box::new(Node::BinaryOp {

                            left: Box::new(Node::Identifier("i".to_string())),
                            op: "+".to_string(),
                            right: Box::new(Node::NumberLit(1.0)),
                        }),
                        pos: None,
                    },
                    Node::If {
                        condition: Box::new(Node::BinaryOp {
                            left: Box::new(Node::Identifier("i".to_string())),
                            op: "==".to_string(),
                            right: Box::new(Node::NumberLit(3.0)),
                        }),
                        body: vec![Node::Continue],
                        otherwise: None,
                    },
                    Node::Set {
                        name: "sum".to_string(),
                        value: Box::new(Node::BinaryOp {

                            left: Box::new(Node::Identifier("sum".to_string())),
                            op: "+".to_string(),
                            right: Box::new(Node::NumberLit(1.0)),
                        }),
                        pos: None,
                    },
                ],
                var_name: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_break_in_while() {
        let program = Node::Program(vec![
            Node::Set {
                name: "i".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::While {
                condition: Box::new(Node::BoolLit(true)),
                body: vec![
                    Node::Set {
                        name: "i".to_string(),
                        value: Box::new(Node::BinaryOp {

                            left: Box::new(Node::Identifier("i".to_string())),
                            op: "+".to_string(),
                            right: Box::new(Node::NumberLit(1.0)),
                        }),
                        pos: None,
                    },
                    Node::If {
                        condition: Box::new(Node::BinaryOp {
                            left: Box::new(Node::Identifier("i".to_string())),
                            op: "==".to_string(),
                            right: Box::new(Node::NumberLit(5.0)),
                        }),
                        body: vec![Node::Break],
                        otherwise: None,
                    },
                ],
            },
            Node::Show(Box::new(Node::Identifier("i".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Try/Catch Tests ----

    #[test]
    fn test_try_catch_file_error() {
        let program = Node::Program(vec![
            Node::Try {
                body: vec![Node::DeleteFile(Box::new(Node::StringLit(
                    "nonexistent_file_12345.txt".to_string(),
                )))],
                catch_var: "error".to_string(),
                catch_body: vec![Node::Show(Box::new(Node::Identifier("error".to_string())))],
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_no_error() {
        let test_file = "test_try_no_error.txt";
        let program = Node::Program(vec![
            Node::Try {
                body: vec![
                    Node::CreateFile(Box::new(Node::StringLit(test_file.to_string()))),
                    Node::DeleteFile(Box::new(Node::StringLit(test_file.to_string()))),
                ],
                catch_var: "error".to_string(),
                catch_body: vec![Node::Show(Box::new(Node::Identifier("error".to_string())))],
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Range Loop Tests ----

    #[test]
    fn test_repeat_range_basic() {
        let program = Node::Program(vec![
            Node::Set {
                name: "sum".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::RepeatRange {
                start: Box::new(Node::NumberLit(1.0)),
                end: Box::new(Node::NumberLit(5.0)),
                var_name: "i".to_string(),
                body: vec![
                    Node::Set {
                        name: "sum".to_string(),
                        value: Box::new(Node::BinaryOp {

                            left: Box::new(Node::Identifier("sum".to_string())),
                            op: "+".to_string(),
                            right: Box::new(Node::Identifier("i".to_string())),
                        }),
                        pos: None,
                    },
                ],
            },
            Node::Show(Box::new(Node::Identifier("sum".to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repeat_range_with_break() {
        let program = Node::Program(vec![
            Node::Set {
                name: "last".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::RepeatRange {
                start: Box::new(Node::NumberLit(1.0)),
                end: Box::new(Node::NumberLit(10.0)),
                var_name: "i".to_string(),
                body: vec![
                    Node::Set {
                        name: "last".to_string(),
                        value: Box::new(Node::Identifier("i".to_string())),

pos: None,
                    },
                    Node::If {
                        condition: Box::new(Node::BinaryOp {
                            left: Box::new(Node::Identifier("i".to_string())),
                            op: "==".to_string(),
                            right: Box::new(Node::NumberLit(3.0)),
                        }),
                        body: vec![Node::Break],
                        otherwise: None,
                    },
                ],
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repeat_with_var_index() {
        let program = Node::Program(vec![
            Node::Set {
                name: "last_index".to_string(),
                value: Box::new(Node::NumberLit(0.0)),

pos: None,
            },
            Node::Repeat {
                times: Box::new(Node::NumberLit(5.0)),
                body: vec![
                    Node::Set {
                        name: "last_index".to_string(),
                        value: Box::new(Node::Identifier("idx".to_string())),

pos: None,
                    },
                ],
                var_name: Some("idx".to_string()),
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Dictionary Display Test ----

    #[test]
    fn test_display_dict() {
        let program = Node::Program(vec![Node::Show(Box::new(Node::DictLiteral(vec![
            ("name".to_string(), Node::StringLit("Sidd".to_string())),
            ("age".to_string(), Node::NumberLit(20.0)),
        ])))]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- New Standard Library Tests ----

    #[test]
    fn test_split_runtime() {
        let program = Node::Program(vec![
            Node::Set {
                name: "parts".to_string(),
                value: Box::new(Node::BuiltInBinary {

                    op: BuiltInBinaryOp::Split,
                    left: Box::new(Node::StringLit("a,b,c".to_string())),
                    right: Box::new(Node::StringLit(",".to_string())),
                }),
                pos: None,
            },
        ]);
        
        // Check that the variable 'parts' contains the correct list
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();
        let parts = interp.environment.borrow().get("parts").unwrap();
        match parts {
            Value::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Text("a".to_string()));
                assert_eq!(items[1], Value::Text("b".to_string()));
                assert_eq!(items[2], Value::Text("c".to_string()));
            }
            _ => panic!("Expected a list of 3 parts"),
        }
    }

    #[test]
    fn test_join_runtime() {
        let program = Node::Program(vec![
            Node::Set {
                name: "parts".to_string(),
                value: Box::new(Node::ListLiteral(vec![
                    Node::StringLit("a".to_string()),
                    Node::StringLit("b".to_string()),
                    Node::StringLit("c".to_string()),
                ])),

pos: None,
            },
            Node::Show(Box::new(Node::BuiltInBinary {
                op: BuiltInBinaryOp::JoinWith,
                left: Box::new(Node::Identifier("parts".to_string())),
                right: Box::new(Node::StringLit(",".to_string())),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_power_runtime() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInBinary {
                op: BuiltInBinaryOp::Power,
                left: Box::new(Node::NumberLit(2.0)),
                right: Box::new(Node::NumberLit(3.0)),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sqrt_runtime() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInUnary {
                op: BuiltInUnaryOp::Sqrt,
                operand: Box::new(Node::NumberLit(16.0)),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sqrt_negative_error() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInUnary {
                op: BuiltInUnaryOp::Sqrt,
                operand: Box::new(Node::NumberLit(-4.0)),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_exists_true() {
        let test_file = "test_file_exists_check.txt";
        let program = Node::Program(vec![
            Node::CreateFile(Box::new(Node::StringLit(test_file.to_string()))),
            Node::Show(Box::new(Node::BuiltInUnary {
                op: BuiltInUnaryOp::FileExists,
                operand: Box::new(Node::StringLit(test_file.to_string())),
            })),
            Node::DeleteFile(Box::new(Node::StringLit(test_file.to_string()))),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_exists_false() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInUnary {
                op: BuiltInUnaryOp::FileExists,
                operand: Box::new(Node::StringLit("nonexistent_file_xyz_12345.txt".to_string())),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_files_runtime() {
        // Create a temp directory with some files
        let test_dir = "test_list_files_dir";
        let _ = std::fs::create_dir_all(test_dir);
        let _ = std::fs::write(format!("{}/file1.txt", test_dir), "test");
        let _ = std::fs::write(format!("{}/file2.txt", test_dir), "test");

        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInUnary {
                op: BuiltInUnaryOp::ListFiles,
                operand: Box::new(Node::StringLit(test_dir.to_string())),
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());

        // Cleanup
        let _ = std::fs::remove_file(format!("{}/file1.txt", test_dir));
        let _ = std::fs::remove_file(format!("{}/file2.txt", test_dir));
        let _ = std::fs::remove_dir(test_dir);
    }

    #[test]
    fn test_current_date_runtime() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInNullary {
                op: BuiltInNullaryOp::CurrentDate,
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_current_time_runtime() {
        let program = Node::Program(vec![
            Node::Show(Box::new(Node::BuiltInNullary {
                op: BuiltInNullaryOp::CurrentTime,
            })),
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    // ---- Test Runner Tests ----

    #[test]
    fn test_expect_pass() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(5.0)),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::Identifier("x".to_string())),
                right: Box::new(Node::NumberLit(5.0)),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_fail() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(3.0)),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::Identifier("x".to_string())),
                right: Box::new(Node::NumberLit(5.0)),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_err());
        match result.unwrap_err() {
            ObsidianError::RuntimeError { message } => {
                assert!(message.contains("Assertion failed"));
                assert!(message.contains("5"));
                assert!(message.contains("3"));
            }
            e => panic!("Expected RuntimeError, got: {}", e),
        }
    }

    #[test]
    fn test_expect_string() {
        let program = Node::Program(vec![
            Node::Set {
                name: "name".to_string(),
                value: Box::new(Node::StringLit("hello".to_string())),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::Identifier("name".to_string())),
                right: Box::new(Node::StringLit("hello".to_string())),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_with_arithmetic() {
        let program = Node::Program(vec![
            Node::Set {
                name: "result".to_string(),
                value: Box::new(Node::BinaryOp {
                    left: Box::new(Node::NumberLit(2.0)),
                    op: "+".to_string(),
                    right: Box::new(Node::NumberLit(3.0)),
                }),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::Identifier("result".to_string())),
                right: Box::new(Node::NumberLit(5.0)),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_test_block_skipped_in_normal_execution() {
        // TestBlock should be skipped during normal program execution
        let program = Node::Program(vec![
            Node::TestBlock {
                name: "skipped test".to_string(),
                body: vec![
                    Node::Set {
                        name: "x".to_string(),
                        value: Box::new(Node::NumberLit(1.0)),
                        pos: None,
                    },
                ],
            },
        ]);
        let mut interp = Interpreter::new();
        let result = interp.execute(&program);
        assert!(result.is_ok());
        // x should not be defined since TestBlock was skipped
        assert!(interp.environment.borrow().get("x").is_err());
    }

    #[test]
    fn test_expect_length() {
        let program = Node::Program(vec![
            Node::Set {
                name: "s".to_string(),
                value: Box::new(Node::StringLit("hello".to_string())),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::BuiltInUnary {
                    op: BuiltInUnaryOp::Length,
                    operand: Box::new(Node::Identifier("s".to_string())),
                }),
                right: Box::new(Node::NumberLit(5.0)),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expect_bool() {
        let program = Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::BoolLit(true)),
                pos: None,
            },
            Node::Expect {
                left: Box::new(Node::Identifier("x".to_string())),
                right: Box::new(Node::BoolLit(true)),
                pos: None,
            },
        ]);
        let result = run_program(program);
        assert!(result.is_ok());
    }
}
