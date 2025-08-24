use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    Newline,

    // 4 basic operations
    Plus,
    Minus,
    Div, // division
    Mul, // multiplication

    ParanOpen,
    ParanClose,

    // ;
    Comment { contents: String },
    Number { value: String },
    EOF,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    line_no: usize,
    start: usize,
    end: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line_no: usize, start: usize, end: usize) -> Token {
        Token {
            token_type,
            line_no,
            start,
            end,
        }
    }

    // getters
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
            self.token_type,
            self.line_no,
            self.start,
            self.end
        )
    }
}

pub struct Tokenizer {
    content: String,
    tokens: Vec<Token>,
}
fn make_error_msg(src: &str, line: usize, col: usize, found: char) -> String {
    let line_str = src.lines().nth(line).unwrap_or("");
    format!(
        "Unexpected character '{}' at line {}, col {}
{}
{:>width$}^
",
        found,
        line + 1,
        col + 1,
        line_str,
        "",
        width = col + 1
    )
}
impl Debug for Tokenizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tokenizer {{")?;
        write!(f, "content:\n{}\n", self.content)?;
        write!(f, "tokens:\n")?;
        for token in &self.tokens {
            write!(f, "\t{}\n", token)?;
        }
        write!(f, "}}")
    }
}
impl Tokenizer {
    pub fn new(content: String) -> Tokenizer {
        Tokenizer {
            content,
            tokens: Vec::new(),
        }
    }
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
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
                    tokens.push(Token::plus(line_no, col));
                    i += 1;
                    col += 1;
                }
                '-' => {
                    tokens.push(Token::minus(line_no, col));
                    i += 1;
                    col += 1;
                }
                '/' => {
                    tokens.push(Token::div(line_no, col));
                    i += 1;
                    col += 1;
                }
                '*' => {
                    tokens.push(Token::mul(line_no, col));
                    i += 1;
                    col += 1;
                }
                '(' => {
                    tokens.push(Token::paran_open(line_no, col));
                    i += 1;
                    col += 1;
                }
                ')' => {
                    tokens.push(Token::paran_close(line_no, col));
                    i += 1;
                    col += 1;
                }
                ';' => {
                    // Read until newline
                    let _start = i;
                    let mut comment = String::new();
                    i += 1;
                    col += 1;

                    while i < len && chars[i] != '\n' {
                        comment.push(chars[i]);
                        i += 1;
                        col += 1;
                    }

                    tokens.push(Token::comment(&comment, line_no, col - comment.len()));
                }

                '\n' => {
                    tokens.push(Token {
                        token_type: TokenType::Newline,
                        line_no,
                        start: col,
                        end: col,
                    });
                    i += 1;
                    col = 0;
                    line_no += 1;
                }
                c if c.is_ascii_digit() => {
                    // Parse number (int or float)
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

                    // now check for scientific notation
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


                    tokens.push(Token::number(&number, line_no, start_col));
                }
                c if c.is_whitespace() => {
                    // Just skip whitespace (other than newline handled above)
                    i += 1;
                    col += 1;
                }
                _ => {
                    return Err(make_error_msg(&self.content, line_no, col, chars[i]));
                }
            }
        }
        tokens.push(Token::eof(line_no, col));
        self.tokens = tokens.clone();
        Ok(tokens)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tokenize_ok(input: &str, expected: Vec<Token>) {
        match Tokenizer::new(input.to_string()).tokenize() {
            Ok(tokens) => assert_eq!(tokens, expected),
            Err(e) => panic!("Tokenizing failed for input \"{}\": {}", input, e),
        }
    }

    fn assert_tokenize_err(input: &str, expected_err: &str) {
        match Tokenizer::new(input.to_string()).tokenize() {
            Ok(tokens) => panic!("Tokenizing should have failed for input \"{}\", but got tokens: {:?}", input, tokens),
            Err(e) => assert!(e.contains(expected_err)),
        }
    }

    #[test]
    fn test_simple_expression() {
        assert_tokenize_ok(
            "1+1",
            vec![
                Token::number("1", 0, 0),
                Token::plus(0, 1),
                Token::number("1", 0, 2),
                Token::eof(0, 3),
            ],
        );
    }

    #[test]
    fn test_parentheses() {
        assert_tokenize_ok(
            "(2*3)",
            vec![
                Token::paran_open(0, 0),
                Token::number("2", 0, 1),
                Token::mul(0, 2),
                Token::number("3", 0, 3),
                Token::paran_close(0, 4),
                Token::eof(0, 5),
            ],
        );
    }

    #[test]
    fn test_multi_digit_number() {
        assert_tokenize_ok(
            "123+456",
            vec![
                Token::number("123", 0, 0),
                Token::plus(0, 3),
                Token::number("456", 0, 4),
                Token::eof(0, 7),
            ],
        );
    }

    #[test]
    fn test_float_number() {
        assert_tokenize_ok(
            "3.14*2",
            vec![
                Token::number("3.14", 0, 0),
                Token::mul(0, 4),
                Token::number("2", 0, 5),
                Token::eof(0, 6),
            ],
        );
    }

    #[test]
    fn test_comment() {
        assert_tokenize_ok(
            "1+2;this is a comment",
            vec![
                Token::number("1", 0, 0),
                Token::plus(0, 1),
                Token::number("2", 0, 2),
                Token::comment("this is a comment", 0, 4),
                Token::eof(0, 21),
            ],
        );
    }

    #[test]
    fn test_newline_and_whitespace() {
        assert_tokenize_ok(
            "1 + 2\n3",
            vec![
                Token::number("1", 0, 0),
                Token::plus(0, 2),
                Token::number("2", 0, 4),
                Token {
                    token_type: TokenType::Newline,
                    line_no: 0,
                    start: 5,
                    end: 5,
                },
                Token::number("3", 1, 0),
                Token::eof(1, 1),
            ],
        );
    }

    #[test]
    fn test_empty_input() {
        assert_tokenize_ok("", vec![Token::eof(0, 0)]);
    }

    #[test]
    fn test_trailing_operator() {
        assert_tokenize_ok(
            "1+",
            vec![
                Token::number("1", 0, 0),
                Token::plus(0, 1),
                Token::eof(0, 2),
            ],
        );
    }

    #[test]
    fn test_numbers_stuck_together() {
        assert_tokenize_ok("123456", vec![Token::number("123456", 0, 0), Token::eof(0, 6)]);
    }

    #[test]
    fn test_invalid_float_number() {
        assert_tokenize_err("3.14.15", "Unexpected character");
    }

    #[test]
    fn test_scientific_notation() {
        assert_tokenize_ok(
            "1e-5",
            vec![Token::number("1e-5", 0, 0), Token::eof(0, 4)],
        );
    }
}

