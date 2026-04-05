use super::*;

fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[test]
fn test_string_interpolation_empty() {
    let tokens = tokenize(r#"show "Hello {}""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("Hello ".to_string()),
        Token::StringInterpolation(String::new()),
        Token::StringLiteral("".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_string_interpolation_with_content() {
    let tokens = tokenize(r#"show "Hello {name}""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("Hello ".to_string()),
        Token::StringInterpolation("name".to_string()),
        Token::StringLiteral("".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_string_interpolation_multiple() {
    let tokens = tokenize(r#"show "Hello {name}, you are {age} years old""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("Hello ".to_string()),
        Token::StringInterpolation("name".to_string()),
        Token::StringLiteral(", you are ".to_string()),
        Token::StringInterpolation("age".to_string()),
        Token::StringLiteral(" years old".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_string_interpolation_at_start() {
    let tokens = tokenize(r#"show "{name} is here""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("".to_string()),
        Token::StringInterpolation("name".to_string()),
        Token::StringLiteral(" is here".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_string_interpolation_at_end() {
    let tokens = tokenize(r#"show "Welcome {name}""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("Welcome ".to_string()),
        Token::StringInterpolation("name".to_string()),
        Token::StringLiteral("".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_string_interpolation_only() {
    let tokens = tokenize(r#"show "{}""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::StringLiteral("".to_string()),
        Token::StringInterpolation(String::new()),
        Token::StringLiteral("".to_string()),
        Token::EOF
    ]);
}

// ============================================================================
// Dictionary Token Tests
// ============================================================================

#[test]
fn test_dict_literal_tokens() {
    let tokens = tokenize(r#"set x to { name: "sidd" }"#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("set".to_string()),
        Token::Identifier("x".to_string()),
        Token::Keyword("to".to_string()),
        Token::LeftBrace,
        Token::Identifier("name".to_string()),
        Token::Colon,
        Token::StringLiteral("sidd".to_string()),
        Token::RightBrace,
        Token::EOF
    ]);
}

#[test]
fn test_dict_multiple_entries() {
    let tokens = tokenize(r#"set p to { name: "sidd", age: 20 }"#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("set".to_string()),
        Token::Identifier("p".to_string()),
        Token::Keyword("to".to_string()),
        Token::LeftBrace,
        Token::Identifier("name".to_string()),
        Token::Colon,
        Token::StringLiteral("sidd".to_string()),
        Token::Comma,
        Token::Identifier("age".to_string()),
        Token::Colon,
        Token::NumberLiteral(20.0),
        Token::RightBrace,
        Token::EOF
    ]);
}

#[test]
fn test_dict_field_access_tokens() {
    let tokens = tokenize("show person.name").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("show".to_string()),
        Token::Identifier("person".to_string()),
        Token::Dot,
        Token::Identifier("name".to_string()),
        Token::EOF
    ]);
}

// ============================================================================
// Break/Continue Token Tests
// ============================================================================

#[test]
fn test_break_keyword() {
    let tokens = tokenize("break").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("break".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_continue_keyword() {
    let tokens = tokenize("continue").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("continue".to_string()),
        Token::EOF
    ]);
}

// ============================================================================
// Try/Catch Token Tests
// ============================================================================

#[test]
fn test_try_catch_tokens() {
    let tokens = tokenize("try delete \"x.txt\" catch error show error end").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("try".to_string()),
        Token::Keyword("delete".to_string()),
        Token::StringLiteral("x.txt".to_string()),
        Token::Keyword("catch".to_string()),
        Token::Identifier("error".to_string()),
        Token::Keyword("show".to_string()),
        Token::Identifier("error".to_string()),
        Token::Keyword("end".to_string()),
        Token::EOF
    ]);
}

// ============================================================================
// Range Loop Token Tests
// ============================================================================

#[test]
fn test_range_loop_tokens() {
    let tokens = tokenize("repeat from 1 to 10 as i { show i }").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("repeat".to_string()),
        Token::Keyword("from".to_string()),
        Token::NumberLiteral(1.0),
        Token::Keyword("to".to_string()),
        Token::NumberLiteral(10.0),
        Token::Keyword("as".to_string()),
        Token::Identifier("i".to_string()),
        Token::LeftBrace,
        Token::Keyword("show".to_string()),
        Token::Identifier("i".to_string()),
        Token::RightBrace,
        Token::EOF
    ]);
}

// ============================================================================
// New Standard Library Token Tests
// ============================================================================

#[test]
fn test_split_tokens() {
    let tokens = tokenize(r#"split "a,b,c" by ",""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("split".to_string()),
        Token::StringLiteral("a,b,c".to_string()),
        Token::Keyword("by".to_string()),
        Token::StringLiteral(",".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_join_tokens() {
    let tokens = tokenize(r#"join parts with ",""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("join".to_string()),
        Token::Identifier("parts".to_string()),
        Token::Keyword("with".to_string()),
        Token::StringLiteral(",".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_power_tokens() {
    let tokens = tokenize("power 2 by 3").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("power".to_string()),
        Token::NumberLiteral(2.0),
        Token::Keyword("by".to_string()),
        Token::NumberLiteral(3.0),
        Token::EOF
    ]);
}

#[test]
fn test_square_root_tokens() {
    let tokens = tokenize("square root of x").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("square".to_string()),
        Token::Keyword("root".to_string()),
        Token::Keyword("of".to_string()),
        Token::Identifier("x".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_file_exists_tokens() {
    let tokens = tokenize(r#"file exists "test.txt""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("file".to_string()),
        Token::Keyword("exists".to_string()),
        Token::StringLiteral("test.txt".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_list_files_tokens() {
    let tokens = tokenize(r#"list files in "folder""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("list".to_string()),
        Token::Keyword("files".to_string()),
        Token::Keyword("in".to_string()),
        Token::StringLiteral("folder".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_current_date_tokens() {
    let tokens = tokenize("current date").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("current".to_string()),
        Token::Keyword("date".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_current_time_tokens() {
    let tokens = tokenize("current time").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("current".to_string()),
        Token::Keyword("time".to_string()),
        Token::EOF
    ]);
}

// ============================================================================
// Test Runner Token Tests
// ============================================================================

#[test]
fn test_test_keyword() {
    let tokens = tokenize(r#"test "my test""#).unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("test".to_string()),
        Token::StringLiteral("my test".to_string()),
        Token::EOF
    ]);
}

#[test]
fn test_expect_keyword() {
    let tokens = tokenize("expect x is 5").unwrap();
    assert_eq!(tokens, vec![
        Token::Keyword("expect".to_string()),
        Token::Identifier("x".to_string()),
        Token::Keyword("is".to_string()),
        Token::NumberLiteral(5.0),
        Token::EOF
    ]);
}
