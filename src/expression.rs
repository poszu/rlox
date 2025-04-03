//! expression  → literal
//!             | unary
//!             | binary
//!             | grouping ;
//!
//! literal     → NUMBER | STRING | "true" | "false" | "nil" ;
//! grouping    → "(" expression ")" ;
//! unary       → ( "-" | "!" ) expression ;
//! binary      → expression operator expression ;
//! operator    → "==" | "!=" | "<" | "<=" | ">" | ">=" | "+"  | "-"  | "*" | "/" ;

pub enum Literal {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(parse_display::Display)]
pub enum Unary {
    #[display("!")]
    Bang,
    #[display("-")]
    Minus,
}

pub enum Expr {
    Grouping(GroupingExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Literal(Literal),
}

#[derive(parse_display::Display)]
pub enum Operator {
    #[display(">")]
    Greater,
    #[display(">=")]
    GreaterEqual,
    #[display("<")]
    Less,
    #[display("<=")]
    LessEqual,
    #[display("==")]
    Equal,
    #[display("!=")]
    NotEqual,
    #[display("-")]
    Minus,
    #[display("+")]
    Plus,
    #[display("/")]
    Divide,
    #[display("*")]
    Multiply,
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Operator,
    pub right: Box<Expr>,
}

pub struct UnaryExpr {
    pub unary: Unary,
    pub expr: Box<Expr>,
}

pub struct GroupingExpr {
    pub expr: Box<Expr>,
}
