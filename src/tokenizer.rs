use crate::errors::TokenizerError;
use std::fmt::{Debug, Display, Formatter};

/// Represents the type of `Token` in the `arith` language.
///
/// Each variant corresponds to a different lexical unit, such as an operator,
/// a number, or a parenthesis.
#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    /// A newline character `\n`.
    Newline,

    // 4 basic operations
    /// The addition operator `+`.
    Plus,
    /// The subtraction operator `-`.
    Minus,
    /// The division operator `/`.
    Div,
    /// The multiplication operator `*`.
    Mul,

    /// An opening parenthesis `(`.
    ParanOpen,
    /// A closing parenthesis `)`.
    ParanClose,

    /// A comment, starting with `;` and extending to the end of the line.
    Comment { contents: String },
    /// A number literal, which can be an integer, a float, or in scientific notation.
    Number { value: String },
    /// Represents the end of the input string.
    EOF,

    // Variable-related tokens
    /// The `let` keyword.
    Let,
    /// An identifier, such as a variable name.
    Identifier { name: String },
    /// The assignment operator `=`.
    Assign,
    /// The colon operator `:` used for type annotations.
    Colon,
    /// The `+=` operator.
    PlusAssign,
    /// The `-=` operator.
    MinusAssign,
    /// The `*=` operator.
    MulAssign,
    /// The `/=` operator.
    DivAssign,
}

/// Represents a token, a single lexical unit of the `arith` language.
///
/// A token has a `token_type`, and its location in the source code is tracked
/// by `line_no`, `start`, and `end` column positions.
#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    line_no: usize,
    start: usize,
    end: usize,
}

impl Token {
    /// Creates a new `Token`.
    pub fn new(token_type: TokenType, line_no: usize, start: usize, end: usize) -> Token {
        Token {
            token_type,
            line_no,
            start,
            end,
        }
    }

    // Getters for token properties.
    pub fn get_type(&self) -> &TokenType {
        &self.token_type
    }
    pub fn get_line_no(&self) -> usize {
        self.line_no
    }
    pub fn get_start(&self) -> usize {
        self.start
    }
    pub fn get_end(&self) -> usize {
        self.end
    }

    // Helper methods for creating tokens of a specific type.
    pub fn plus(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Plus, line_no, pos, pos)
    }
    pub fn minus(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Minus, line_no, pos, pos)
    }
    pub fn div(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Div, line_no, pos, pos)
    }
    pub fn mul(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Mul, line_no, pos, pos)
    }
    pub fn paran_open(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::ParanOpen, line_no, pos, pos)
    }
    pub fn paran_close(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::ParanClose, line_no, pos, pos)
    }
    pub fn comment(contents: &str, line_no: usize, start: usize) -> Token {
        Token::new(
            TokenType::Comment {
                contents: contents.to_string(),
            },
            line_no,
            start,
            start + contents.len(),
        )
    }
    pub fn number(value: &str, line_no: usize, start: usize) -> Token {
        Token::new(
            TokenType::Number {
                value: value.to_string(),
            },
            line_no,
            start,
            start + value.len(),
        )
    }
    pub fn let_token(line_no: usize, start: usize, end: usize) -> Token {
        Token::new(TokenType::Let, line_no, start, end)
    }

    pub fn identifier(name: &str, line_no: usize, start: usize, end: usize) -> Token {
        Token::new(
            TokenType::Identifier {
                name: name.to_string(),
            },
            line_no,
            start,
            end,
        )
    }

    pub fn colon(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Colon, line_no, pos, pos)
    }

    pub fn assign(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::Assign, line_no, pos, pos)
    }
    pub fn plus_assign(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::PlusAssign, line_no, pos, pos)
    }
    pub fn minus_assign(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::MinusAssign, line_no, pos, pos)
    }
    pub fn mul_assign(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::MulAssign, line_no, pos, pos)
    }
    pub fn div_assign(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::DivAssign, line_no, pos, pos)
    }
    pub fn eof(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::EOF, line_no, pos, pos)
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Newline => write!(f, "Newline\n"),
            TokenType::Plus => write!(f, "Plus\n"),
            TokenType::Minus => write!(f, "Minus\n"),
            TokenType::Div => write!(f, "Div\n"),
            TokenType::Mul => write!(f, "Mul\n"),
            TokenType::ParanOpen => write!(f, "ParanOpen\n"),
            TokenType::ParanClose => write!(f, "ParanClose\n"),
            TokenType::Comment { contents } => write!(f, "Comment: {}\n", contents),
            TokenType::Number { value } => write!(f, "Number({})\n", value),
            TokenType::EOF => write!(f, "eof\n"),
            TokenType::Let => write!(f, "Let\n"),
            TokenType::Identifier { name } => write!(f, "Identifier({})\n", name),
            TokenType::Assign => write!(f, "Assign\n"),
            TokenType::Colon => write!(f, "Colon\n"),
            TokenType::PlusAssign => write!(f, "PlusAssign\n"),
            TokenType::MinusAssign => write!(f, "MinusAssign\n"),
            TokenType::MulAssign => write!(f, "MulAssign\n"),
            TokenType::DivAssign => write!(f, "DivAssign\n"),
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, l_no: {}, s: {}, e:{}",
            self.token_type, self.line_no, self.start, self.end
        )
    }
}

