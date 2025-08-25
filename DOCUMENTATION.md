# Arith Language and Compiler Documentation

## 1. Introduction

`arith` is a simple, command-line based arithmetic language and its associated Read-Eval-Print Loop (REPL). It is designed to evaluate arithmetic expressions with support for basic operations, floating-point numbers, and a few syntactic conveniences.

The primary purpose of this project is to serve as a clear and concise example of a compiler pipeline, demonstrating the fundamental concepts of lexical analysis, parsing, abstract syntax tree (AST) generation, bytecode compilation, and execution on a stack-based virtual machine. It is an excellent resource for anyone looking to understand the inner workings of a programming language interpreter/compiler.

## 2. Syntax Specification

The `arith` language supports a straightforward syntax for arithmetic expressions.

### 2.1. Numbers

Numbers can be integers, floating-point numbers, or in scientific notation.

-   **Integers**: e.g., `10`, `42`, `1000`
-   **Floating-point numbers**: e.g., `3.14`, `0.5`, `2.71828`
-   **Scientific notation**: e.g., `1e-5`, `2.5E+3`, `6.022e23`

### 2.2. Operators

The language supports the four basic arithmetic operations:

-   **Addition**: `+`
-   **Subtraction**: `-`
-   **Multiplication**: `*`
-   **Division**: `/`

### 2.3. Operator Precedence and Associativity

The operators follow standard mathematical precedence and associativity:
-   `*` and `/` have higher precedence than `+` and `-`.
-   All operators are left-associative.

For example, `1 + 2 * 3` is evaluated as `1 + (2 * 3) = 7`.

### 2.4. Parentheses

Parentheses `()` can be used to group expressions and override the default operator precedence.

-   e.g., `(1 + 2) * 3` evaluates to `9`.

Empty parentheses `()` are a valid expression and evaluate to `0`.

### 2.5. Unary Operators

The `+` and `-` operators can be used as unary operators (i.e., to indicate the sign of a number).

-   **Unary Minus**: e.g., `-5`, `-(2+3)`
-   **Unary Plus**: e.g., `+5`, `+(2+3)` (Unary plus has no effect on the value).

### 2.6. Implicit Multiplication

Implicit multiplication is supported when a number or a parenthesized expression is immediately followed by an opening parenthesis.

-   e.g., `3(5)` is equivalent to `3 * 5`.
-   e.g., `(2+1)(4)` is equivalent to `(2+1) * 4`.

### 2.7. Comments

Comments start with a semicolon `;` and continue to the end of the line. They are ignored by the evaluator.

-   e.g., `; this is a comment`
-   e.g., `5 + 5 ; this is also a comment`

### 2.8. Whitespace

Whitespace characters (spaces, tabs) are ignored, except for newlines which terminate an expression (unless escaped).

## 3. EBNF Grammar

The following EBNF (Extended Backus-Naur Form) grammar formally defines the syntax of the `arith` language. The grammar is designed to be read from top to bottom, with each rule defining a part of the language's structure.

```ebnf
(* The entry point for an expression. Handles addition and subtraction. *)
expression      = term, { (PLUS | MINUS), term } ;

(* Handles multiplication, division, and implicit multiplication. *)
term            = factor, { (MUL | DIV), factor | LPAREN, expression, RPAREN } ;

(* Handles numbers, parenthesized expressions, and unary operators. *)
factor          = NUMBER |
                  LPAREN, [expression], RPAREN |
                  (PLUS | MINUS), factor ;

(* Defines the format of a number, including integers, floats, and scientific notation. *)
NUMBER          = digit, { digit }, [ ".", { digit } ], [ ('e' | 'E'), [PLUS | MINUS], digit, { digit } ] ;
digit           = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' ;

(* Terminal symbols for operators and parentheses. *)
PLUS            = '+' ;
MINUS           = '-' ;
MUL             = '*' ;
DIV             = '/' ;
LPAREN          = '(' ;
RPAREN          = ')' ;
```

**Explanation of the Grammar:**

*   **`expression`**: This is the top-level rule. It defines an expression as a sequence of one or more `term`s separated by `+` or `-` operators. This handles the lowest precedence operations and ensures left-associativity.
*   **`term`**: This rule handles multiplication, division, and implicit multiplication. It's defined as a sequence of `factor`s separated by `*` or `/` operators. The `| LPAREN, expression, RPAREN` part of the rule is a more explicit way to show that a `factor` can be followed by a parenthesized expression to indicate implicit multiplication.
*   **`factor`**: This rule handles the highest precedence elements. A `factor` can be a `NUMBER`, a full `expression` enclosed in parentheses, or a `factor` preceded by a unary `+` or `-` operator.
*   **`NUMBER`**: This rule defines the lexical structure of a number, including integers, decimals, and scientific notation.
*   Terminals: `PLUS`, `MINUS`, `MUL`, `DIV`, `LPAREN`, and `RPAREN` are the terminal symbols, representing the literal characters in the input.


