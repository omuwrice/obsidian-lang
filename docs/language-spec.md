# Obsidian Language Specification

> **Version:** 0.2.0 — Updated with string interpolation, dictionaries, range loops, test runner, and error handling.

## Overview

Obsidian is a plain English programming language designed for readability and simplicity.
All keywords are case-insensitive, and the syntax reads like natural English sentences.

**File extension:** `.obs`

**Compilation pipeline:**
```
English Source → Lexer → Parser → AST → Interpreter → Output
```

---

## 1. Lexical Structure

### 1.1 Comments

Single-line comments begin with `//`:

```obsidian
// This is a comment
show "Hello" // This is also a comment
```

### 1.2 Identifiers

Identifiers are names for variables and functions. They start with a letter or underscore,
followed by any number of letters, digits, or underscores.

```obsidian
set myVariable to 42
set _count to 0
set user_name to "Alice"
```

### 1.3 Literals

| Type | Example | Description |
|------|---------|-------------|
| Number | `42`, `3.14` | Integer or floating-point number |
| String | `"hello"` | Text enclosed in double quotes |
| Boolean | `true`, `false` | Truth values |
| List | `list 1, 2, 3` | Comma-separated values prefixed with `list` |

### 1.4 Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `+` | Add / Concatenate | `3 add 5`, `"hello" + " world"` |
| `-` | Subtract | `10 subtract 3` |
| `*` | Multiply | `4 multiply 5` |
| `/` | Divide | `20 divide 4` |
| `==` | Equal | `x == 5` |
| `!=` | Not equal | `x != 0` |
| `<` | Less than | `x < 10` |
| `>` | Greater than | `x > 5` |
| `<=` | Less than or equal | `x <= 10` |
| `>=` | Greater than or equal | `x >= 5` |

---

## 2. Keywords

### 2.1 Complete Keyword List

| Category | Keywords |
|----------|----------|
| **I/O** | `show`, `display`, `print`, `ask`, `input` |
| **Assignment** | `set`, `to`, `is`, `as` |
| **Control Flow** | `if`, `otherwise`, `else`, `then`, `end`, `repeat`, `times`, `while`, `until`, `stop` |
| **Functions** | `define`, `function`, `call`, `return`, `give`, `back` |
| **Math** | `add`, `subtract`, `multiply`, `divide`, `by`, `from` |
| **File Ops** | `create`, `delete`, `read`, `write`, `append`, `rename`, `copy`, `move` |
| **Logic** | `exists`, `not`, `and`, `or` |
| **List Ops** | `list`, `empty`, `contains`, `count`, `length`, `at`, `index`, `push`, `pop`, `remove` |
| **Type Conversion** | `text`, `number`, `truth` |
| **Structural** | `of`, `with`, `in`, `into`, `content` |
| **System** | `run`, `exit`, `clear` |
| **Literals** | `true`, `false` |
| **Text Functions** | `uppercase`, `lowercase`, `trim`, `reverse`, `replace` |
| **Number Functions** | `round`, `floor`, `ceiling`, `absolute`, `random`, `between` |
| **List Functions** | `first`, `last`, `sort` |
| **Error Handling** | `try`, `catch` |
| **Testing** | `test`, `expect` |

---

## 3. Syntax Rules

### 3.1 Variables

Variables are created with the `set` keyword. Three assignment syntaxes are supported:

```obsidian
set x to 42
set x is 42
set x = 42
```

Variables can be reassigned:

```obsidian
set x to 10
set x to 20
```

### 3.2 Output

Display values to the console:

```obsidian
show "Hello, World!"
show x
show "The answer is " + result
```

### 3.3 Input

Read user input:

```obsidian
ask "Enter your name: " into name
show "Hello, " + name
```

### 3.4 Conditionals

```obsidian
if <condition> then
    <statements>
otherwise
    <statements>
end
```

**Example:**

```obsidian
if x is 5 then
    show "x is five"
otherwise
    show "x is not five"
end
```

The `otherwise` branch is optional:

```obsidian
if x > 0 then
    show "positive"
end
```

### 3.5 Loops

**Repeat (counted loop):**

```obsidian
repeat <count> times {
    <statements>
}
```

**Example:**

```obsidian
repeat 5 times {
    show "Hello!"
}
```

**While loop:**

```obsidian
while <condition> do
    <statements>
end
```

**Example:**

```obsidian
set i to 0
while i < 5 do
    show i
    set i to i add 1
end
```

### 3.6 Functions