/// The `Tokenizer` is responsible for lexical analysis. It takes a raw string
/// input and breaks it down into a sequence of `Token`s.
///
/// It handles different types of tokens, including operators, numbers (integers,
/// floats, scientific notation), parentheses, comments, and whitespace.
pub struct Tokenizer {
    content: String,
    tokens: Vec<Token>,
}

impl Debug for Tokenizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tokenizer {{ ")?;
        write!(f, "content:\n{}\n", self.content)?;
        write!(f, "tokens:\n")?;
        for token in &self.tokens {
            write!(f, "\t{}\n", token)?;
        }
        write!(f, "}}")
    }
}
impl Tokenizer {
    /// Creates a new `Tokenizer` with the given input content.
    pub fn new(content: String) -> Tokenizer {
        Tokenizer {
            content,
            tokens: Vec::new(),
        }
    }

    /// Performs the tokenization of the input string.
    ///
    /// It iterates through the characters of the input string and constructs a
    /// vector of `Token`s. It returns a `Result` containing the vector of tokens
    /// or a `TokenizerError` if an unexpected character is found.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let chars: Vec<char> = self.content.chars().collect();
        let len = chars.len();

        let mut tokens: Vec<Token> = Vec::new();
        let mut i = 0;
        let mut line_no = 0;
        let mut col = 0;

