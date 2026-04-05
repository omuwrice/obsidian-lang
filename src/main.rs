mod ast;
mod interpreter;
mod lexer;
mod parser;

use clap::{Parser, Subcommand};
use colored::Colorize;
use lexer::{Lexer, LexError};
use parser::{Parser as ObsidianParser, ParseError};
use interpreter::{Interpreter, ObsidianError};
use std::fs;
use std::path::Path;
use std::process;

// ============================================================================
// CLI Definition using clap derive
// ============================================================================

#[derive(Parser, Debug)]
#[command(
    name = "obsidian",
    about = "◆ Obsidian — Plain English Programming",
    long_about = None,
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run an Obsidian script
    Run {
        /// Path to the .obs file to run
        file: String,
    },
    /// Check syntax of an Obsidian script without running it
    Check {
        /// Path to the .obs file to check
        file: String,
    },
    /// Run tests in an Obsidian script
    Test {
        /// Path to the .obs file to test
        file: String,
    },
    /// Create a new Obsidian project
    New {
        /// Name of the new project
        name: String,
    },
}

// ============================================================================
// Error formatting helpers
// ============================================================================

/// Format a lexing error with friendly English and line number context.
fn format_lex_error(err: &LexError, source: &str) -> String {
    let (position, message) = match err {
        LexError::UnexpectedCharacter { ch, position } => {
            (*position, format!("I found an unexpected character '{}'", ch))
        }
        LexError::UnterminatedString { position } => {
            (*position, "I found a string that never ends. Did you forget a closing \"?".to_string())
        }
        LexError::InvalidNumber { value, position } => {
            (*position, format!("I couldn't understand '{}' as a number", value))
        }
    };

    let (line_num, col, line_content) = get_line_info(source, position);
    let pointer = build_pointer(col, 1);
    format!(
        "{}\n  at line {}, column {}\n  {}\n  {}",
        "Syntax error".red().bold(),
        line_num,
        col,
        message,
        highlight_line(&line_content, &pointer)
    )
}

/// Format a parsing error with friendly English and line number context.
fn format_parse_error(err: &ParseError, source: &str) -> String {
    let (position, message, span_width) = match err {
        ParseError::UnexpectedToken {
            expected,
            found,
            position,
        } => (
            *position,
            format!("I was expecting {}, but found '{}'", expected, found),
            1,
        ),
        ParseError::UnexpectedEOF { expected } => (
            source.len().saturating_sub(1),
            format!("I was expecting {} but reached the end of the file", expected),
            1,
        )
    };

    let (line_num, col, line_content) = get_line_info(source, position);
    let pointer = build_pointer(col, span_width);
    format!(
        "{}\n  at line {}, column {}\n  {}\n  {}",
        "Syntax error".red().bold(),
        line_num,
        col,
        message,
        highlight_line(&line_content, &pointer)
    )
}

/// Format a runtime error with friendly English and source position.
fn format_runtime_error(err: &ObsidianError, source: Option<(&str, usize, usize)>) -> String {
    match source {
        Some((line_content, line_num, col)) => {
            let pointer = build_pointer(col, 1);
            format!(
                "{}\n  at line {}, column {}\n  {}\n  {}",
                "Runtime error".red().bold(),
                line_num,
                col,
                err.to_string().red(),
                highlight_line(line_content, &pointer)
            )
        }
        None => format!(
            "{}\n  {}",
            "Runtime error".red().bold(),
            err.to_string().red()
        ),
    }
}

/// Get the line number (1-based), column (1-based), and content for a given byte position.
fn get_line_info(source: &str, byte_pos: usize) -> (usize, usize, String) {
    let up_to_pos = &source[..byte_pos.min(source.len())];
    let line_num = up_to_pos.lines().count();
    let line_content = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("").to_string();
    // Column: position within the line (1-based)
    let line_start = source.lines().take(line_num.saturating_sub(1)).map(|l| l.len() + 1).sum::<usize>();
    let col = byte_pos.saturating_sub(line_start) + 1;
    (line_num, col, line_content)
}

