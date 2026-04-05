use super::*;
use crate::ast::{Node, BuiltInUnaryOp, BuiltInBinaryOp, BuiltInNullaryOp};
use crate::lexer::Lexer;

/// Helper function to parse a source string and return the AST.
fn parse(source: &str) -> Result<Node, ParseError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| {
        ParseError::UnexpectedToken {
            expected: "valid token".to_string(),
            found: format!("{:?}", e),
            position: 0,
        }
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// ============================================================================
// Set Statement Tests
// ============================================================================

#[test]
fn test_set_to_number() {
    let ast = parse("set x to 5").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::NumberLit(5.0)),

pos: None,
        }])
    );
}

#[test]
fn test_set_to_string() {
    let ast = parse(r#"set name to "hello""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "name".to_string(),
            value: Box::new(Node::StringLit("hello".to_string())),

pos: None,
        }])
    );
}

#[test]
fn test_set_is_number() {
    let ast = parse("set x is 10").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::NumberLit(10.0)),

pos: None,
        }])
    );
}

#[test]
fn test_set_equals_number() {
    let ast = parse("set x = 10").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::NumberLit(10.0)),

pos: None,
        }])
    );
}

// ============================================================================
// Show Statement Tests
// ============================================================================

#[test]
fn test_show_identifier() {
    let ast = parse("show x").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Show(Box::new(Node::Identifier("x".to_string())))])
    );
}

#[test]
fn test_show_string_literal() {
    let ast = parse(r#"show "hello world""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Show(Box::new(Node::StringLit(
            "hello world".to_string()
        )))])
    );
}

// ============================================================================
// Repeat Statement Tests
// ============================================================================

#[test]
fn test_repeat_empty_body() {
    let ast = parse("repeat 3 times { }").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Repeat {
            times: Box::new(Node::NumberLit(3.0)),
            body: vec![],
            var_name: None,
        }])
    );
}

#[test]
fn test_repeat_with_body() {
    let ast = parse("repeat 3 times { show x }").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Repeat {
            times: Box::new(Node::NumberLit(3.0)),
            body: vec![Node::Show(Box::new(Node::Identifier("x".to_string())))],
            var_name: None,
        }])
    );
}

// ============================================================================
// If Statement Tests
// ============================================================================

#[test]
fn test_if_without_otherwise() {
    let ast = parse("if x is 5 then show x end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::If {
            condition: Box::new(Node::BinaryOp {
                left: Box::new(Node::Identifier("x".to_string())),
                op: "==".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            body: vec![Node::Show(Box::new(Node::Identifier("x".to_string())))],
            otherwise: None,
        }])
    );
}

#[test]
fn test_if_with_otherwise() {
    let ast = parse("if x is 5 then show x otherwise show y end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::If {
            condition: Box::new(Node::BinaryOp {
                left: Box::new(Node::Identifier("x".to_string())),
                op: "==".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            body: vec![Node::Show(Box::new(Node::Identifier("x".to_string())))],
            otherwise: Some(vec![Node::Show(Box::new(Node::Identifier(
                "y".to_string()
            )))]),
        }])
    );
}

// ============================================================================
// Define Statement Tests
// ============================================================================

#[test]
fn test_define_with_single_param() {
    let ast = parse("define greet with name show name end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Define {
            name: "greet".to_string(),
            params: vec!["name".to_string()],
            body: vec![Node::Show(Box::new(Node::Identifier("name".to_string())))],
        }])
    );
}

#[test]
fn test_define_with_multiple_params() {
    let ast = parse("define add with a, b return a add b end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Define {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![Node::Return(Box::new(Node::BinaryOp {
                left: Box::new(Node::Identifier("a".to_string())),
                op: "+".to_string(),
                right: Box::new(Node::Identifier("b".to_string())),
            }))],
        }])
    );
}

#[test]
fn test_define_without_params() {
    let ast = parse("define hello show \"hi\" end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Define {
            name: "hello".to_string(),
            params: vec![],
            body: vec![Node::Show(Box::new(Node::StringLit("hi".to_string())))],
        }])
    );
}

// ============================================================================
// Call Statement Tests
// ============================================================================

#[test]
fn test_call_without_args() {
    let ast = parse("call greet").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Call {
            name: "greet".to_string(),
            args: vec![],
        }])
    );
}

#[test]
fn test_call_with_single_arg() {
    let ast = parse("call greet with name").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Call {
            name: "greet".to_string(),
            args: vec![Node::Identifier("name".to_string())],
        }])
    );
}

