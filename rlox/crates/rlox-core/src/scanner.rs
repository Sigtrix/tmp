use crate::token::{Token, TokenType, Literal};

#[derive(Debug)]
pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("[Line {line}] Unexpected character '{character}'")]
    UnexpectedCharacter {
        character: char,
        line: usize,
    },

    #[error("[Line {line}] Unterminated string")]
    UnterminatedString {
        line: usize,
    },
}

pub struct Scanner<'source> {
    source: &'source str,
    start: usize,
    current: usize,
    line: usize,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) -> ScanResult {
        self.scan_tokens()
    }

    pub fn scan_tokens(&mut self) -> ScanResult {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;

            if let Err(error) = self.scan_token(&mut tokens) {
                errors.push(error);
            }
        }

        tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            literal: None,
            line: self.line,
        });

        ScanResult { tokens, errors }
    }

    fn scan_token(&mut self, tokens: &mut Vec<Token>) -> Result<(), ScanError> {
        let c = self.advance();

        match c {
            '(' => self.add_token(tokens, TokenType::LeftParen, None),
            ')' => self.add_token(tokens, TokenType::RightParen, None),
            '{' => self.add_token(tokens, TokenType::LeftBrace, None),
            '}' => self.add_token(tokens, TokenType::RightBrace, None),
            ',' => self.add_token(tokens, TokenType::Comma, None),
            '.' => self.add_token(tokens, TokenType::Dot, None),
            '-' => self.add_token(tokens, TokenType::Minus, None),
            '+' => self.add_token(tokens, TokenType::Plus, None),
            ';' => self.add_token(tokens, TokenType::Semicolon, None),
            '*' => self.add_token(tokens, TokenType::Star, None),

            '!' => {
                let token_type = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };

                self.add_token(tokens, token_type, None);
            }

            '=' => {
                let token_type = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };

                self.add_token(tokens, token_type, None);
            }

            '<' => {
                let token_type = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };

                self.add_token(tokens, token_type, None);
            }

            '>' => {
                let token_type = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(tokens, token_type, None);
            }

            '/' => {
                if self.matches('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(tokens, TokenType::Slash, None);
                }
            }

            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
            }

            '\n' => {
                self.line += 1;
            }

            '"' => {
                self.string(tokens)?;
            }

            c if c.is_ascii_digit() => {
                self.number(tokens);
            }

            c if is_identifier_start(c) => {
                self.identifier(tokens);
            }

            c => {
                return Err(
                    ScanError::UnexpectedCharacter {
                        line: self.line,
                        character: c,
                })
            }
        }

        Ok(())
    }

    fn string(&mut self, tokens: &mut Vec<Token>) -> Result<(), ScanError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return Err(
                ScanError::UnterminatedString {
                    line: self.line,
                })
        }

        // Consume the closing quote.
        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(tokens, TokenType::String, Some(Literal::String(value.to_owned())));

        Ok(())
    }

    fn number(&mut self, tokens: &mut Vec<Token>) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a fractional part.
        // Only consume the '.' if it is followed by a digit.
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = self.lexeme().parse::<f64>().unwrap();
        self.add_token(tokens, TokenType::Number, Some(Literal::Number(value)));
    }

    fn identifier(&mut self, tokens: &mut Vec<Token>) {
        while is_identifier_continue(self.peek()) {
            self.advance();
        }

        let lexeme = self.lexeme();

        match lexeme {
            "var" => self.add_token(tokens, TokenType::Var, None),
            "nil" => self.add_token(tokens, TokenType::Nil, Some(Literal::Nil)),
            "true" => self.add_token(tokens, TokenType::True, Some(Literal::Bool(true))),
            "false" => self.add_token(tokens, TokenType::False, Some(Literal::Bool(false))),
            "while" => self.add_token(tokens, TokenType::While, None),
            "for" => self.add_token(tokens, TokenType::For, None),
            "if" => self.add_token(tokens, TokenType::If, None),
            "else" => self.add_token(tokens, TokenType::Else, None),
            "or" => self.add_token(tokens, TokenType::Or, None),
            "and" => self.add_token(tokens, TokenType::And, None),
            "fun" => self.add_token(tokens, TokenType::Fun, None),
            "return" => self.add_token(tokens, TokenType::Return, None),
            "class" => self.add_token(tokens, TokenType::Class, None),
            "super" => self.add_token(tokens, TokenType::Super, None),
            "this" => self.add_token(tokens, TokenType::This, None),
            "print" => self.add_token(tokens, TokenType::Print, None),
            _ => self.add_token(tokens, TokenType::Identifier, None),
        }
    }

    fn add_token(&self, tokens: &mut Vec<Token>, token_type: TokenType, literal: Option<Literal>) {
        tokens.push(Token {
            token_type,
            lexeme: self.lexeme().to_owned(),
            literal,
            line: self.line,
        });
    }

    fn advance(&mut self) -> char {
        let c = self.peek();

        if !c.is_ascii() {
            // Lox source is ASCII for now. Consume the UTF-8 character
            // so that we still make progress and report it as invalid.
            let len = c.len_utf8();
            self.current += len;
        } else {
            self.current += 1;
        }

        c
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.current += expected.len_utf8();
        true
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        let mut chars = self.source[self.current..].chars();

        chars.next();
        chars.next().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn lexeme(&self) -> &str {
        &self.source[self.start..self.current]
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_single_character_tokens() {
        let result = Scanner::new("(){},.-+;/*").scan();

        assert!(result.errors.is_empty());

        let types: Vec<_> = result
            .tokens
            .iter()
            .map(|token| token.token_type)
            .collect();

        assert_eq!(
            types,
            [
                TokenType::LeftParen,
                TokenType::RightParen,
                TokenType::LeftBrace,
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::Dot,
                TokenType::Minus,
                TokenType::Plus,
                TokenType::Semicolon,
                TokenType::Slash,
                TokenType::Star,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn scans_two_character_operators() {
        let result = Scanner::new("! != = == < <= > >=").scan();

        assert!(result.errors.is_empty());

        let types: Vec<_> = result
            .tokens
            .iter()
            .map(|token| token.token_type)
            .collect();

        assert_eq!(
            types,
            [
                TokenType::Bang,
                TokenType::BangEqual,
                TokenType::Equal,
                TokenType::EqualEqual,
                TokenType::Less,
                TokenType::LessEqual,
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn scans_strings() {
        let result = Scanner::new(r#""hello world""#).scan();

        assert!(result.errors.is_empty());
        assert_eq!(result.tokens[0].token_type, TokenType::String);
        assert_eq!(result.tokens[0].lexeme, r#""hello world""#);
    }

    #[test]
    fn scans_numbers() {
        let result = Scanner::new("123 123.456").scan();

        assert!(result.errors.is_empty());
        assert_eq!(result.tokens[0].token_type, TokenType::Number);
        assert_eq!(result.tokens[0].lexeme, "123");
        assert_eq!(result.tokens[1].token_type, TokenType::Number);
        assert_eq!(result.tokens[1].lexeme, "123.456");
    }

    #[test]
    fn scans_keywords_and_identifiers() {
        let result = Scanner::new("and class foo bar true false").scan();

        assert!(result.errors.is_empty());

        let types: Vec<_> = result
            .tokens
            .iter()
            .map(|token| token.token_type)
            .collect();

        assert_eq!(
            types,
            [
                TokenType::And,
                TokenType::Class,
                TokenType::Identifier,
                TokenType::Identifier,
                TokenType::True,
                TokenType::False,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn skips_comments() {
        let result = Scanner::new("foo // this is a comment\nbar").scan();

        assert!(result.errors.is_empty());
        assert_eq!(result.tokens[0].lexeme, "foo");
        assert_eq!(result.tokens[1].lexeme, "bar");
        assert_eq!(result.tokens[1].line, 2);
    }

    #[test]
    fn reports_multiple_errors() {
        let result = Scanner::new("@ # $").scan();

        assert_eq!(result.errors.len(), 3);
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].token_type, TokenType::Eof);
    }

    #[test]
    fn reports_unterminated_string() {
        let result = Scanner::new("\"hello").scan();

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].to_string(), "[Line 1] Unterminated string");
    }
}