## 4. Technical Overview

The `arith` compiler is implemented in Rust and follows a classic multi-stage pipeline to process and evaluate expressions.

### 4.1. Architecture and Working Outline

The evaluation of an `arith` expression goes through the following stages:

1.  **REPL (`main.rs`)**: The `run_repl` function provides the interactive command-line interface. It reads user input, handles REPL commands (like `:q`), and manages multi-line statements.

2.  **Line Preprocessing (`executor.rs`)**: The `evaluate_lines` function first preprocesses the input string to handle line continuations (lines ending with `\`).

3.  **Tokenizer (`tokenizer.rs`)**: The `Tokenizer` performs *lexical analysis*. It takes the raw input string and breaks it down into a sequence of `Token`s. Each token represents a single lexical unit, such as a number, an operator, or a parenthesis.

4.  **Parser (`parser.rs`)**: The `Parser` performs *syntactic analysis*. It consumes the stream of tokens from the tokenizer and constructs an **Abstract Syntax Tree (AST)**. The AST is a tree-like data structure (`Expr` enum) that represents the grammatical structure of the expression. The parser is responsible for handling operator precedence and associativity.

5.  **Bytecode Compiler (`executor.rs`)**: The `BytecodeCompiler` traverses the AST and compiles it into a linear sequence of simple instructions, known as **bytecode**. This process is often called "lowering" the AST.

6.  **Executor (`executor.rs`)**: The `SimpleExecutor` is a **stack-based virtual machine** that executes the bytecode generated by the compiler. It uses a stack to hold intermediate values during computation and produces the final result.

### 4.2. Core Data Structures

-   **`Token` / `TokenType` (`tokenizer.rs`)**: These structs represent the tokens produced by the tokenizer. `TokenType` is an enum that defines the kind of token (e.g., `Plus`, `Number`, `ParanOpen`).

-   **`Expr` (`parser.rs`)**: This enum defines the nodes of the Abstract Syntax Tree. It has variants for numbers, unary operations, binary operations, and empty expressions.

-   **`Instr` (`executor.rs`)**: This enum defines the bytecode instructions for the stack machine, such as `Push(f64)`, `Add`, `Sub`, `Mul`, `Div`, and `Neg`.

### 4.3. Error Handling

The compiler features a robust error handling system with distinct error types for each stage of the pipeline:

-   **`TokenizerError`**: For lexical errors, like encountering an unexpected character.
-   **`ParserError`**: For syntax errors, such as an unexpected token or an invalid number format.
-   **`CompileError`**: For errors during bytecode compilation, like an unsupported operator in the AST.
-   **`ExecError`**: For runtime errors during execution, such as division by zero or stack underflow.

These errors are wrapped in a top-level `EvalError` enum, which provides detailed, user-friendly error messages, including the line and column number of the error.

## 5. How to Use

### 5.1. Building and Running the REPL

1.  **Build the project:**
    ```bash
    cargo build
    ```

2.  **Run the REPL:**
    ```bash
    cargo run
    ```

### 5.2. REPL Usage

-   Enter an arithmetic expression at the `>>` prompt.
-   The result will be printed with an `=` prefix.
-   To quit the REPL, type `:q`, `:quit`, or `:exit`.
-   For help, type `:h` or `:help`.
-   To continue an expression on the next line, end the current line with a backslash (`\`).

## 6. Testing Strategy

The `arith` project has a comprehensive test suite located in the `tests/` directory. The tests are crucial for ensuring the correctness and robustness of the compiler.

-   **Unit Tests**: The `tokenizer` and `parser` modules have extensive unit tests that cover a wide range of valid and invalid inputs.
-   **Integration Tests**: The `executor` is tested with integration tests that evaluate full expressions and verify the results. These tests cover the entire pipeline from tokenization to execution.

To run the tests, use the following command:
```bash
cargo test
```

## 7. Future Improvements

The `arith` project has several potential areas for future development:

-   **Enhanced REPL**: Implement features like command history (up/down arrows) and auto-completion.
-   **Variables**: Add support for variables to store and reuse values.
-   **Functions**: Introduce user-defined and built-in mathematical functions (e.g., `sin`, `cos`, `log`).
-   **More Operators**: Expand the language with more operators like exponentiation (`^`) and modulo (`%`).
-   **Improved Error Messages**: Make error messages even more specific and helpful.