/// Build a pointer string with ^ characters at the right position.
fn build_pointer(column: usize, width: usize) -> String {
    let mut ptr = String::new();
    for _ in 0..(column.saturating_sub(1)) {
        ptr.push(' ');
    }
    for _ in 0..width {
        ptr.push('^');
    }
    ptr
}

/// Highlight the approximate position in a line of source code.
fn highlight_line(line: &str, pointer: &str) -> String {
    format!("  {}\n  {}", line.dimmed(), pointer.red().bold())
}

// ============================================================================
// Run command
// ============================================================================

fn cmd_run(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}\n  I couldn't find a file named '{}': {}",
                "File error".red().bold(),
                file,
                e
            );
            process::exit(1);
        }
    };

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", format_lex_error(&e, &source));
            process::exit(1);
        }
    };

    // Parse
    let mut parser = ObsidianParser::new_with_source(tokens, source.clone());
    let ast = match parser.parse_program() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", format_parse_error(&e, &source));
            process::exit(1);
        }
    };

    // Interpret
    let mut interp = Interpreter::new();
    match interp.execute(&ast) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{}", format_runtime_error(&e, Some((&source, 0, 0))));
            process::exit(1);
        }
    }
}

// ============================================================================
// Check command
// ============================================================================

fn cmd_check(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}\n  I couldn't find a file named '{}': {}",
                "File error".red().bold(),
                file,
                e
            );
            process::exit(1);
        }
    };

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", format_lex_error(&e, &source));
            process::exit(1);
        }
    };

    // Parse
    let mut parser = ObsidianParser::new_with_source(tokens, source.clone());
    match parser.parse_program() {
        Ok(_) => {
            println!("{}", "Looks good! No syntax errors found.".green().bold());
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{}", format_parse_error(&e, &source));
            process::exit(1);
        }
    }
}

// ============================================================================
// New command
// ============================================================================

fn cmd_new(name: &str) {
    let dir_path = Path::new(name);
    if dir_path.exists() {
        eprintln!(
            "{}\n  A directory named '{}' already exists.",
            "Error".red().bold(),
            name
        );
        process::exit(1);
    }

    match fs::create_dir_all(dir_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "{}\n  I couldn't create the directory '{}': {}",
                "Error".red().bold(),
                name,
                e
            );
            process::exit(1);
        }
    }

    let hello_content = "show \"Hello from Obsidian!\"\n";
    let hello_path = dir_path.join("hello.obs");
    match fs::write(&hello_path, hello_content) {
        Ok(_) => {
            println!(
                "{}\n  Created project '{}' with {}",
                "Project created!".green().bold(),
                name,
                "hello.obs".cyan()
            );
        }
        Err(e) => {
            eprintln!(
                "{}\n  I couldn't write hello.obs: {}",
                "Error".red().bold(),
                e
            );
            process::exit(1);
        }
    }
}

// ============================================================================
// REPL mode
// ============================================================================

