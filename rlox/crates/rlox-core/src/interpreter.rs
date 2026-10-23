use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use crate::token::{Token, Literal, TokenType};
use crate::ast::{Expr, Stmt};

#[derive(Debug, thiserror::Error)]
#[error("[line {token_line}] {message}", token_line = .token.line)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Nil => Value::Nil,
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub type SharedEnvironment = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<SharedEnvironment>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new(enclosing: Option<SharedEnvironment>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError {
            token: name.clone(),
            message: format!("Undefined variable '{}'.", name.lexeme),
        })
    }
}

pub struct Interpreter {
    environment: SharedEnvironment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new(None))),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }

            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{value}");
                Ok(())
            }

            Stmt::Var {name, init} => {
                let value = match init {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };

                self.environment.borrow_mut().define(name.lexeme.clone(), value);
                Ok(())
            }

            Stmt::Block { statements } => {
                let environment = Rc::new(RefCell::new(
                    Environment::new(Some(Rc::clone(&self.environment))),
                ));

                self.execute_block(statements, environment)
            }
        }
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(literal) => {
                Ok(Value::from(literal.clone()))
            }

            Expr::Grouping(expression) => {
                self.evaluate(expression)
            }

            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match &right {
                        Value::Number(n) => Ok(Value::Number(-*n)),
                        _ => Err(RuntimeError {
                            token: operator.clone(),
                            message: "Operand must be a number".to_string(),
                        }),
                    },
                    TokenType::Bang => Ok(Value::Bool(!right.is_truthy())),
                    _ => unreachable!(),
                }
            }


            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus
                    | TokenType::Slash
                    | TokenType::Star
                    | TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual => {
                        let (Value::Number(left_num), Value::Number(right_num)) =
                            (&left, &right)
                        else {
                            return Err(RuntimeError {
                                token: operator.clone(),
                                message: "Operands must be numbers.".to_string(),
                            });
                        };

                        match operator.token_type {
                            TokenType::Minus => {
                                Ok(Value::Number(left_num - right_num))
                            }
                            TokenType::Slash => {
                                Ok(Value::Number(left_num / right_num))
                            }
                            TokenType::Star => {
                                Ok(Value::Number(left_num * right_num))
                            }
                            TokenType::Greater => {
                                Ok(Value::Bool(left_num > right_num))
                            }
                            TokenType::GreaterEqual => {
                                Ok(Value::Bool(left_num >= right_num))
                            }
                            TokenType::Less => {
                                Ok(Value::Bool(left_num < right_num))
                            }
                            TokenType::LessEqual => {
                                Ok(Value::Bool(left_num <= right_num))
                            }
                            _ => unreachable!(),
                        }
                    }

                    TokenType::Plus => match (&left, &right) {
                        (Value::Number(a), Value::Number(b)) => {
                            Ok(Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => {
                            Ok(Value::String(format!("{a}{b}")))
                        }
                        _ => Err(RuntimeError {
                            token: operator.clone(),
                            message:
                            "Operands must be two numbers or two strings.".to_string(),
                        }),
                    },

                    TokenType::EqualEqual => {
                        Ok(Value::Bool(left == right))
                    }

                    TokenType::BangEqual => {
                        Ok(Value::Bool(left != right))
                    }

                    _ => unreachable!(),
                }
            }

            Expr::Variable { name } => {
                self.environment.borrow().get(name)
            }

            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
        }
    }

    pub fn execute_block(&mut self, statements: &[Stmt], environment: SharedEnvironment) -> Result<(), RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, environment);

        let result = (|| {
            for statement in statements {
                self.execute(statement)?;
            }
            Ok(())
        })();

        self.environment = previous;
        result
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::token::{Literal, Token, TokenType};

    fn token(token_type: TokenType, lexeme: &str) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            literal: None,
            line: 1,
        }
    }

    fn literal(value: Literal) -> Expr {
        Expr::Literal(value)
    }

    fn unary(operator: TokenType, lexeme: &str, right: Expr) -> Expr {
        Expr::Unary {
            operator: token(operator, lexeme),
            right: Box::new(right),
        }
    }

    fn binary(
        left: Expr,
        operator: TokenType,
        lexeme: &str,
        right: Expr,
    ) -> Expr {
        Expr::Binary {
            left: Box::new(left),
            operator: token(operator, lexeme),
            right: Box::new(right),
        }
    }

    fn evaluate(expr: &Expr) -> Result<Value, RuntimeError> {
        let mut interpreter = Interpreter::new();
        interpreter.evaluate(expr)
    }

    #[test]
    fn evaluates_literals() {
        assert_eq!(
            evaluate(&literal(Literal::Number(42.0))).unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            evaluate(&literal(Literal::String("hello".into()))).unwrap(),
            Value::String("hello".into())
        );
        assert_eq!(
            evaluate(&literal(Literal::Bool(true))).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            evaluate(&literal(Literal::Nil)).unwrap(),
            Value::Nil
        );
    }

    #[test]
    fn evaluates_grouping() {
        let expr = Expr::Grouping(Box::new(literal(Literal::Number(42.0))));

        assert_eq!(evaluate(&expr).unwrap(), Value::Number(42.0));
    }

    #[test]
    fn evaluates_unary_minus() {
        let expr = unary(
            TokenType::Minus,
            "-",
            literal(Literal::Number(5.0)),
        );

        assert_eq!(evaluate(&expr).unwrap(), Value::Number(-5.0));
    }

    #[test]
    fn unary_minus_rejects_non_numbers() {
        let expr = unary(
            TokenType::Minus,
            "-",
            literal(Literal::String("hello".into())),
        );

        let error = evaluate(&expr).unwrap_err();

        assert_eq!(error.message, "Operand must be a number");
        assert_eq!(error.token.lexeme, "-");
    }

    #[test]
    fn evaluates_logical_not_using_truthiness() {
        let cases = [
            (Literal::Bool(true), false),
            (Literal::Bool(false), true),
            (Literal::Nil, true),
            (Literal::Number(0.0), false),
            (Literal::String(String::new()), false),
        ];

        for (literal_value, expected) in cases {
            let expr = unary(
                TokenType::Bang,
                "!",
                literal(literal_value),
            );

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(expected)
            );
        }
    }

    #[test]
    fn evaluates_arithmetic() {
        let cases = [
            (TokenType::Plus, "+", 2.0, 3.0, 5.0),
            (TokenType::Minus, "-", 7.0, 3.0, 4.0),
            (TokenType::Star, "*", 4.0, 3.0, 12.0),
            (TokenType::Slash, "/", 8.0, 2.0, 4.0),
        ];

        for (operator, lexeme, left, right, expected) in cases {
            let expr = binary(
                literal(Literal::Number(left)),
                operator,
                lexeme,
                literal(Literal::Number(right)),
            );

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Number(expected),
                "failed for {left} {lexeme} {right}"
            );
        }
    }

    #[test]
    fn evaluates_comparisons() {
        let cases = [
            (TokenType::Greater, ">", 3.0, 2.0, true),
            (TokenType::GreaterEqual, ">=", 3.0, 3.0, true),
            (TokenType::Less, "<", 2.0, 3.0, true),
            (TokenType::LessEqual, "<=", 3.0, 3.0, true),
        ];

        for (operator, lexeme, left, right, expected) in cases {
            let expr = binary(
                literal(Literal::Number(left)),
                operator,
                lexeme,
                literal(Literal::Number(right)),
            );

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(expected)
            );
        }
    }

    #[test]
    fn arithmetic_operators_reject_non_numbers() {
        let expr = binary(
            literal(Literal::String("hello".into())),
            TokenType::Minus,
            "-",
            literal(Literal::Number(1.0)),
        );

        assert_eq!(
            evaluate(&expr).unwrap_err().message,
            "Operands must be numbers."
        );
    }

    #[test]
    fn plus_adds_numbers_and_concatenates_strings() {
        let number_expr = binary(
            literal(Literal::Number(2.0)),
            TokenType::Plus,
            "+",
            literal(Literal::Number(3.0)),
        );

        assert_eq!(
            evaluate(&number_expr).unwrap(),
            Value::Number(5.0)
        );

        let string_expr = binary(
            literal(Literal::String("hello ".into())),
            TokenType::Plus,
            "+",
            literal(Literal::String("world".into())),
        );

        assert_eq!(
            evaluate(&string_expr).unwrap(),
            Value::String("hello world".into())
        );
    }

    #[test]
    fn plus_rejects_mixed_operand_types() {
        let expr = binary(
            literal(Literal::String("hello".into())),
            TokenType::Plus,
            "+",
            literal(Literal::Number(1.0)),
        );

        assert_eq!(
            evaluate(&expr).unwrap_err().message,
            "Operands must be two numbers or two strings."
        );
    }

    #[test]
    fn evaluates_equality() {
        let cases = [
            (Literal::Number(2.0), Literal::Number(2.0), true),
            (Literal::Number(2.0), Literal::Number(3.0), false),
            (Literal::String("a".into()), Literal::String("a".into()), true),
            (Literal::Bool(true), Literal::Bool(false), false),
            (Literal::Nil, Literal::Nil, true),
        ];

        for (left, right, expected) in cases {
            let equal_expr = binary(
                literal(left.clone()),
                TokenType::EqualEqual,
                "==",
                literal(right.clone()),
            );

            assert_eq!(
                evaluate(&equal_expr).unwrap(),
                Value::Bool(expected)
            );

            let not_equal_expr = binary(
                literal(left),
                TokenType::BangEqual,
                "!=",
                literal(right),
            );

            assert_eq!(
                evaluate(&not_equal_expr).unwrap(),
                Value::Bool(!expected)
            );
        }
    }

    #[test]
    fn evaluates_nested_expressions() {
        let expr = binary(
            Expr::Grouping(Box::new(binary(
                literal(Literal::Number(1.0)),
                TokenType::Plus,
                "+",
                literal(Literal::Number(2.0)),
            ))),
            TokenType::Star,
            "*",
            literal(Literal::Number(3.0)),
        );

        assert_eq!(evaluate(&expr).unwrap(), Value::Number(9.0));
    }
}