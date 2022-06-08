use std::fmt::Write;

use crate::expression::{BinaryExpr, Expr, GroupingExpr, Literal, UnaryExpr};

pub trait AstPrint {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result;
    fn print_ast(&self) -> String {
        let mut output = String::new();
        self.write_to(&mut output)
            .expect("pretty printing returned an error");
        output
    }
}

impl AstPrint for Literal {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result {
        match self {
            Literal::Nil => f.write_str("nil"),
            Literal::Bool(v) => write!(f, "{}", v),
            Literal::Number(v) => write!(f, "{}", v),
            Literal::String(v) => write!(f, "{}", v),
        }
    }
}

impl AstPrint for GroupingExpr {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result {
        f.write_str("(group ")?;
        self.expr.write_to(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

impl AstPrint for UnaryExpr {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result {
        write!(f, "({} ", self.unary)?;
        self.expr.write_to(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

impl AstPrint for BinaryExpr {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result {
        write!(f, "({} ", self.operator)?;
        self.left.write_to(f)?;
        f.write_char(' ')?;
        self.right.write_to(f)?;
        f.write_char(')')?;
        Ok(())
    }
}

impl AstPrint for Expr {
    fn write_to(&self, f: &mut impl Write) -> std::fmt::Result {
        match self {
            Expr::Grouping(v) => v.write_to(f),
            Expr::Unary(v) => v.write_to(f),
            Expr::Binary(v) => v.write_to(f),
            Expr::Literal(v) => v.write_to(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::{BinaryExpr, Expr, GroupingExpr, Literal, Unary::Minus, UnaryExpr};

    use super::AstPrint;

    #[test]
    fn binary_plus() {
        let expr = Expr::Binary(BinaryExpr {
            left: Box::new(Expr::Literal(Literal::Number(1.0))),
            operator: crate::expression::Operator::Plus,
            right: Box::new(Expr::Literal(Literal::Number(3.0))),
        });

        assert_eq!(expr.print_ast(), "(+ 1 3)")
    }

    #[test]
    fn from_book() {
        let expr = Expr::Binary(BinaryExpr {
            left: Box::new(Expr::Unary(UnaryExpr {
                unary: Minus,
                expr: Box::new(Expr::Literal(Literal::Number(123.0))),
            })),
            operator: crate::expression::Operator::Multiply,
            right: Box::new(Expr::Grouping(GroupingExpr {
                expr: Box::new(Expr::Literal(Literal::Number(45.67))),
            })),
        });

        assert_eq!(expr.print_ast(), "(* (- 123) (group 45.67))")
    }
}