#[test]
fn test_call_with_multiple_args() {
    let ast = parse("call greet with name, age").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Call {
            name: "greet".to_string(),
            args: vec![
                Node::Identifier("name".to_string()),
                Node::Identifier("age".to_string()),
            ],
        }])
    );
}

// ============================================================================
// Return Statement Tests
// ============================================================================

#[test]
fn test_return_identifier() {
    let ast = parse("return x").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Return(Box::new(Node::Identifier(
            "x".to_string()
        )))])
    );
}

#[test]
fn test_return_number() {
    let ast = parse("return 42").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Return(Box::new(Node::NumberLit(42.0)))])
    );
}

// ============================================================================
// Exit Statement Tests
// ============================================================================

#[test]
fn test_exit() {
    let ast = parse("exit").unwrap();
    assert_eq!(ast, Node::Program(vec![Node::Exit]));
}

// ============================================================================
// BinaryOp Tests
// ============================================================================

#[test]
fn test_binary_op_addition() {
    let ast = parse("set x to 3 add 5").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::BinaryOp {

                left: Box::new(Node::NumberLit(3.0)),
                op: "+".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            pos: None,
        }])
    );
}

#[test]
fn test_binary_op_comparison() {
    let ast = parse("set x to a is 5").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::BinaryOp {

                left: Box::new(Node::Identifier("a".to_string())),
                op: "==".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            pos: None,
        }])
    );
}

// ============================================================================
// Literal Tests
// ============================================================================

#[test]
fn test_number_literal() {
    let ast = parse("set x to 3.14").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::NumberLit(3.14)),

pos: None,
        }])
    );
}

#[test]
fn test_bool_literal_true() {
    let ast = parse("set x to true").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::BoolLit(true)),

pos: None,
        }])
    );
}

#[test]
fn test_bool_literal_false() {
    let ast = parse("set x to false").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::BoolLit(false)),

pos: None,
        }])
    );
}

#[test]
fn test_list_literal() {
    let ast = parse("set x to list 1, 2, 3").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::ListLiteral(vec![
                Node::NumberLit(1.0),
                Node::NumberLit(2.0),
                Node::NumberLit(3.0),
            ])),

pos: None,
        }])
    );
}

// ============================================================================
// File Operation Tests
// ============================================================================

#[test]
fn test_create_file() {
    let ast = parse(r#"create "file.txt""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::CreateFile(Box::new(Node::StringLit(
            "file.txt".to_string()
        )))])
    );
}

#[test]
fn test_delete_file() {
    let ast = parse(r#"delete "file.txt""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::DeleteFile(Box::new(Node::StringLit(
            "file.txt".to_string()
        )))])
    );
}

#[test]
fn test_read_file() {
    let ast = parse(r#"read "file.txt" into content"#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::ReadFile {
            path: Box::new(Node::StringLit("file.txt".to_string())),
            into: "content".to_string(),
        }])
    );
}

#[test]
fn test_write_file() {
    let ast = parse(r#"write "file.txt" content "hello""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::WriteFile {
            path: Box::new(Node::StringLit("file.txt".to_string())),
            content: Box::new(Node::StringLit("hello".to_string())),
        }])
    );
}

#[test]
fn test_append_file() {
    let ast = parse(r#"append "file.txt" content "world""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::AppendFile {
            path: Box::new(Node::StringLit("file.txt".to_string())),
            content: Box::new(Node::StringLit("world".to_string())),
        }])
    );
}

#[test]
fn test_copy_file() {
    let ast = parse(r#"copy "a.txt" to "b.txt""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::CopyFile {
            from: Box::new(Node::StringLit("a.txt".to_string())),
            to: Box::new(Node::StringLit("b.txt".to_string())),
        }])
    );
}

#[test]
fn test_rename_file() {
    let ast = parse(r#"rename "old.txt" to "new.txt""#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::RenameFile {
            from: Box::new(Node::StringLit("old.txt".to_string())),
            to: Box::new(Node::StringLit("new.txt".to_string())),
        }])
    );
}

// ============================================================================
// Ask Statement Tests
// ============================================================================

#[test]
fn test_ask_into() {
    let ast = parse(r#"ask "Enter name:" into name"#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Ask {
            prompt: Box::new(Node::StringLit("Enter name:".to_string())),
            into: "name".to_string(),
        }])
    );
}

// ============================================================================
// While Statement Tests
// ============================================================================

#[test]
fn test_while_loop() {
    let ast = parse("while x is 5 do show x end").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::While {
            condition: Box::new(Node::BinaryOp {
                left: Box::new(Node::Identifier("x".to_string())),
                op: "==".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            body: vec![Node::Show(Box::new(Node::Identifier("x".to_string())))],
        }])
    );
}

