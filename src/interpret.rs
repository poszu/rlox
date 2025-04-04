use anyhow::{anyhow, Result};

use crate::expression::{BinaryExpr, Expr, GroupingExpr, Literal, Operator, Unary, UnaryExpr};

#[derive(Debug, Clone, PartialEq, PartialOrd, parse_display::Display)]
#[display(style = "lowercase")]
pub enum Value {
    #[display("nil")]
    Nil,
    #[display("{0}")]
    Bool(bool),
    #[display("{0}")]
    Number(f64),
    #[display("{0}")]
    String(String),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(v) => *v,
            Value::Number(_) => true,
            Value::String(_) => true,
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, other: &bool) -> bool {
        match self {
            Value::Bool(b) => b == other,
            _ => false,
        }
    }
}

impl PartialEq<Value> for bool {
    fn eq(&self, other: &Value) -> bool {
        match other {
            Value::Bool(b) => b == self,
            _ => false,
        }
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        match self {
            Value::String(s) => s == other,
            _ => false,
        }
    }
}

impl PartialEq<Value> for String {
    fn eq(&self, other: &Value) -> bool {
        match other {
            Value::String(s) => s == self,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Value::String(s) => s == *other,
            _ => false,
        }
    }
}

impl PartialEq<Value> for &str {
    fn eq(&self, other: &Value) -> bool {
        match other {
            Value::String(s) => s == *self,
            _ => false,
        }
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match self {
            Value::Number(n) => n == other,
            _ => false,
        }
    }
}

impl PartialEq<Value> for f64 {
    fn eq(&self, other: &Value) -> bool {
        match other {
            Value::Number(n) => n == self,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct Interpreter {}

impl Interpreter {
    pub fn evaluate(&mut self, expr: Expr) -> Result<Value> {
        match expr {
            Expr::Grouping(grouping) => self.eval_grouping(grouping),
            Expr::Unary(unary) => self.eval_unary(unary),
            Expr::Binary(binary) => self.eval_binary(binary),
            Expr::Literal(literal) => match literal {
                Literal::Nil => Ok(Value::Nil),
                Literal::Bool(v) => Ok(Value::Bool(v)),
                Literal::Number(v) => Ok(Value::Number(v)),
                Literal::String(v) => Ok(Value::String(v)),
            },
        }
    }

    fn eval_grouping(&mut self, expr: GroupingExpr) -> Result<Value> {
        self.evaluate(*expr.expr)
    }

    fn eval_unary(&mut self, expr: UnaryExpr) -> Result<Value> {
        let value = self.evaluate(*expr.expr)?;
        match (expr.unary, value) {
            (Unary::Bang, v) => Ok((!v.is_truthy()).into()),
            (Unary::Minus, Value::Number(v)) => Ok((-v).into()),

            (Unary::Minus, Value::Nil) => Err(anyhow!("can't '- nil'")),
            (Unary::Minus, Value::Bool(_)) => Err(anyhow!("can't '- bool'")),
            (Unary::Minus, Value::String(_)) => Err(anyhow!("can't '- string'")),
        }
    }

    fn eval_binary(&mut self, expr: BinaryExpr) -> Result<Value> {
        let left = self.evaluate(*expr.left)?;
        let right = self.evaluate(*expr.right)?;
        match expr.operator {
            Operator::Greater => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a > b).into()),
                _ => Err(anyhow!("can > only numbers")),
            },
            Operator::GreaterEqual => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a >= b).into()),
                _ => Err(anyhow!("can >= only numbers")),
            },
            Operator::Less => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a < b).into()),
                _ => Err(anyhow!("can < only numbers")),
            },
            Operator::LessEqual => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a != b).into()),
                _ => Err(anyhow!("can <= only numbers")),
            },
            Operator::Equal => match (left, right) {
                (Value::Bool(v), right) => Ok((v == right.is_truthy()).into()),
                (left, Value::Bool(v)) => Ok((v == left.is_truthy()).into()),
                (left, right) => Ok((left == right).into()),
            },
            Operator::NotEqual => match (left, right) {
                (Value::Bool(v), right) => Ok((v != right.is_truthy()).into()),
                (left, Value::Bool(v)) => Ok((v != left.is_truthy()).into()),
                (left, right) => Ok((left != right).into()),
            },
            Operator::Minus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a - b).into()),
                _ => Err(anyhow!("can only subtract numbers")),
            },
            Operator::Plus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a + b).into()),
                (Value::String(mut a), Value::String(b)) => {
                    a.push_str(&b);
                    Ok(a.into())
                }
                _ => Err(anyhow!(
                    "can only add numbers or strings (for concatenation)"
                )),
            },
            Operator::Divide => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a / b).into()),
                _ => Err(anyhow!("can only divide numbers")),
            },
            Operator::Multiply => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok((a * b).into()),
                _ => Err(anyhow!("can only multiply numbers")),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse::Parser, scanner::scan_tokens};

    use super::{Interpreter, Value};

    fn eval(line: &str) -> anyhow::Result<Value> {
        let tokens = scan_tokens(line).unwrap();
        let expr = Parser::new(tokens).parse().unwrap();
        Interpreter::default().evaluate(expr)
    }
    #[test]
    fn addition() {
        assert_eq!(3.0, eval("1+2").unwrap());
        assert_eq!("foobar", eval(r#""foo" + "bar""#).unwrap());
    }

    #[test]
    fn subtraction() {
        assert_eq!(-1.0, eval("1-2").unwrap());
        assert!(eval(r#""foo" - "bar""#).is_err());
    }

    #[test]
    fn grouping() {
        assert_eq!(4.0, eval("5 - (2 - 1)").unwrap());
        assert_eq!(2.0, eval("(5 - 2) - 1").unwrap());
    }

    #[test]
    fn comapre_nans() {
        assert_eq!(false, eval("(0 / 0) == (0 / 0)").unwrap());
    }

    #[test]
    fn truthness() {
        assert_eq!(false, eval("nil == true").unwrap());

        assert_eq!(false, eval("false").unwrap());
        assert_eq!(true, eval("true").unwrap());

        assert_eq!(true, eval("1 == true").unwrap());
        assert_eq!(true, eval("1 == true").unwrap());
        assert_eq!(true, eval("0 == true").unwrap());

        assert_eq!(true, eval(r#""" == true"#).unwrap());
        assert_eq!(true, eval(r#""foobar" == true"#).unwrap());
    }

    #[test]
    fn unary() {
        assert_eq!(false, eval("!true").unwrap());
        assert_eq!(true, eval("!false").unwrap());
        assert_eq!(-5.0, eval("-5").unwrap());
        assert_eq!(5.0, eval("----5").unwrap());
        assert_eq!(false, eval(r#"!"string""#).unwrap());
    }
}