fn cmd_repl() {
    // Welcome banner
    println!("{}", "◆ Obsidian — Plain English Programming".bold().purple());
    println!("{}", "Type English. Press Enter. Watch it happen.".dimmed());
    println!("{}", "Type \"exit\" to quit.".dimmed());
    println!();

    // Set up rustyline
    let mut rl = match rustyline::DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("{} I couldn't initialize the readline: {}", "Warning:".yellow(), e);
            repl_fallback();
            return;
        }
    };

    // Persistent interpreter across lines
    let mut interp = Interpreter::new();

    loop {
        // Read line
        let readline = rl.readline(&"> ".to_string().purple().bold().to_string());
        let line = match readline {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("{}", "\nGoodbye!".dimmed());
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("{}", "\nGoodbye!".dimmed());
                break;
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red(), e);
                continue;
            }
        };

        let trimmed = line.trim();

        // Exit command
        if trimmed == "exit" || trimmed == "quit" {
            println!("{}", "Goodbye!".dimmed());
            break;
        }

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Add to history
        let _ = rl.add_history_entry(trimmed);

        // Lex
        let mut lexer = Lexer::new(trimmed);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", format_lex_error(&e, trimmed));
                continue;
            }
        };

        // Parse
        let mut parser = ObsidianParser::new(tokens);
        let ast = match parser.parse_program() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", format_parse_error(&e, trimmed));
                continue;
            }
        };

        // Execute
        match interp.execute(&ast) {
            Ok(Some(value)) => {
                // If the expression returned a value, show it
                println!("{}", format!("{}", value).white());
            }
            Ok(None) => {
                // Statement executed silently (e.g., set, show already printed)
            }
            Err(ObsidianError::Return { value }) => {
                if let Some(v) = value {
                    println!("{}", format!("{}", v).white());
                }
            }
            Err(e) => {
                eprintln!("{}", format_runtime_error(&e, None));
            }
        }
    }
}

/// Fallback REPL if rustyline fails to initialize.
fn repl_fallback() {
    use std::io::{self, BufRead, Write};

    let stdin = io::stdin();
    let mut interp = Interpreter::new();

    loop {
        print!("{}", "> ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "quit" {
            println!("{}", "Goodbye!".dimmed());
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let mut lexer = Lexer::new(trimmed);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", format_lex_error(&e, trimmed));
                continue;
            }
        };

        let mut parser = ObsidianParser::new(tokens);
        let ast = match parser.parse_program() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{}", format_parse_error(&e, trimmed));
                continue;
            }
        };

        match interp.execute(&ast) {
            Ok(Some(value)) => {
                println!("{}", format!("{}", value).white());
            }
            Ok(None) => {}
            Err(ObsidianError::Return { value }) => {
                if let Some(v) = value {
                    println!("{}", format!("{}", v).white());
                }
            }
            Err(e) => {
                eprintln!("{}", format_runtime_error(&e, None));
            }
        }
    }
}

// ============================================================================
// Test command
// ============================================================================

fn cmd_test(file: &str) {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}\n  I couldn't find a file named '{}': {}",
                "File error".red().bold(),
                file,
                e
            );
            process::exit(1);
        }
    };

    // Lex
    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", format_lex_error(&e, &source));
            process::exit(1);
        }
    };

    // Parse
    let mut parser = ObsidianParser::new_with_source(tokens, source.clone());
    let ast = match parser.parse_program() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", format_parse_error(&e, &source));
            process::exit(1);
        }
    };

    // Execute tests
    let interp = Interpreter::new();
    let mut passed = 0;
    let mut failed = 0;

    if let ast::Node::Program(statements) = &ast {
        for stmt in statements {
            if let ast::Node::TestBlock { name, body } = stmt {
                // Run each test in a fresh interpreter (but with the same global env state)
                let mut test_interp = Interpreter::with_environment(interp.environment.clone());
                match test_interp.execute(&ast::Node::Program(body.clone())) {
                    Ok(_) => {
                        println!("{} {}", "\u{2713}".green(), name);
                        passed += 1;
                    }
                    Err(e) => {
                        println!("{} {} - {}", "\u{2717}".red().bold(), name, e);
                        failed += 1;
                    }
                }
            }
        }
    }

    // Summary
    if failed == 0 {
        println!("\n{}", format!("{} test{} passed, 0 failed", passed, if passed != 1 { "s" } else { "" }).green().bold());
    } else {
        println!(
            "\n{}",
            format!("{} passed, {} failed", passed, failed).red().bold()
        );
    }

    if failed > 0 {
        process::exit(1);
    }
}

// ============================================================================
// Main entry point
// ============================================================================

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run { file }) => cmd_run(&file),
        Some(Commands::Check { file }) => cmd_check(&file),
        Some(Commands::Test { file }) => cmd_test(&file),
        Some(Commands::New { name }) => cmd_new(&name),
        None => cmd_repl(),
    }
}