// ============================================================================
// Multiple Statements Tests
// ============================================================================

#[test]
fn test_multiple_statements() {
    let ast = parse("set x to 5\nshow x").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![
            Node::Set {
                name: "x".to_string(),
                value: Box::new(Node::NumberLit(5.0)),

pos: None,
            },
            Node::Show(Box::new(Node::Identifier("x".to_string()))),
        ])
    );
}

// ============================================================================
// Parenthesized Expression Tests
// ============================================================================

#[test]
fn test_parenthesized_expression() {
    let ast = parse("set x to (3 add 5)").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "x".to_string(),
            value: Box::new(Node::BinaryOp {

                left: Box::new(Node::NumberLit(3.0)),
                op: "+".to_string(),
                right: Box::new(Node::NumberLit(5.0)),
            }),
            pos: None,
        }])
    );
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_empty_program() {
    let ast = parse("").unwrap();
    assert_eq!(ast, Node::Program(vec![]));
}

#[test]
fn test_unexpected_token() {
    let result = parse("to 5");
    assert!(result.is_err());
}

// ============================================================================
// String Interpolation Tests
// ============================================================================

#[test]
fn test_interpolated_string_single_var() {
    let ast = parse(r#"show "Hello {name}""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Show(expr) => {
                    match expr.as_ref() {
                        Node::InterpolatedString(parts) => {
                            assert_eq!(parts.len(), 2);
                        }
                        _ => panic!("Expected InterpolatedString"),
                    }
                }
                _ => panic!("Expected Show"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_interpolated_string_multiple_vars() {
    let ast = parse(r#"show "Hello {name}, you are {age} years old""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Show(expr) => {
                    match expr.as_ref() {
                        Node::InterpolatedString(parts) => {
                            // "Hello " + {name} + ", you are " + {age} + " years old"
                            assert!(parts.len() >= 4, "Expected at least 4 parts, got {}", parts.len());
                        }
                        other => panic!("Expected InterpolatedString, got {:?}", other),
                    }
                }
                _ => panic!("Expected Show"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

// ============================================================================
// Dictionary Tests
// ============================================================================

#[test]
fn test_dict_literal_parse() {
    let ast = parse(r#"set person to { name: "sidd", age: 20 }"#).unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Set {
            name: "person".to_string(),
            value: Box::new(Node::DictLiteral(vec![
                ("name".to_string(), Node::StringLit("sidd".to_string())),
                ("age".to_string(), Node::NumberLit(20.0)),
            ])),

pos: None,
        }])
    );
}

#[test]
fn test_dict_field_access_parse() {
    let ast = parse("show person.name").unwrap();
    assert_eq!(
        ast,
        Node::Program(vec![Node::Show(Box::new(Node::DictAccess {
            dict: Box::new(Node::Identifier("person".to_string())),
            field: "name".to_string(),
        }))])
    );
}

// ============================================================================
// Break/Continue Tests
// ============================================================================

#[test]
fn test_break_parse() {
    let ast = parse("break").unwrap();
    assert_eq!(ast, Node::Program(vec![Node::Break]));
}

#[test]
fn test_continue_parse() {
    let ast = parse("continue").unwrap();
    assert_eq!(ast, Node::Program(vec![Node::Continue]));
}

#[test]
fn test_break_in_loop_parse() {
    let ast = parse("repeat 5 times { if x is 5 then break end }").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Repeat { body, .. } => {
                    assert_eq!(body.len(), 1);
                }
                _ => panic!("Expected Repeat"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

// ============================================================================
// Try/Catch Tests
// ============================================================================

#[test]
fn test_try_catch_parse() {
    let ast = parse(r#"try delete "nonexistent.txt" catch error show error end"#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Try { body, catch_var, catch_body } => {
                    assert_eq!(body.len(), 1);
                    assert_eq!(catch_var, "error");
                    assert_eq!(catch_body.len(), 1);
                }
                _ => panic!("Expected Try"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

// ============================================================================
// Range Loop Tests
// ============================================================================

#[test]
fn test_range_loop_parse() {
    let ast = parse("repeat from 1 to 10 as i { show i }").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::RepeatRange { start, end, var_name, body } => {
                    assert_eq!(**start, Node::NumberLit(1.0));
                    assert_eq!(**end, Node::NumberLit(10.0));
                    assert_eq!(var_name, "i");
                    assert_eq!(body.len(), 1);
                }
                _ => panic!("Expected RepeatRange"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_range_loop_default_var() {
    let ast = parse("repeat from 1 to 5 { show i }").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::RepeatRange { var_name, .. } => {
                    assert_eq!(var_name, "i"); // default
                }
                _ => panic!("Expected RepeatRange"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_repeat_with_var_parse() {
    let ast = parse("repeat 3 times as idx { show idx }").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Repeat { var_name, body, .. } => {
                    assert_eq!(var_name, &Some("idx".to_string()));
                    assert_eq!(body.len(), 1);
                }
                _ => panic!("Expected Repeat"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

// ============================================================================
// New Standard Library Parser Tests
// ============================================================================

#[test]
fn test_split_parse() {
    let ast = parse(r#"split "a,b,c" by ",""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInBinary { op, .. } => {
                    assert_eq!(*op, BuiltInBinaryOp::Split);
                }
                _ => panic!("Expected BuiltInBinary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_join_parse() {
    let ast = parse(r#"join parts with ",""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInBinary { op, .. } => {
                    assert_eq!(*op, BuiltInBinaryOp::JoinWith);
                }
                _ => panic!("Expected BuiltInBinary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_power_parse() {
    let ast = parse("power 2 by 3").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInBinary { op, left, right } => {
                    assert_eq!(*op, BuiltInBinaryOp::Power);
                    assert_eq!(**left, Node::NumberLit(2.0));
                    assert_eq!(**right, Node::NumberLit(3.0));
                }
                _ => panic!("Expected BuiltInBinary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_sqrt_parse() {
    let ast = parse("square root of x").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInUnary { op, operand } => {
                    assert_eq!(*op, BuiltInUnaryOp::Sqrt);
                    assert_eq!(**operand, Node::Identifier("x".to_string()));
                }
                _ => panic!("Expected BuiltInUnary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_file_exists_parse() {
    let ast = parse(r#"file exists "test.txt""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInUnary { op, operand } => {
                    assert_eq!(*op, BuiltInUnaryOp::FileExists);
                    assert_eq!(**operand, Node::StringLit("test.txt".to_string()));
                }
                _ => panic!("Expected BuiltInUnary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_list_files_parse() {
    let ast = parse(r#"list files in "folder""#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInUnary { op, operand } => {
                    assert_eq!(*op, BuiltInUnaryOp::ListFiles);
                    assert_eq!(**operand, Node::StringLit("folder".to_string()));
                }
                _ => panic!("Expected BuiltInUnary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_current_date_parse() {
    let ast = parse("current date").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInNullary { op } => {
                    assert_eq!(*op, BuiltInNullaryOp::CurrentDate);
                }
                _ => panic!("Expected BuiltInNullary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

// ============================================================================
// Test Runner Tests
// ============================================================================

#[test]
fn test_test_block_parse() {
    let ast = parse(r#"test "addition works"
set x to 2
set y to 3
set result to x add y
expect result is 5
end"#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::TestBlock { name, body } => {
                    assert_eq!(name, "addition works");
                    assert_eq!(body.len(), 4);
                }
                _ => panic!("Expected TestBlock"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_test_block_multiple() {
    let ast = parse(r#"test "one"
set x to 1
end
test "two"
set y to 2
end"#).unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 2);
            match &nodes[0] {
                Node::TestBlock { name, .. } => {
                    assert_eq!(name, "one");
                }
                _ => panic!("Expected TestBlock"),
            }
            match &nodes[1] {
                Node::TestBlock { name, .. } => {
                    assert_eq!(name, "two");
                }
                _ => panic!("Expected TestBlock"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_expect_parse() {
    let ast = parse("expect x is 5").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Expect { left, right, .. } => {
                    assert_eq!(**left, Node::Identifier("x".to_string()));
                    assert_eq!(**right, Node::NumberLit(5.0));
                }
                _ => panic!("Expected Expect"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_expect_with_expression() {
    let ast = parse("expect a add b is 10").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::Expect { left, right, .. } => {
                    match left.as_ref() {
                        Node::BinaryOp { op, .. } => {
                            assert_eq!(op, "+");
                        }
                        _ => panic!("Expected BinaryOp"),
                    }
                    assert_eq!(**right, Node::NumberLit(10.0));
                }
                _ => panic!("Expected Expect"),
            }
        }
        _ => panic!("Expected Program"),
    }
}

#[test]
fn test_current_time_parse() {
    let ast = parse("current time").unwrap();
    match ast {
        Node::Program(nodes) => {
            assert_eq!(nodes.len(), 1);
            match &nodes[0] {
                Node::BuiltInNullary { op } => {
                    assert_eq!(*op, BuiltInNullaryOp::CurrentTime);
                }
                _ => panic!("Expected BuiltInNullary"),
            }
        }
        _ => panic!("Expected Program"),
    }
}
