# ◆ Obsidian — Plain English Programming

```
    ◆
   ◆ ◆
  ◆   ◆
 ◆ Obs ◆
  ◆   ◆
   ◆ ◆
    ◆
```

![155 tests passing](https://img.shields.io/badge/tests-155%20passing-brightgreen)

## What is Obsidian?

Obsidian is a programming language designed for **readability** and **simplicity**.
Write code that reads like plain English sentences — no semicolons, no curly braces
for blocks, no cryptic symbols.

**Key features:**
- 📖 **English-like syntax** — `set x to 42`, `show "Hello"`, `if x is 5 then`
- 🔧 **Built-in file operations** — create, read, write, append, copy, rename files
- 📋 **Native list support** — `list 1, 2, 3`, `push x to y`, `pop from y`
- 📚 **Dictionary type** — `{ name: "Alice", age: 30 }` with dot access
- 💬 **String interpolation** — `"Hello, {name}!"` embeds variables in strings
- 🔄 **Range loops** — `repeat from 1 to 10 as i { }`
- 🧪 **Built-in test runner** — `test "name" ... end` with `expect` assertions
- 🛡️ **Error handling** — `try ... catch error ... end`
- 🎲 **Rich standard library** — text, number, list, file, and date operations
- 🧪 **Tree-walking interpreter** — written in Rust for safety and performance

---

## Install

### Quick Install

```bash
./install.sh
```

### Manual Install

```bash
cargo build --release
sudo cp target/release/obsidian /usr/local/bin/
```

### Verify Installation

```bash
obsidian run examples/hello.obs
```

---

## Quick Start

### Hello World

```obsidian
show "Hello, World!"
```

### Variables and Math

```obsidian
set x to 10
set y to 20
show x add y
```

### User Input

```obsidian
ask "What is your name? " into name
show "Hello, " + name
```

### Conditionals

```obsidian
set age to 18
if age is 18 then
    show "You are an adult"
otherwise
    show "You are a minor"
end
```

### Loops

```obsidian
repeat 5 times {
    show "Hello!"
}
```

### Functions

```obsidian
define greet with name
    show "Hello, " + name
end

call greet with "Alice"
```

### Dictionaries

```obsidian
set person to { name: "Alice", age: 30 }
show "Name: " add person.name
show "Age: " add person.age
```

### String Interpolation

```obsidian
set name to "Alice"
set age to 30
show "Hello, I am {name} and I am {age} years old."
```

### Range Loops

```obsidian
repeat from 1 to 5 as i {
    show "Number: {i}"
}
```

### Error Handling

```obsidian
try
    delete "nonexistent.txt"
catch error
    show "Caught: {error}"
end
```

### Testing

```obsidian
test "addition works"
    set x to 2
    set y to 3
    set result to x add y
    expect result is 5
end
```

Run with `obsidian test script.obs`:
```
✓ addition works
1 test passed, 0 failed
```

---

## Keyword Reference

### I/O Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `show` | Print to console | `show "Hello"` |
| `display` | Alias for show | `display x` |
| `print` | Alias for show | `print x` |
| `ask` | Read user input | `ask "Name: " into name` |
| `input` | Alias for ask | `input "Age: " into age` |

### Assignment Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `set` | Create/assign variable | `set x to 42` |
| `to` | Assignment operator | `set x to 10` |
| `is` | Assignment or comparison | `set x is 5`, `if x is 5` |
| `as` | Type conversion | `x as text` |

### Control Flow Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `if` | Conditional | `if x is 5 then` |
| `otherwise` | Else branch | `otherwise show "no"` |
| `else` | Alias for otherwise | `else show "no"` |
| `then` | Start if block | `if x is 5 then` |
| `end` | Close block | `end` |
| `repeat` | Counted loop | `repeat 5 times` |
| `times` | Loop count suffix | `repeat 5 times` |
| `while` | While loop | `while x < 10 do` |
| `until` | Alias for while | `until x is 10 do` |

### Function Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `define` | Define function | `define greet with name` |
| `function` | Alias for define | `function greet with name` |
| `call` | Call function | `call greet with "Alice"` |
| `return` | Return value | `return x add y` |
| `give` | Alias for return | `give back x` |
| `back` | Part of give back | `give back x` |

### Math Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `add` | Addition | `x add y` |
| `subtract` | Subtraction | `x subtract y` |
| `multiply` | Multiplication | `x multiply y` |
| `divide` | Division | `x divide y` |
| `by` | Used in operations | — |
| `from` | Used in operations | — |

### File Operation Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `create` | Create file | `create "file.txt"` |
| `delete` | Delete file | `delete "file.txt"` |
| `read` | Read file | `read "file.txt" into content` |
| `write` | Write file | `write "file.txt" content "Hello"` |
| `append` | Append to file | `append "file.txt" content "World"` |
| `copy` | Copy file | `copy "a.txt" to "b.txt"` |
| `rename` | Rename file | `rename "old.txt" to "new.txt"` |
| `move` | Alias for rename | `move "old.txt" to "new.txt"` |

### Logic Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `not` | Logical NOT | `not x` |
| `and` | Logical AND | `x and y` |
| `or` | Logical OR | `x or y` |
| `exists` | Existence check | — |

### List Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `list` | Create list | `list 1, 2, 3` |
| `push` | Add to list | `push 4 to my_list` |
| `pop` | Remove from list | `pop from my_list` |
| `contains` | Membership check | `my_list contains 3` |

### Type Conversion Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `text` | Convert to string | `x as text` |
| `number` | Convert to number | `x as number` |
| `truth` | Convert to bool | `x as truth` |

### System Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `exit` | Terminate program | `exit` |
| `run` | Run external command | — |
| `clear` | Clear screen | — |

### Error Handling Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `try` | Start error handling block | `try` |
| `catch` | Catch errors | `catch error` |

### Testing Keywords
| Keyword | Usage | Example |
|---------|-------|---------|
| `test` | Define test block | `test "name"` |
| `expect` | Assert equality | `expect x is 5` |

---

## Standard Library Reference

### Text Functions

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| `uppercase` | `uppercase of x` | text | `uppercase of "hello"` → `"HELLO"` |
| `lowercase` | `lowercase of x` | text | `lowercase of "HELLO"` → `"hello"` |
| `length` | `length of x` | number | `length of "hello"` → `5` |
| `trim` | `trim of x` | text | `trim of "  hi  "` → `"hi"` |
| `reverse` | `reverse of x` | text | `reverse of "abc"` → `"cba"` |
| `contains` | `x contains y` | bool | `"hello" contains "ell"` → `true` |
| `replace` | `replace old with new in text` | text | `replace "a" with "b" in "cat"` → `"cbt"` |

### Number Functions

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| `round` | `round x` | number | `round 3.7` → `4` |
| `floor` | `floor x` | number | `floor 3.7` → `3` |
| `ceiling` | `ceiling x` | number | `ceiling 3.2` → `4` |
| `absolute` | `absolute of x` | number | `absolute of -5` → `5` |
| `random` | `random between low and high` | number | `random between 1 and 10` |

### List Functions

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| `count` | `count of x` | number | `count of list 1, 2, 3` → `3` |
| `first` | `first of x` | any | `first of list 1, 2, 3` → `1` |
| `last` | `last of x` | any | `last of list 1, 2, 3` → `3` |
| `sort` | `sort of x` | list | `sort of list 3, 1, 2` → `list 1, 2, 3` |
| `reverse` | `reverse of x` | list | `reverse of list 1, 2, 3` → `list 3, 2, 1` |

### Type Conversion

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| `as text` | `x as text` | text | `42 as text` → `"42"` |
| `as number` | `x as number` | number | `"3.14" as number` → `3.14` |
| `as truth` | `x as truth` | bool | `1 as truth` → `true` |

### Additional Functions

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| `split` | `split text by delim` | list | `split "a,b" by ","` → `list "a", "b"` |
| `join` | `join list with sep` | text | `join parts with ","` → `"a,b"` |
| `power` | `power base by exp` | number | `power 2 by 3` → `8` |
| `square root` | `square root of x` | number | `square root of 16` → `4` |
| `file exists` | `file exists path` | bool | `file exists "test.txt"` → `true` |
| `list files` | `list files in dir` | list | `list files in "."` |
| `current date` | `current date` | text | `current date` → `"2025-01-15"` |
| `current time` | `current time` | text | `current time` → `"14:30:00"` |

---

## How It Works

Obsidian uses a classic **tree-walking interpreter** pipeline:

```
┌─────────────┐     ┌──────────┐     ┌─────────┐     ┌─────────────┐     ┌────────────┐
│  English    │ ──► │  Lexer   │ ──► │ Parser  │ ──► │     AST     │ ──► │Interpreter │
│  Source     │     │          │     │         │     │             │     │            │
│  (.obs)     │     │ Tokens   │     │ Syntax  │     │ Tree        │     │ Execute    │
└─────────────┘     └──────────┘     └─────────┘     └─────────────┘     └────────────┘
```

1. **Lexer** — Reads source text character by character and produces tokens
2. **Parser** — Consumes tokens and builds an Abstract Syntax Tree (AST)
3. **AST** — A tree representation of the program's structure
4. **Interpreter** — Walks the tree and executes each node

---

## Project Structure

```
obsidian/
├── Cargo.toml              # Rust package manifest
├── install.sh              # Quick install script
├── README.md               # This file
├── docs/
│   └── language-spec.md    # Full language specification
├── examples/
│   ├── hello.obs           # Hello World
│   ├── calculator.obs      # Simple calculator
│   ├── fizzbuzz.obs        # FizzBuzz game
│   ├── todo.obs            # Todo list with file I/O
│   ├── strings.obs         # String operations demo
│   ├── lists.obs           # List operations demo
│   ├── math.obs            # Math functions demo
│   ├── interpolation.obs   # String interpolation demo
│   ├── dictionary.obs      # Dictionary type demo
│   ├── ranges.obs          # Range loop demo
│   ├── error_handling.obs  # Try/catch demo
│   ├── testing.obs         # Test runner demo
│   └── advanced.obs        # Contact book (all features)
├── src/
│   ├── main.rs             # CLI entry point (run, check, test, new)
│   ├── ast.rs              # AST node definitions + SourcePosition
│   ├── lexer/
│   │   ├── mod.rs          # Lexer implementation
│   │   └── tests.rs        # Lexer tests
│   ├── parser/
│   │   ├── mod.rs          # Parser implementation with source tracking
│   │   └── tests.rs        # Parser tests
│   └── interpreter/
│       ├── mod.rs          # Interpreter implementation + tests
└── vscode-obsidian/        # VS Code syntax highlighting extension
    ├── package.json
    └── syntaxes/
        └── obsidian.tmLanguage.json
```

---

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Examples

```bash
cargo run -- run examples/hello.obs
cargo run -- run examples/fizzbuzz.obs
cargo run -- run examples/advanced.obs
```

### Check Syntax (without running)

```bash
cargo run -- check examples/hello.obs
```

### Run Tests with Built-in Test Runner

```bash
cargo run -- test examples/testing.obs
```

---

## License

MIT
