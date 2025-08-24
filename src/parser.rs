use crate::tokenizer::{Token, TokenType};
use crate::errors::ParserError;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Number(f64),
    UnaryOp {
        op: TokenType,
        expr: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        op: TokenType,
        right: Box<Expr>,
    },
    Empty,
}
pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
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

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParserError> {
        if matches!(self.current().get_type(), TokenType::EOF) {
            return Ok(Expr::Empty);
        }
        self.parse_expr()
    }

    // Parse addition and subtraction
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

    // Parse multiplication and division
    fn parse_term(&mut self) -> Result<Expr, ParserError> {
        let mut node = self.parse_factor()?;

        while matches!(
            self.current().get_type(),
            TokenType::Mul | TokenType::Div | TokenType::ParanOpen
        ) {
            if matches!(self.current().get_type(), TokenType::ParanOpen) {
                // Implicit multiplication
                let right = self.parse_factor()?;
                node = Expr::BinaryOp {
                    left: Box::new(node),
                    op: TokenType::Mul,
                    right: Box::new(right),
                };
            } else {
                // Explicit multiplication or division
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

    // Parse numbers, parentheses, and unary operators
    fn parse_factor(&mut self) -> Result<Expr, ParserError> {
        match &self.current().get_type() {
            TokenType::Plus | TokenType::Minus => {
                let op = self.current().get_type().clone();
                self.advance();
                let expr = self.parse_factor()?;
                Ok(Expr::UnaryOp {
                    op,
                    expr: Box::new(expr),
                })
            }
            TokenType::Number { value } => {
                let n: f64 = value
                    .parse()
                    .map_err(|_| ParserError::InvalidNumber(value.clone()))?;
                self.advance();
                Ok(Expr::Number(n))
            }
            TokenType::ParanOpen => {
                self.advance();
                let expr = self.parse_expr()?;
                if !matches!(self.current().get_type(), TokenType::ParanClose) {
                    return Err(ParserError::UnexpectedToken(self.current().get_type().clone()));
                }
                self.advance();
                Ok(expr)
            }
            _ => Err(ParserError::UnexpectedToken(self.current().get_type().clone())),
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
}