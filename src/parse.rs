use anyhow::anyhow;

use crate::{
    expression::{BinaryExpr, Expr, GroupingExpr, Literal, Operator, Unary, UnaryExpr},
    token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    pub fn parse(mut self) -> anyhow::Result<Expr> {
        self.expression()
    }

    fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.position).map(|t| &t.ty)
    }

    fn expression(&mut self) -> anyhow::Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.comparison()?;

        while let Some(operator) = match self.peek() {
            Some(TokenType::BangEqual) => Some(Operator::NotEqual),
            Some(TokenType::EqualEqual) => Some(Operator::Equal),
            _ => None,
        } {
            self.position += 1;
            let right = self.comparison()?;
            left = Expr::Binary(BinaryExpr {
                left: left.into(),
                operator,
                right: right.into(),
            });
        }
        Ok(left)
    }

    fn comparison(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.term()?;

        while let Some(operator) = match self.peek() {
            Some(TokenType::Greater) => Some(Operator::Greater),
            Some(TokenType::GreaterEqual) => Some(Operator::GreaterEqual),
            Some(TokenType::Less) => Some(Operator::Less),
            Some(TokenType::LessEqual) => Some(Operator::LessEqual),
            _ => None,
        } {
            self.position += 1;
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {
                left: expr.into(),
                operator,
                right: right.into(),
            });
        }
        Ok(expr)
    }

    fn term(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.factor()?;

        while let Some(operator) = match self.peek() {
            Some(TokenType::Minus) => Some(Operator::Minus),
            Some(TokenType::Plus) => Some(Operator::Plus),
            _ => None,
        } {
            self.position += 1;
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr {
                left: expr.into(),
                operator,
                right: right.into(),
            });
        }
        Ok(expr)
    }

    fn factor(&mut self) -> anyhow::Result<Expr> {
        let mut expr = self.unary()?;

        while let Some(operator) = match self.peek() {
            Some(TokenType::Slash) => Some(Operator::Divide),
            Some(TokenType::Star) => Some(Operator::Multiply),
            _ => None,
        } {
            self.position += 1;
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr {
                left: expr.into(),
                operator,
                right: right.into(),
            });
        }
        Ok(expr)
    }

    fn unary(&mut self) -> anyhow::Result<Expr> {
        if let Some(operator) = match self.peek() {
            Some(TokenType::Bang) => Some(Unary::Bang),
            Some(TokenType::Minus) => Some(Unary::Minus),
            _ => None,
        } {
            self.position += 1;
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                expr: right.into(),
                unary: operator,
            }));
        }

        self.primary()
    }

    fn primary(&mut self) -> anyhow::Result<Expr> {
        let primary = match self.peek() {
            Some(TokenType::False) => Some(Expr::Literal(Literal::Bool(false))),
            Some(TokenType::True) => Some(Expr::Literal(Literal::Bool(true))),
            Some(TokenType::Nil) => Some(Expr::Literal(Literal::Nil)),
            Some(TokenType::Number(value)) => Some(Expr::Literal(Literal::Number(*value))),
            Some(TokenType::String(value)) => Some(Expr::Literal(Literal::String(value.clone()))),
            Some(TokenType::LeftParen) => {
                self.position += 1;
                let expr = self.expression()?;
                anyhow::ensure!(self.peek() == Some(&TokenType::RightParen), "");
                self.position += 1;
                return Ok(Expr::Grouping(GroupingExpr { expr: expr.into() }));
            }
            _ => None,
        };

        if let Some(primary) = primary {
            self.position += 1;
            return Ok(primary);
        }

        Err(anyhow!("expected expression"))
    }
}
