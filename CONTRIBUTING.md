# Contributing to arith

We welcome contributions to the `arith` project! By participating, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

1.  **Fork the Repository:** Start by forking the `arith` repository to your GitHub account.
2.  **Clone Your Fork:** Clone your forked repository to your local machine:
    ```bash
    git clone https://github.com/YOUR_USERNAME/arith.git
    cd arith
    ```
3.  **Create a New Branch:** Create a new branch for your feature or bug fix:
    ```bash
    git checkout -b feature/your-feature-name
    # or
    git checkout -b bugfix/your-bug-fix
    ```
4.  **Make Your Changes:** Implement your changes. Ensure your code adheres to the existing style and conventions.
5.  **Run Tests:** Before submitting, make sure all existing tests pass and add new tests for your changes if applicable.
    ```bash
    cargo test
    ```
6.  **Format and Lint:** Ensure your code is properly formatted and passes lint checks.
    ```bash
    cargo fmt --all
    cargo clippy --all-targets -- -D warnings
    ```
7.  **Commit Your Changes:** Write clear and concise commit messages.
    ```bash
    git commit -m "feat: Add a new feature"
    # or
    git commit -m "fix: Resolve bug in X"
    ```
8.  **Push to Your Fork:** Push your new branch to your forked repository:
    ```bash
    git push origin feature/your-feature-name
    ```
9.  **Create a Pull Request:** Go to the original `arith` repository on GitHub and open a new pull request from your forked branch. Provide a clear description of your changes.

## Code Style

*   Follow Rust's official style guidelines (enforced by `cargo fmt`).
*   Use `cargo clippy` to catch common mistakes and improve code quality.

## Reporting Bugs

If you find a bug, please open an issue on the [GitHub Issues page](https://github.com/TheBruh141/arith/issues). Provide as much detail as possible, including steps to reproduce the bug, expected behavior, and actual behavior.

## Feature Requests

If you have an idea for a new feature, please open an issue on the [GitHub Issues page](https://github.com/TheBruh141/arith/issues). Describe your idea and why you think it would be a valuable addition.
