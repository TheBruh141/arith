# arith: A Simple Command-Line Arithmetic Interpreter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

`arith` is a lightweight and efficient command-line interpreter for arithmetic expressions, built with Rust. It provides a simple yet powerful way to evaluate mathematical expressions directly from your terminal, supporting basic operations, operator precedence, implicit multiplication, and an interactive Read-Eval-Print Loop (REPL).

## ✨ Features

*   **Basic Arithmetic Operations:** Supports addition (`+`), subtraction (`-`), multiplication (`*`), and division (`/`).
*   **Operator Precedence:** Correctly evaluates expressions based on standard mathematical operator precedence rules.
*   **Implicit Multiplication:** Automatically interprets expressions like `3(5)` or `(2)(3)` as multiplication.
*   **Unary Operators:** Handles unary plus (`+`) and minus (`-`).
*   **Parentheses Support:** Allows grouping of expressions for explicit control over evaluation order.
*   **Interactive REPL:** A user-friendly Read-Eval-Print Loop for real-time expression evaluation.
*   **File Mode:** Evaluate expressions from one or more input files.
*   **Line Continuations:** Use `\` to continue expressions across multiple lines in the REPL or input files.
*   **Comments:** Supports single-line comments starting with `;`.
*   **Robust Error Handling:** Provides clear and informative error messages with line and column details.

## 🚀 Installation

To build and run `arith`, you need to have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/YOUR_USERNAME/arith.git
    cd arith
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The executable will be located in `target/release/arith`.

3.  **Add to PATH (Optional):** For easier access, you can add the `target/release` directory to your system's PATH.
    ```bash
    # On Linux/macOS
    export PATH="$PATH:$(pwd)/target/release"
    # On Windows (PowerShell)
    $env:Path += ";$(Get-Location)\target\release"
    ```

## 💡 Usage

### Interactive REPL

Run `arith` without any arguments to start the interactive REPL:

```bash
arith
```

You can then type expressions and press Enter to evaluate them:

```
>> 1 + 2 * 3
= 7
>> (1 + 2) * 3
= 9
>> 3(5) ; Implicit multiplication
= 15
>> 10 / (2 + 3)
= 2
>> 1 + \
... 2
= 3
>> :q ; Type :q or :quit to exit
```

### File Mode

Evaluate expressions from one or more files:

```bash
arith -f input.arith another.arith
```

Example `input.arith`:

```arith
10 + 5
(2 + 3) * 4 ; This is a comment
100 / 25
```

Output:

```
--- Results from input.arith ---
10 + 5 [1]: 15
(2 + 3) * 4 [2]: 20
100 / 25 [3]: 4

--- Results from another.arith ---
...
```

### Supported Syntax

`arith` supports a straightforward syntax for arithmetic expressions:

*   **Numbers:** Integers (`123`), floating-point numbers (`3.14`), and scientific notation (`1e-5`, `2.5E+3`).
*   **Operators:** `+`, `-`, `*`, `/`.
*   **Parentheses:** `()` for grouping.
*   **Implicit Multiplication:** `3(5)`, `(2)(3)`, `5(1+1)`.
*   **Comments:** Start with `;` and extend to the end of the line.
*   **Line Continuations:** End a line with `\` to continue the expression on the next line.

## 🤝 Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. If you'd like to contribute code, please fork the repository and submit a pull request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Contact

For questions or support, please open an issue on the [GitHub repository](https://github.com/YOUR_USERNAME/arith).