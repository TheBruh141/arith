# arith: An Arithmetic Expression Evaluator

`arith` is a command-line Read-Eval-Print Loop (REPL) for evaluating arithmetic expressions. It supports basic arithmetic operations, floating-point numbers, scientific notation, and implicit multiplication.

## How to Use

To build and run the `arith` REPL, follow these steps:

1.  **Clone the repository (if you haven't already):**
    ```bash
    git clone <repository_url>
    cd arith
    ```

2.  **Build the project:**
    ```bash
    cargo build
    ```

3.  **Run the REPL:**
    ```bash
    cargo run
    ```

    The REPL will start, and you'll see a `>>` prompt:
    ```
    arith REPL — enter expressions. Use \ for line-continuation. :q to quit.
    >> 
    ```

### REPL Commands:

*   `:q` or `:quit` or `:exit`: Quits the REPL.
*   `:h` or `:help`: Displays help information.

### Line Continuation:

You can continue an expression on the next line by ending the current line with a backslash (`\
`).

## Syntax Specification

`arith` supports the following syntax:

*   **Numbers:** Integers (e.g., `10`, `42`) and floating-point numbers (e.g., `3.14`, `0.5`). Scientific notation is also supported (e.g., `1e-5`, `2.5E+3`).
*   **Operators:**
    *   Addition: `+`
    *   Subtraction: `-`
    *   Multiplication: `*`
    *   Division: `/`
    *   Unary Minus: `-` (e.g., `-5`, `-(2+3)`)
*   **Parentheses:** `()` for grouping expressions and controlling order of operations.
*   **Implicit Multiplication:** A number or a parenthesized expression immediately followed by an opening parenthesis implies multiplication (e.g., `3(5)` is `3 * 5`, `(2+1)(4)` is `(2+1) * 4`).
*   **Comments:** Lines starting with a semicolon (`;`) are treated as comments and are ignored by the evaluator. Comments can also appear after an expression on the same line.

## Examples

```
>> 1 + 2
= 3
>> 3 * (4 - 1)
= 9
>> -5 + 10
= 5
>> 2(3 + 4)
= 14
>> (1 + 1)(5)
= 10
>> 10 / 3
= 3.333333333333333
>> 1e-5 * 100
= 0.001
>> ; This is a comment
>> 5 + 5 ; This is also a comment
= 10
>> 10 + \
... 20
= 30
```

## TODOs

*   **Improve Error Messages:** Provide more specific and user-friendly error messages from the parser and executor, including line and column numbers where applicable.
*   **Enhance REPL Features:** Implement features like command history (up/down arrow keys) and auto-completion for a better user experience.
*   **Optimize Performance:** Investigate and implement performance improvements, particularly in the tokenizer (e.g., using character iterators instead of collecting into a `Vec<char>`).
*   **Handle Floating-Point Precision:** Address potential floating-point precision issues to ensure more accurate results for complex calculations.
*   **Expand Functionality:** Add support for:
    *   Variables
    *   User-defined functions
    *   Additional mathematical operators (e.g., exponentiation, modulo)
    *   Built-in mathematical functions (e.g., `sin`, `cos`, `log`)
*   **Comprehensive Testing:** Add more unit and integration tests to increase code coverage and ensure robustness.
*   **Code Documentation:** Add detailed documentation comments to the source code for better understanding and maintainability.
