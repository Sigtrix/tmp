use std::fmt;

use crate::token::{Literal, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Grouping(Box<Expr>),

    Literal(Literal),

    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }

            Self::Grouping(expression) => {
                write!(f, "(group {})", expression)
            }

            Self::Literal(literal) => {
                write!(f, "{literal}")
            }

            Self::Unary { operator, right } => {
                write!(f, "({} {})", operator.lexeme, right)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{TokenType};

    #[test]
    fn test_display() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(1.0))),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                literal: None,
                line: 0,
            },
            right: Box::new(Expr::Literal(Literal::Number(2.0))),
        };

        assert_eq!(expr.to_string(), "(+ 1 2)");

        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 0,
                },
                right: Box::new(Expr::Literal(Literal::Number(123.0))),
            }),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
                line: 0,
            },
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(45.67))))),
        };

        assert_eq!(expr.to_string(), "(* (- 123) (group 45.67))")
    }
}