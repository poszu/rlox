#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub ty: TokenType,
    pub line: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, parse_display::Display, parse_display::FromStr, PartialEq)]
#[display(style = "lowercase")]
pub enum TokenType {
    // Single-character tokens.
    #[display("(")]
    LeftParen,
    #[display(")")]
    RightParen,
    #[display("{{")]
    LeftBrace,
    #[display("}}")]
    RightBrace,
    #[display(",")]
    Comma,
    #[display(".")]
    Dot,
    #[display("-")]
    Minus,
    #[display("+")]
    Plus,
    #[display(";")]
    Semicolon,
    #[display("/")]
    Slash,
    #[display("*")]
    Star,

    // One or two character tokens
    #[display("!")]
    Bang,
    #[display("!=")]
    BangEqual,
    #[display("=")]
    Equal,
    #[display("==")]
    EqualEqual,
    #[display(">")]
    Greater,
    #[display(">=")]
    GreaterEqual,
    #[display("<")]
    Less,
    #[display("<=")]
    LessEqual,

    // Literals,
    #[display("{0}")]
    Number(f64),
    #[display("\"{0}\"")]
    String(String),
    #[display("{0}")]
    #[from_str(regex = "(?P<0>[0-9a-zA-Z_]+)")]
    Identifier(String),

    // Keywords,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // A never matching regex
    #[from_str(regex = "$.")]
    Eof,
}

#[test]
fn parsing() {
    assert_eq!("!=".parse(), Ok(TokenType::BangEqual));
    assert!("!=123".parse::<TokenType>().is_err());
}