**Defining functions:**

```obsidian
define <name> with <param1>, <param2>
    <statements>
end
```

**Example:**

```obsidian
define greet with name
    show "Hello, " + name
end
```

**Calling functions:**

```obsidian
call greet with "Alice"
call greet with name, age
```

**Returning values:**

```obsidian
define add with a, b
    return a add b
end
```

### 3.7 Lists

**Creating lists:**

```obsidian
set my_list to list 1, 2, 3, 4, 5
```

**Push to list:**

```obsidian
push 6 to my_list
```

**Pop from list:**

```obsidian
pop from my_list
```

### 3.8 File Operations

```obsidian
create "file.txt"
delete "file.txt"
read "file.txt" into content
write "file.txt" content "Hello"
append "file.txt" content "World"
copy "source.txt" to "dest.txt"
rename "old.txt" to "new.txt"
```

### 3.9 Exit

Terminate the program:

```obsidian
exit
```

### 3.10 String Interpolation

Variables can be embedded directly inside strings using `{variable}` syntax:

```obsidian
set name to "Alice"
set age to 30
show "Hello, my name is {name} and I am {age} years old."
```

Interpolation works with any variable name and can appear multiple times in a single string:

```obsidian
set city to "Tokyo"
set score to 95
show "{name} scored {score} points in {city}."
```

### 3.11 Dictionaries

Dictionaries store key-value pairs using brace syntax:

```obsidian
set person to { name: "Alice", age: 30, city: "Tokyo" }
```

Access fields using dot notation:

```obsidian
show person.name
show person.age
```

Dictionaries can contain other dictionaries as values:

```obsidian
set company to { name: "Acme Corp", ceo: "Bob", employees: 100 }
show company.name
show company.ceo
```

Dictionaries can be stored in lists:

```obsidian
set contacts to list { name: "Alice", phone: "555-0101" }, { name: "Bob", phone: "555-0202" }
set first_contact to first of contacts
show first_contact.name
```

### 3.12 Range Loops

Iterate over a range of numbers using `repeat from X to Y as var`:

```obsidian
repeat from 1 to 5 as i {
    show i
}
```

The loop variable defaults to `i` if not specified:

```obsidian
repeat from 10 to 15 {
    show i
}
```

Range loops support `break` and `continue`:

```obsidian
repeat from 1 to 100 as n {
    if n is 50 then
        break
    end
    show n
}
```

### 3.13 Error Handling (Try/Catch)

Handle runtime errors gracefully using `try/catch/end`:

```obsidian
try
    delete "nonexistent_file.txt"
catch error
    show "Caught error: {error}"
end
```

The following error types are caught:
- File errors (file not found, permission denied)
- Type mismatches
- Undefined variables
- Runtime errors

Successful operations inside `try` blocks execute normally:

```obsidian
try
    create "temp.txt"
    write "temp.txt" content "Hello"
    read "temp.txt" into content
    show content
    delete "temp.txt"
catch error
    show "Error: {error}"
end
```

### 3.14 Testing

Write tests using `test` blocks with `expect` assertions:

```obsidian
test "addition works"
    set x to 2
    set y to 3
    set result to x add y
    expect result is 5
end

test "string length"
    set s to "hello"
    set len to length of s
    expect len is 5
end
```

Run tests with the `obsidian test` command:

```bash
obsidian test examples/testing.obs
```

Output:
```
✓ addition works
✓ string length
2 tests passed, 0 failed
```

The `expect` keyword compares two values for equality. If they differ, the test fails with an error message showing the expected and actual values.

Tests run in isolated environments — each test starts fresh but shares the global environment state.

---

## 4. Built-in Functions (Standard Library)

### 4.1 Text Operations

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| uppercase | `uppercase of x` | text | `uppercase of "hello"` → `"HELLO"` |
| lowercase | `lowercase of x` | text | `lowercase of "HELLO"` → `"hello"` |
| length | `length of x` | number | `length of "hello"` → `5` |
| trim | `trim of x` | text | `trim of "  hi  "` → `"hi"` |
| reverse | `reverse of x` | text/list | `reverse of "abc"` → `"cba"` |
| contains | `x contains y` | bool | `"hello" contains "ell"` → `true` |
| replace | `replace old with new in text` | text | `replace "cat" with "dog" in sentence` |

