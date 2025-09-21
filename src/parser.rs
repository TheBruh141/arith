use crate::ast::{Expr, Statement};
use crate::errors::ParserError;
use crate::tokenizer::{Token, TokenType};

pub struct Parser {
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

    fn consume(&mut self, expected: &TokenType) -> Result<(), ParserError> {
        if self.current().get_type() == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                found: self.current().get_type().clone(),
                line: self.current().get_line_no(),
                col: self.current().get_start(),
            })
        }
    }

    pub fn parse(&mut self) -> Result<Statement, ParserError> {
        if matches!(self.current().get_type(), TokenType::EOF) {
            return Ok(Statement::Expression(Expr::Empty));
        }
        self.parse_statement()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos + 1]
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.current().get_type() {
            TokenType::Let => self.parse_let_statement(),
            TokenType::Identifier { .. } => match self.peek().get_type() {
                TokenType::Assign => self.parse_assignment_statement(),
                TokenType::PlusAssign
                | TokenType::MinusAssign
                | TokenType::MulAssign
                | TokenType::DivAssign => self.parse_compound_assignment_statement(),
                _ => {
                    let expr = self.parse_expr(0)?;
                    Ok(Statement::Expression(expr))
                }
            },
            _ => {
                let expr = self.parse_expr(0)?;
                Ok(Statement::Expression(expr))
            }
        }
    }

    fn parse_compound_assignment_statement(&mut self) -> Result<Statement, ParserError> {
        let name = if let TokenType::Identifier { name } = self.current().get_type() {
            name.clone()
        } else {
            return Err(ParserError::UnexpectedToken {
                found: self.current().get_type().clone(),
                line: self.current().get_line_no(),
                col: self.current().get_start(),
            });
        };
        self.advance();

        let op = self.current().get_type().clone();
        self.advance();

        let value = self.parse_expr(0)?;

        Ok(Statement::CompoundAssignment { name, op, value })
    }

    fn parse_assignment_statement(&mut self) -> Result<Statement, ParserError> {
        let name = if let TokenType::Identifier { name } = self.current().get_type() {
            name.clone()
        } else {
            return Err(ParserError::UnexpectedToken {
                found: self.current().get_type().clone(),
                line: self.current().get_line_no(),
                col: self.current().get_start(),
            });
        };
        self.advance();

        self.consume(&TokenType::Assign)?;

        let value = self.parse_expr(0)?;

        Ok(Statement::Assignment { name, value })
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(&TokenType::Let)?;

        let name = if let TokenType::Identifier { name } = self.current().get_type() {
            name.clone()
        } else {
            return Err(ParserError::UnexpectedToken {
                found: self.current().get_type().clone(),
                line: self.current().get_line_no(),
                col: self.current().get_start(),
            });
        };
        self.advance();

        let mut type_name = None;
        if let TokenType::Colon = self.current().get_type() {
            self.advance();
            if let TokenType::Identifier { name } = self.current().get_type() {
                type_name = Some(name.clone());
                self.advance();
            } else {
                return Err(ParserError::UnexpectedToken {
                    found: self.current().get_type().clone(),
                    line: self.current().get_line_no(),
                    col: self.current().get_start(),
                });
            }
        }

        self.consume(&TokenType::Assign)?;

        let value = self.parse_expr(0)?;

        Ok(Statement::Let {
            name,
            type_name,
            value,
        })
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParserError> {
        let mut left = self.parse_primary()?;

        while self.pos < self.tokens.len() - 1 {
            let op = self.current().get_type().clone();
            let prec = match op {
                TokenType::Plus | TokenType::Minus => 1,
                TokenType::Mul | TokenType::Div => 2,
                TokenType::ParanOpen | TokenType::Number { .. } | TokenType::Identifier { .. } => 3, // Implicit multiplication
                _ => break,
            };

            if prec < min_prec {
                break;
            }

            if matches!(
                op,
                TokenType::ParanOpen | TokenType::Number { .. } | TokenType::Identifier { .. }
            ) {
                let right = self.parse_expr(prec + 1)?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: TokenType::Mul,
                    right: Box::new(right),
                };
            } else {
                self.advance();
                let right = self.parse_expr(prec + 1)?;
                left = Expr::BinaryOp {
                    left: Box::new(left),
                    op: op.clone(),
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let token_type = self.current().get_type().clone();
        self.advance();

        match token_type {
            TokenType::Number { value } => {
                let n: f64 = value.parse().map_err(|_| ParserError::InvalidNumber {
                    value: value.clone(),
                    line: self.current().get_line_no(),
                    col: self.current().get_start(),
                })?;
                Ok(Expr::Number(n))
            }
            TokenType::Identifier { name } => Ok(Expr::Variable(name)),
            TokenType::ParanOpen => {
                if matches!(self.current().get_type(), TokenType::ParanClose) {
                    self.advance();
                    return Ok(Expr::EmptyParen);
                }
                let expr = self.parse_expr(0)?;
                self.consume(&TokenType::ParanClose)?;
                Ok(expr)
            }
            TokenType::Plus | TokenType::Minus => {
                let expr = self.parse_expr(3)?;
                Ok(Expr::UnaryOp {
                    op: token_type,
                    expr: Box::new(expr),
                })
            }
            _ => Err(ParserError::UnexpectedToken {
                found: token_type,
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

    fn parse_ok(input: &str) -> Result<Statement, ParserError> {
        let mut tokenizer = Tokenizer::new(input.to_string());
        let tokens = tokenizer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    fn assert_parse_ok(input: &str, expected: Statement) {
        match parse_ok(input) {
            Ok(stmt) => assert_eq!(stmt, expected),
            Err(e) => panic!("Parsing failed for input '{}': {}", input, e),
        }
    }

    #[test]
    fn test_simple_number() {
        assert_parse_ok("42", Statement::Expression(Expr::Number(42.0)));
    }

    #[test]
    fn test_simple_binary() {
        assert_parse_ok(
            "1+2",
            Statement::Expression(Expr::BinaryOp {
                left: Box::new(Expr::Number(1.0)),
                op: TokenType::Plus,
                right: Box::new(Expr::Number(2.0)),
            }),
        );
    }

    #[test]
    fn test_let_statement_untyped() {
        assert_parse_ok(
            "let x = 42",
            Statement::Let {
                name: "x".to_string(),
                type_name: None,
                value: Expr::Number(42.0),
            },
        );
    }

    #[test]
    fn test_let_statement_typed() {
        assert_parse_ok(
            "let y: Int = 3.14",
            Statement::Let {
                name: "y".to_string(),
                type_name: Some("Int".to_string()),
                value: Expr::Number(3.14),
            },
        );
    }

    #[test]
    fn test_variable_reference() {
        assert_parse_ok("x", Statement::Expression(Expr::Variable("x".to_string())));
    }

    #[test]
    fn test_complex_expression() {
        assert_parse_ok(
            "let result = 1 + 2 * 3",
            Statement::Let {
                name: "result".to_string(),
                type_name: None,
                value: Expr::BinaryOp {
                    left: Box::new(Expr::Number(1.0)),
                    op: TokenType::Plus,
                    right: Box::new(Expr::BinaryOp {
                        left: Box::new(Expr::Number(2.0)),
                        op: TokenType::Mul,
                        right: Box::new(Expr::Number(3.0)),
                    }),
                },
            },
        );
    }
}
