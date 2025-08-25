use crate::errors::ParserError;
use crate::tokenizer::{Token, TokenType};

/// Represents a node in the Abstract Syntax Tree (AST).
///
/// The AST is a tree representation of the grammatical structure of the
/// source code. Each variant of this enum corresponds to a different type of
/// expression in the `arith` language.
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// A literal floating-point number, e.g., `42.0`, `3.14`.
    Number(f64),

    /// A unary operation, e.g., `-5`, `+x`.
    ///
    /// It consists of an operator (`op`) and an expression (`expr`) that it
    /// applies to.
    UnaryOp { op: TokenType, expr: Box<Expr> },

    /// A binary operation, e.g., `a + b`, `c * d`.
    ///
    /// It consists of a left-hand side expression (`left`), an operator (`op`),
    /// and a right-hand side expression (`right`).
    BinaryOp {
        left: Box<Expr>,
        op: TokenType,
        right: Box<Expr>,
    },

    /// Represents an empty expression, typically from an empty input string.
    Empty,

    /// Represents empty parentheses, e.g., `()`. In `arith`, this evaluates to `0`.
    EmptyParen,
}

/// The `Parser` is responsible for syntactic analysis. It consumes a stream of
/// `Token`s from the `Tokenizer` and produces an Abstract Syntax Tree (AST)
/// that represents the grammatical structure of the input expression.
///
/// The parser implements a top-down recursive descent strategy, specifically
/// a Pratt parser, to handle operator precedence and associativity correctly.
///
/// The grammar rules are applied in the parsing methods:
/// - `parse_expr`: Handles the lowest precedence operators (`+`, `-`).
/// - `parse_term`: Handles higher precedence operators (`*`, `/`) and implicit
///   multiplication.
/// - `parse_factor`: Handles the highest precedence elements, including numbers,
///   parenthesized expressions, and unary operators.
pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Creates a new `Parser`.
    ///
    /// It filters out comments and newlines from the token stream, as they are
    /// not relevant to the parsing logic.
    pub fn new(tokens: Vec<Token>) -> Self {
        let filtered_tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t.get_type(), TokenType::Comment { .. } | TokenType::Newline))
            .collect();
        Parser {
            tokens: filtered_tokens,
            pos: 0,
        }
    }

    /// Returns the current token without consuming it.
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// Advances the parser to the next token.
    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    /// Parses the entire token stream and returns the root of the AST.
    ///
    /// If the input is empty (only an EOF token), it returns `Expr::Empty`.
    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        if matches!(self.current().get_type(), TokenType::EOF) {
            return Ok(Expr::Empty);
        }
        self.parse_expr()
    }

    /// Parses expressions with the lowest precedence (addition and subtraction).
    ///
    /// This method forms the entry point for parsing expressions and handles
    /// left-associative binary operators `+` and `-`.
    ///
    /// Grammar rule: `expression = term, { (PLUS | MINUS), term } `;
    fn parse_expr(&mut self) -> Result<Expr, ParserError> {
        let mut node = self.parse_term()?;

        while matches!(
            self.current().get_type(),
            TokenType::Plus | TokenType::Minus
        ) {
            let op = self.current().get_type().clone();
            self.advance();
            let right = self.parse_term()?;
            node = Expr::BinaryOp {
                left: Box::new(node),
                op,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    /// Parses expressions with higher precedence (multiplication and division).
    ///
    /// This method handles left-associative binary operators `*` and `/`, as
    /// well as implicit multiplication (e.g., `3(5)` or `(2)(3)`).
    ///
    /// Grammar rule: `term = factor, { (MUL | DIV), factor | factor } `;
    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let mut node = self.parse_factor()?;

        while matches!(
            self.current().get_type(),
            TokenType::Mul | TokenType::Div | TokenType::ParanOpen | TokenType::Number { .. }
        ) {
            if matches!(self.current().get_type(), TokenType::ParanOpen) || matches!(
                self.current().get_type(),
                TokenType::Number { .. }
            ) {
                // Implicit multiplication has the same precedence as explicit multiplication.
                // e.g., `3(5)` is parsed as `3 * 5`.
                let right = self.parse_factor()?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op: TokenType::Mul,
                    right: Box::new(right),
                };
            } else {
                // Explicit multiplication or division.
                let op = self.current().get_type().clone();
                self.advance();
                let right = self.parse_factor()?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op,
                    right: Box::new(right),
                };
            }
        }

        Ok(node)
    }

    /// Parses the highest precedence expressions (factors).
    ///
    /// Factors include numbers, parenthesized expressions, and unary operators.
    ///
    /// Grammar rule:
    /// `factor = NUMBER | LPAREN, [expression], RPAREN | (PLUS | MINUS), factor `;
    fn parse_factor(&mut self) -> Result<Expr, ParserError> {
        match &self.current().get_type() {
            // Unary plus and minus operators.
            TokenType::Plus | TokenType::Minus => {
                let op = self.current().get_type().clone();
                self.advance();
                let expr = self.parse_factor()?;
                Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(expr),
                })
            }
            // Literal numbers.
            TokenType::Number { value } => {
                let n: f64 = value.parse().map_err(|_| ParserError::InvalidNumber {
                    value: value.clone(),
                    line: self.current().get_line_no(),
                    col: self.current().get_start(),
                })?;
                self.advance();
                Ok(Expr::Number(n))
            }
            // Parenthesized expressions.
            TokenType::ParanOpen => {
                self.advance();
                // Handle empty parentheses `()`.
                if matches!(self.current().get_type(), TokenType::ParanClose) {
                    self.advance();
                    return Ok(Expr::EmptyParen);
                }
                let expr = self.parse_expr()?;
                // Ensure the expression is followed by a closing parenthesis.
                if !matches!(self.current().get_type(), TokenType::ParanClose) {
                    return Err(ParserError::UnexpectedToken {
                        found: self.current().get_type().clone(),
                        line: self.current().get_line_no(),
                        col: self.current().get_start(),
                    });
                }
                self.advance();
                Ok(expr)
            }
            // Handle unexpected tokens.
            _ => Err(ParserError::UnexpectedToken {
                found: self.current().get_type().clone(),
                line: self.current().get_line_no(),
                col: self.current().get_start(),
            }),
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::tokenizer::Tokenizer;

    fn parse_ok(input: &str) -> Result<Expr, ParserError> {
        let mut tokenizer = Tokenizer::new(input.to_string());
        let tokens = tokenizer.tokenize().unwrap(); // This unwrap is in a test, which is acceptable.
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn assert_parse_ok(input: &str, expected: Expr) {
        match parse_ok(input) {
            Ok(expr) => assert_eq!(expr, expected),
            Err(e) => panic!("Parsing failed for input '{}': {}", input, e),
        }
    }

    #[test]
    fn test_simple_number() {
        assert_parse_ok("42", Expr::Number(42.0));
    }

    #[test]
    fn test_simple_binary() {
        assert_parse_ok(
            "1+2",
            Expr::BinaryOp {
                left: Box::new(Expr::Number(1.0)),
                op: TokenType::Plus,
                right: Box::new(Expr::Number(2.0)),
            },
        );
    }

    #[test]
    fn test_unary_positive() {
        assert_parse_ok(
            "+5",
            Expr::UnaryOp {
                op: TokenType::Plus,
                expr: Box::new(Expr::Number(5.0)),
            },
        );
    }

    #[test]
    fn test_unary_negative() {
        assert_parse_ok(
            "-7",
            Expr::UnaryOp {
                op: TokenType::Minus,
                expr: Box::new(Expr::Number(7.0)),
            },
        );
    }

    #[test]
    fn test_unary_chain() {
        assert_parse_ok(
            "--3",
            Expr::UnaryOp {
                op: TokenType::Minus,
                expr: Box::new(Expr::UnaryOp {
                    op: TokenType::Minus,
                    expr: Box::new(Expr::Number(3.0)),
                }),
            },
        );
    }

    #[test]
    fn test_unary_with_binary() {
        assert_parse_ok(
            "-2+3",
            Expr::BinaryOp {
                left: Box::new(Expr::UnaryOp {
                    op: TokenType::Minus,
                    expr: Box::new(Expr::Number(2.0)),
                }),
                op: TokenType::Plus,
                right: Box::new(Expr::Number(3.0)),
            },
        );
    }

    #[test]
    fn test_parentheses() {
        assert_parse_ok(
            "(1+2)*3",
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Number(1.0)),
                    op: TokenType::Plus,
                    right: Box::new(Expr::Number(2.0)),
                }),
                op: TokenType::Mul,
                right: Box::new(Expr::Number(3.0)),
            },
        );
    }

    #[test]
    fn test_unary_inside_parentheses() {
        assert_parse_ok(
            "-(2+3)",
            Expr::UnaryOp {
                op: TokenType::Minus,
                expr: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Number(2.0)),
                    op: TokenType::Plus,
                    right: Box::new(Expr::Number(3.0)),
                }),
            },
        );
    }

    #[test]
    fn test_float_numbers() {
        assert_parse_ok(
            "3.14*2.0",
            Expr::BinaryOp {
                left: Box::new(Expr::Number(3.14)),
                op: TokenType::Mul,
                right: Box::new(Expr::Number(2.0)),
            },
        );
    }

    #[test]
    fn test_mixed_unary_binary_parentheses() {
        assert_parse_ok(
            "-(-1+2)*+3",
            Expr::BinaryOp {
                left: Box::new(Expr::UnaryOp {
                    op: TokenType::Minus,
                    expr: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::UnaryOp {
                            op: TokenType::Minus,
                            expr: Box::new(Expr::Number(1.0)),
                        }),
                        op: TokenType::Plus,
                        right: Box::new(Expr::Number(2.0)),
                    }),
                }),
                op: TokenType::Mul,
                right: Box::new(Expr::UnaryOp {
                    op: TokenType::Plus,
                    expr: Box::new(Expr::Number(3.0)),
                }),
            },
        );
    }

    #[test]
    fn test_trailing_unary_error() {
        let result = parse_ok("-");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let mut tokenizer = Tokenizer::new("".to_string());
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(expr) => assert_eq!(expr, Expr::Empty),
            Err(e) => panic!("Parsing failed: {}", e),
        }
    }

    #[test]
    fn test_complex_expression() {
        match parse_ok("1+2*(3-4)/-5") {
            Ok(expr) => assert!(matches!(expr, Expr::BinaryOp { .. })),
            Err(e) => panic!("Parsing failed: {}", e),
        }
    }

    #[test]
    fn test_line_starting_with_comment() {
        assert_parse_ok(
            ";this is a comment\n1+2",
            Expr::BinaryOp {
                left: Box::new(Expr::Number(1.0)),
                op: TokenType::Plus,
                right: Box::new(Expr::Number(2.0)),
            },
        );
    }

    #[test]
    fn test_implicit_multiplication() {
        assert_parse_ok(
            "3(5)",
            Expr::BinaryOp {
                left: Box::new(Expr::Number(3.0)),
                op: TokenType::Mul,
                right: Box::new(Expr::Number(5.0)),
            },
        );
    }

    #[test]
    fn test_implicit_multiplication_with_parens() {
        assert_parse_ok(
            "(3)(5)",
            Expr::BinaryOp {
                left: Box::new(Expr::Number(3.0)),
                op: TokenType::Mul,
                right: Box::new(Expr::Number(5.0)),
            },
        );
    }

    #[test]
    fn test_implicit_multiplication_between_parens() {
        assert_parse_ok(
            "(1+2)(3+4)",
            Expr::BinaryOp {
                left: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Number(1.0)),
                    op: TokenType::Plus,
                    right: Box::new(Expr::Number(2.0)),
                }),
                op: TokenType::Mul,
                right: Box::new(Expr::BinaryOp {
                    left: Box::new(Expr::Number(3.0)),
                    op: TokenType::Plus,
                    right: Box::new(Expr::Number(4.0)),
                }),
            },
        );
    }
}