### 4.2 Number Operations

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| round | `round x` | number | `round 3.7` → `4` |
| floor | `floor x` | number | `floor 3.7` → `3` |
| ceiling | `ceiling x` | number | `ceiling 3.2` → `4` |
| absolute | `absolute of x` | number | `absolute of -5` → `5` |
| random | `random between low and high` | number | `random between 1 and 10` |

### 4.3 List Operations

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| count | `count of x` | number | `count of list 1, 2, 3` → `3` |
| first | `first of x` | any | `first of list 1, 2, 3` → `1` |
| last | `last of x` | any | `last of list 1, 2, 3` → `3` |
| sort | `sort of x` | list | `sort of list 3, 1, 2` → `list 1, 2, 3` |
| reverse | `reverse of x` | list | `reverse of list 1, 2, 3` → `list 3, 2, 1` |
| length | `length of x` | number | `length of list 1, 2, 3` → `3` |
| contains | `x contains y` | bool | `list 1, 2, 3 contains 2` → `true` |

### 4.4 Type Conversion

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| as text | `x as text` | text | `42 as text` → `"42"` |
| as number | `x as number` | number | `"3.14" as number` → `3.14` |
| as truth | `x as truth` | bool | `1 as truth` → `true` |

### 4.5 Additional Functions (v0.2.0)

| Function | Syntax | Returns | Example |
|----------|--------|---------|---------|
| split | `split text by delimiter` | list | `split "a,b,c" by ","` → `list "a", "b", "c"` |
| join | `join list with separator` | text | `join parts with ","` → `"a,b,c"` |
| power | `power base by exponent` | number | `power 2 by 3` → `8` |
| square root | `square root of x` | number | `square root of 16` → `4` |
| file exists | `file exists path` | bool | `file exists "test.txt"` → `true` |
| list files | `list files in directory` | list | `list files in "."` → `list "src", "docs"` |
| current date | `current date` | text | `current date` → `"2025-01-15"` |
| current time | `current time` | text | `current time` → `"14:30:00"` |

---

## 5. Operator Precedence

From lowest to highest precedence:

1. `or`
2. `and`
3. Comparison: `is`, `!=`, `<`, `>`, `<=`, `>=`, `contains`
4. Addition/Subtraction: `add`, `subtract`
5. Multiplication/Division: `multiply`, `divide`
6. Unary: `not`, built-in prefix operations
7. Primary: literals, identifiers, parenthesized expressions

---

## 6. Truthy/Falsy Values

| Value | Truthy? |
|-------|---------|
| Non-zero number | true |
| Zero (`0`) | false |
| Non-empty string | true |
| Empty string (`""`) | false |
| Non-empty list | true |
| Empty list (`list`) | false |
| `null` | false |

---

## 7. Comparison: Obsidian vs Python

| Task | Obsidian | Python |
|------|----------|--------|
| **Print output** | `show "Hello"` | `print("Hello")` |
| **Variable assignment** | `set x to 42` | `x = 42` |
| **If/else** | `if x is 5 then`<br>`    show "yes"`<br>`otherwise`<br>`    show "no"`<br>`end` | `if x == 5:`<br>`    print("yes")`<br>`else:`<br>`    print("no")` |
| **Loop N times** | `repeat 5 times {`<br>`    show i`<br>`}` | `for i in range(5):`<br>`    print(i)` |
| **Define function** | `define greet with name`<br>`    show "Hi, " + name`<br>`end` | `def greet(name):`<br>`    print("Hi, " + name)` |

---

## 8. Complete Examples

### 8.1 Hello World

```obsidian
show "Hello, World!"
```

### 8.2 FizzBuzz

```obsidian
set i to 0
set c3 to 0
set c5 to 0
set c15 to 0
repeat 15 times {
    set i to i add 1
    set c3 to c3 add 1
    set c5 to c5 add 1
    set c15 to c15 add 1
    if c15 is 15 then
        show "FizzBuzz"
        set c15 to 0
        set c3 to 0
        set c5 to 0
    otherwise
        if c3 is 3 then
            show "Fizz"
            set c3 to 0
        otherwise
            if c5 is 5 then
                show "Buzz"
                set c5 to 0
            otherwise
                show i
            end
        end
    end
}
```

### 8.3 Functions

```obsidian
define factorial with n
    if n is 0 then
        return 1
    otherwise
        return n multiply call factorial with n subtract 1
    end
end

show call factorial with 5
```

### 8.4 String Manipulation

```obsidian
set greeting to "hello, world!"
show uppercase of greeting
show length of greeting
show replace "world" with "Obsidian" in greeting
```