        while i < len {
            let c = chars[i];

            match c {
                '+' => {
                    if i + 1 < len && chars[i + 1] == '=' {
                        tokens.push(Token::plus_assign(line_no + 1, col + 1));
                        i += 2;
                        col += 2;
                    } else {
                        tokens.push(Token::plus(line_no + 1, col + 1));
                        i += 1;
                        col += 1;
                    }
                }
                '-' => {
                    if i + 1 < len && chars[i + 1] == '=' {
                        tokens.push(Token::minus_assign(line_no + 1, col + 1));
                        i += 2;
                        col += 2;
                    } else {
                        tokens.push(Token::minus(line_no + 1, col + 1));
                        i += 1;
                        col += 1;
                    }
                }
                '*' => {
                    if i + 1 < len && chars[i + 1] == '=' {
                        tokens.push(Token::mul_assign(line_no + 1, col + 1));
                        i += 2;
                        col += 2;
                    } else {
                        tokens.push(Token::mul(line_no + 1, col + 1));
                        i += 1;
                        col += 1;
                    }
                }
                '/' => {
                    if i + 1 < len && chars[i + 1] == '=' {
                        tokens.push(Token::div_assign(line_no + 1, col + 1));
                        i += 2;
                        col += 2;
                    } else {
                        tokens.push(Token::div(line_no + 1, col + 1));
                        i += 1;
                        col += 1;
                    }
                }
                '(' => {
                    tokens.push(Token::paran_open(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                ')' => {
                    tokens.push(Token::paran_close(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                '=' => {
                    tokens.push(Token::assign(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                ':' => {
                    tokens.push(Token::colon(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                ';' => {
                    let start_col = col;
                    i += 1;
                    let mut comment = String::new();
                    while i < len && chars[i] != '\n' {
                        comment.push(chars[i]);
                        i += 1;
                    }
                    col = start_col + 1 + comment.len();
                    tokens.push(Token::comment(&comment, line_no + 1, start_col + 1));
                }
                '\n' => {
                    tokens.push(Token {
                        token_type: TokenType::Newline,
                        line_no: line_no + 1,
                        start: col + 1,
                        end: col + 1,
                    });
                    i += 1;
                    col = 0;
                    line_no += 1;
                }
                c if c.is_ascii_digit() => {
                    let start_col = col;
                    let mut number = String::new();
                    let mut has_dot = false;

                    while i < len && (chars[i].is_ascii_digit() || (chars[i] == '.' && !has_dot)) {
                        if chars[i] == '.' {
                            has_dot = true;
                        }
                        number.push(chars[i]);
                        i += 1;
                        col += 1;
                    }

                    if i < len && (chars[i] == 'e' || chars[i] == 'E') {
                        number.push(chars[i]);
                        i += 1;
                        col += 1;
                        if i < len && (chars[i] == '+' || chars[i] == '-') {
                            number.push(chars[i]);
                            i += 1;
                            col += 1;
                        }
                        while i < len && chars[i].is_ascii_digit() {
                            number.push(chars[i]);
                            i += 1;
                            col += 1;
                        }
                    }

                    tokens.push(Token::number(&number, line_no + 1, start_col + 1));
                }
                c if c.is_whitespace() => {
                    i += 1;
                    col += 1;
                }

                c if c.is_alphabetic() || c == '_' => {
                    let start_col = col;
                    let mut ident = String::new();

                    while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        ident.push(chars[i]);
                        i += 1;
                        col += 1;
                    }

                    match ident.as_str() {
                        "let" => tokens.push(Token::let_token(line_no + 1, start_col + 1, col)),
                        _ => {
                            tokens.push(Token::identifier(&ident, line_no + 1, start_col + 1, col))
                        }
                    }
                }

                _ => {
                    return Err(TokenizerError::UnexpectedCharacter {
                        found: c,
                        line: line_no + 1,
                        col: col + 1,
                    });
                }
            }
        }

        tokens.push(Token::eof(line_no + 1, col + 1));
        self.tokens = tokens.clone();
        Ok(tokens)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::TokenizerError;

    fn assert_tokenize_ok(input: &str, expected: Vec<Token>) {
        match Tokenizer::new(input.to_string()).tokenize() {
            Ok(tokens) => assert_eq!(tokens, expected),
            Err(e) => panic!("Tokenizing failed for input \"{}\": {}", input, e),
        }
    }

    fn assert_tokenize_err(input: &str, expected_err: TokenizerError) {
        match Tokenizer::new(input.to_string()).tokenize() {
            Ok(tokens) => panic!(
                "Tokenizing should have failed for input \"{}\", but got tokens: {:?}",
                input, tokens
            ),
            Err(e) => assert_eq!(e, expected_err),
        }
    }

    #[test]
    fn test_simple_expression() {
        assert_tokenize_ok(
            "1+1",
            vec![
                Token::number("1", 1, 1),
                Token::plus(1, 2),
                Token::number("1", 1, 3),
                Token::eof(1, 4),
            ],
        );
    }

    #[test]
    fn test_parentheses() {
        assert_tokenize_ok(
            "(2*3)",
            vec![
                Token::paran_open(1, 1),
                Token::number("2", 1, 2),
                Token::mul(1, 3),
                Token::number("3", 1, 4),
                Token::paran_close(1, 5),
                Token::eof(1, 6),
            ],
        );
    }

    #[test]
    fn test_multi_digit_number() {
        assert_tokenize_ok(
            "123+456",
            vec![
                Token::number("123", 1, 1),
                Token::plus(1, 4),
                Token::number("456", 1, 5),
                Token::eof(1, 8),
            ],
        );
    }

    #[test]
    fn test_float_number() {
        assert_tokenize_ok(
            "3.14*2",
            vec![
                Token::number("3.14", 1, 1),
                Token::mul(1, 5),
                Token::number("2", 1, 6),
                Token::eof(1, 7),
            ],
        );
    }

    #[test]
    fn test_comment() {
        assert_tokenize_ok(
            "1+2;this is a comment",
            vec![
                Token::number("1", 1, 1),
                Token::plus(1, 2),
                Token::number("2", 1, 3),
                Token::comment("this is a comment", 1, 4),
                Token::eof(1, 22),
            ],
        );
    }

    #[test]
    fn test_newline_and_whitespace() {
        assert_tokenize_ok(
            "1 + 2\n3",
            vec![
                Token::number("1", 1, 1),
                Token::plus(1, 3),
                Token::number("2", 1, 5),
                Token {
                    token_type: TokenType::Newline,
                    line_no: 1,
                    start: 6,
                    end: 6,
                },
                Token::number("3", 2, 1),
                Token::eof(2, 2),
            ],
        );
    }

    #[test]
    fn test_empty_input() {
        assert_tokenize_ok("", vec![Token::eof(1, 1)]);
    }

    #[test]
    fn test_trailing_operator() {
        assert_tokenize_ok(
            "1+",
            vec![
                Token::number("1", 1, 1),
                Token::plus(1, 2),
                Token::eof(1, 3),
            ],
        );
    }

    #[test]
    fn test_numbers_stuck_together() {
        assert_tokenize_ok(
            "123456",
            vec![Token::number("123456", 1, 1), Token::eof(1, 7)],
        );
    }

    #[test]
    fn test_invalid_float_number() {
        assert_tokenize_err(
            "3.14.15",
            TokenizerError::UnexpectedCharacter {
                found: '.',
                line: 1,
                col: 5,
            },
        );
    }

    #[test]
    fn test_scientific_notation() {
        assert_tokenize_ok("1e-5", vec![Token::number("1e-5", 1, 1), Token::eof(1, 5)]);
    }

    #[test]
    fn test_whitespace_only_input() {
        assert_tokenize_ok("   ", vec![Token::eof(1, 4)]);
    }

    #[test]
    fn test_variable_declaration_untyped() {
        assert_tokenize_ok(
            "let x = 42",
            vec![
                Token::let_token(1, 1, 3),
                Token::identifier("x", 1, 5, 5),
                Token::assign(1, 7),
                Token::number("42", 1, 9),
                Token::eof(1, 11),
            ],
        );
    }

    #[test]
    fn test_variable_declaration_typed() {
        assert_tokenize_ok(
            "let y: Int = 3.14",
            vec![
                Token::let_token(1, 1, 3),
                Token::identifier("y", 1, 5, 5),
                Token::colon(1, 6),
                Token::identifier("Int", 1, 8, 10),
                Token::assign(1, 12),
                Token::number("3.14", 1, 14),
                Token::eof(1, 18),
            ],
        );
    }

    #[test]
    fn test_variable_reference() {
        assert_tokenize_ok("x", vec![Token::identifier("x", 1, 1, 1), Token::eof(1, 2)]);
    }

    #[test]
    fn test_variable_reassignment() {
        assert_tokenize_ok(
            "x = 99",
            vec![
                Token::identifier("x", 1, 1, 1),
                Token::assign(1, 3),
                Token::number("99", 1, 5),
                Token::eof(1, 7),
            ],
        );
    }

    #[test]
    fn test_multiple_variables() {
        assert_tokenize_ok(
            "let a = 1\nlet b: Float = 2.5\na = 3\nb",
            vec![
                Token::let_token(1, 1, 3),
                Token::identifier("a", 1, 5, 5),
                Token::assign(1, 7),
                Token::number("1", 1, 9),
                Token {
                    token_type: TokenType::Newline,
                    line_no: 1,
                    start: 10,
                    end: 10,
                },
                Token::let_token(2, 1, 3),
                Token::identifier("b", 2, 5, 5),
                Token::colon(2, 6),
                Token::identifier("Float", 2, 8, 12),
                Token::assign(2, 14),
                Token::number("2.5", 2, 16),
                Token {
                    token_type: TokenType::Newline,
                    line_no: 2,
                    start: 19,
                    end: 19,
                },
                Token::identifier("a", 3, 1, 1),
                Token::assign(3, 3),
                Token::number("3", 3, 5),
                Token {
                    token_type: TokenType::Newline,
                    line_no: 3,
                    start: 6,
                    end: 6,
                },
                Token::identifier("b", 4, 1, 1),
                Token::eof(4, 2),
            ],
        );
    }
}
