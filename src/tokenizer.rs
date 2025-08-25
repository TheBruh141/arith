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
    pub fn eof(line_no: usize, pos: usize) -> Token {
        Token::new(TokenType::EOF, line_no, pos, pos)
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Newline => write!(f, "Newline"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Div => write!(f, "Div"),
            TokenType::Mul => write!(f, "Mul"),
            TokenType::ParanOpen => write!(f, "ParanOpen"),
            TokenType::ParanClose => write!(f, "ParanClose"),
            TokenType::Comment { contents } => write!(f, "Comment: {}", contents),
            TokenType::Number { value } => write!(f, "Number({})", value),
            TokenType::EOF => write!(f, "eof"),
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
                    tokens.push(Token::plus(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                '-' => {
                    tokens.push(Token::minus(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                '/' => {
                    tokens.push(Token::div(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
                }
                '*' => {
                    tokens.push(Token::mul(line_no + 1, col + 1));
                    i += 1;
                    col += 1;
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
                ';' => {
                    // Comments run to the end of the line.
                    let start_col = col;
                    i += 1; // consume ';'

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
                    // Parse a number, which can be an integer, a float, or in
                    // scientific notation.
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

                    // Handle scientific notation (e.g., 1e-5, 2.5E+3).
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
                    // Ignore whitespace characters (other than newlines).
                    i += 1;
                    col += 1;
                }
                _ => {
                    log::debug!("DEBUG: Tokenizer line_no = {}, col = {}", line_no, col);
                    return Err(TokenizerError::UnexpectedCharacter {
                        found: chars[i],
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
}

// TODO: merge with git, fix 1 index. brand
