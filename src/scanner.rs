use std::borrow::BorrowMut;

use itertools::Itertools;

use crate::token::{Token, TokenType};

pub fn scan_tokens(source: &str) -> Result<impl Iterator<Item = Token>, anyhow::Error> {
    let mut tokens = Vec::new();
    let mut total_lines = 0;
    for (line_no, line) in source.lines().enumerate() {
        total_lines = line_no;
        let (line_remainder, tokens_in_line) = scan_line(line)?;
        assert!(
            line_remainder.is_empty(),
            "Not every character of line: '{line}' was consumed. Leftovers: '{line_remainder}'"
        );

        tokens.extend(tokens_in_line.into_iter().map(|t| Token {
            ty: t,
            line: line_no,
        }));
    }

    tokens.push(Token {
        ty: crate::token::TokenType::Eof,
        line: total_lines,
    });
    Ok(tokens.into_iter())
}

fn scan_line(mut input_line: &str) -> Result<(&str, Vec<TokenType>), anyhow::Error> {
    let mut tokens = Vec::new();
    loop {
        let (line, maybe_token) = scan_token(input_line)?;
        input_line = line;
        if let Some(token) = maybe_token {
            tokens.push(token);
        } else {
            return Ok((line, tokens));
        }
    }
}

fn scan_token(input: &str) -> Result<(&str, Option<TokenType>), anyhow::Error> {
    let input = input.trim_start();
    if input.is_empty() {
        return Ok((input, None));
    }
    let mut chars = input.chars();

    if let Some(c) = chars.next() {
        let token = match c {
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            '.' => Some(TokenType::Dot),
            ',' => Some(TokenType::Comma),
            '-' => Some(TokenType::Minus),
            '+' => Some(TokenType::Plus),
            ';' => Some(TokenType::Semicolon),
            '*' => Some(TokenType::Star),
            '/' => {
                // "/" or "//"
                match chars.clone().peekable().peek() {
                    Some('/') => {
                        // Consume till end of line
                        return Ok(("", None));
                    }
                    _ => Some(TokenType::Slash),
                }
            }
            '!' => {
                // "!" or "!="
                match chars.clone().peekable().peek() {
                    Some('=') => {
                        chars.next();
                        Some(TokenType::BangEqual)
                    }
                    _ => Some(TokenType::Bang),
                }
            }
            '=' => {
                // "=" or "=="
                match chars.clone().peekable().peek() {
                    Some('=') => {
                        chars.next();
                        Some(TokenType::EqualEqual)
                    }
                    _ => Some(TokenType::Equal),
                }
            }
            '>' => {
                // ">" or ">="
                match chars.clone().peekable().peek() {
                    Some('=') => {
                        chars.next();
                        Some(TokenType::GreaterEqual)
                    }
                    _ => Some(TokenType::Greater),
                }
            }
            '<' => {
                // "=" or "=="
                match chars.clone().peekable().peek() {
                    Some('=') => {
                        chars.next();
                        Some(TokenType::LessEqual)
                    }
                    _ => Some(TokenType::Less),
                }
            }
            '"' => {
                // TODO: figure out support for multi-line strings
                let mut closed = false;
                let str_content: String = chars
                    .borrow_mut()
                    .take_while(|c| {
                        closed = *c == '"';
                        !closed
                    })
                    .collect();
                if !closed {
                    return Err(anyhow::anyhow!("Unterminated string"));
                }
                Some(TokenType::String(str_content))
            }
            '0'..='9' => {
                let digits = 1 + chars.peeking_take_while(|c| c.is_ascii_digit()).count();
                let mut peek = chars.clone();
                let fractional_chars = if let Some('.') = peek.next() {
                    let digits = peek.take_while(|c| c.is_ascii_digit()).count();
                    if digits > 0 {
                        digits + 1
                    } else {
                        0
                    }
                } else {
                    0
                };

                let num = input[..digits + fractional_chars].parse::<f64>()?;
                for _ in 0..fractional_chars {
                    chars.next();
                }
                Some(TokenType::Number(num))
            }
            c if !(c.is_alphanumeric() || c == '_') => {
                return Err(anyhow::anyhow!("Invalid character: '{c}'"));
            }
            _ => None,
        };

        if let Some(token) = token {
            return Ok((chars.as_str(), Some(token)));
        }
    }

    // Try as a keyword or an identifier:
    let pos_word_end = input
        .chars()
        .take_while_ref(|c| c.is_alphanumeric() || *c == '_')
        .count();
    let word = &input[..pos_word_end];
    let token = word.parse::<TokenType>()?;
    Ok((&input[pos_word_end..], Some(token)))
}

#[cfg(test)]
mod tests {
    use crate::{scanner::scan_token, token::TokenType};

    use super::scan_line;

    #[test]
    fn scanning_line() {
        assert_eq!(
            scan_line("()").unwrap(),
            ("", vec![TokenType::LeftParen, TokenType::RightParen])
        );

        assert_eq!(
            scan_line("(    )").unwrap(),
            ("", vec![TokenType::LeftParen, TokenType::RightParen])
        );
        assert_eq!(
            scan_line("!*+-/=<> <= == // operators").unwrap(),
            (
                "",
                vec![
                    TokenType::Bang,
                    TokenType::Star,
                    TokenType::Plus,
                    TokenType::Minus,
                    TokenType::Slash,
                    TokenType::Equal,
                    TokenType::Less,
                    TokenType::Greater,
                    TokenType::LessEqual,
                    TokenType::EqualEqual,
                ]
            )
        );
    }

    #[test]
    fn scan_empty() {
        let token = scan_token("");
        assert_eq!(token.unwrap(), ("", None));
    }

    #[test]
    fn scan_invalid_characters() {
        for c in &[':', '@', '#', '$', '%', '^', '&', '[', ']'] {
            assert!(scan_token(&c.to_string()).is_err());
        }
    }

    #[test]
    fn scan_left_paren() {
        let token = scan_token("(");
        assert_eq!(token.unwrap(), ("", Some(TokenType::LeftParen)));

        let token = scan_token("(foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::LeftParen)));
    }

    #[test]
    fn scan_right_paren() {
        let token = scan_token(")foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::RightParen)));
    }

    #[test]
    fn scan_left_brace() {
        let token = scan_token("{foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::LeftBrace)));
    }

    #[test]
    fn scan_right_brace() {
        let token = scan_token("}foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::RightBrace)));
    }

    #[test]
    fn scan_dot() {
        let token = scan_token(".foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Dot)));
    }
    #[test]
    fn scan_comma() {
        let token = scan_token(",foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Comma)));
    }
    #[test]
    fn scan_minus() {
        let token = scan_token("-foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Minus)));
    }
    #[test]
    fn scan_plus() {
        let token = scan_token("+foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Plus)));
    }
    #[test]
    fn scan_star() {
        let token = scan_token("*foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Star)));
    }
    #[test]
    fn scan_slash() {
        let token = scan_token("/foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Slash)));
    }

    #[test]
    fn scan_comment() {
        let token = scan_token("//foo");
        assert_eq!(token.unwrap(), ("", None));
    }

    #[test]
    fn scan_bang() {
        let token = scan_token("!foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Bang)));

        let token = scan_token("!=foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::BangEqual)));

        let token = scan_token("! =foo");
        assert_eq!(token.unwrap(), (" =foo", Some(TokenType::Bang)));
    }
    #[test]
    fn scan_less() {
        let token = scan_token("<foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Less)));

        let token = scan_token("<=foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::LessEqual)));
    }
    #[test]
    fn scan_greater() {
        let token = scan_token(">foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Greater)));

        let token = scan_token(">=foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::GreaterEqual)));
    }

    #[test]
    fn scan_equal() {
        let token = scan_token("=foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::Equal)));

        let token = scan_token("==foo");
        assert_eq!(token.unwrap(), ("foo", Some(TokenType::EqualEqual)));
    }

    #[test]
    fn scan_string() {
        let token = scan_token(r#""some string" some leftovers"#);
        assert_eq!(
            token.unwrap(),
            (
                " some leftovers",
                Some(TokenType::String("some string".to_string()))
            )
        );

        let token = scan_token(r#""an unterminated string"#);
        assert!(token.is_err());
    }

    #[test]
    fn scan_number() {
        assert_eq!(
            scan_token("12.34").unwrap(),
            ("", Some(TokenType::Number(12.34)))
        );
        assert_eq!(
            scan_token("10").unwrap(),
            ("", Some(TokenType::Number(10.0)))
        );
        assert_eq!(
            scan_token("10.").unwrap(),
            (".", Some(TokenType::Number(10.0)))
        );
        assert_eq!(
            scan_token("12..34").unwrap(),
            ("..34", Some(TokenType::Number(12.0)))
        );
        assert_eq!(
            scan_token("1.2.3.4").unwrap(),
            (".3.4", Some(TokenType::Number(1.2)))
        );
        assert_eq!(scan_token(".1234").unwrap(), ("1234", Some(TokenType::Dot)));
    }

    #[test]
    fn scan_identifier() {
        assert_eq!(
            scan_token("eof").unwrap(),
            ("", Some(TokenType::Identifier("eof".to_string())))
        );
        assert_eq!(
            scan_token("Eof").unwrap(),
            ("", Some(TokenType::Identifier("Eof".to_string())))
        );
        assert_eq!(
            scan_token("foo").unwrap(),
            ("", Some(TokenType::Identifier("foo".to_string())))
        );
        assert_eq!(
            scan_token("_").unwrap(),
            ("", Some(TokenType::Identifier("_".to_string())))
        );
        assert_eq!(
            scan_token("   _123").unwrap(),
            ("", Some(TokenType::Identifier("_123".to_string())))
        );
        assert_eq!(
            scan_token("_for").unwrap(),
            ("", Some(TokenType::Identifier("_for".to_string())))
        );
    }
    #[test]
    fn scan_keyword() {
        assert_eq!(scan_token("and").unwrap(), ("", Some(TokenType::And)));
        assert_eq!(scan_token("class").unwrap(), ("", Some(TokenType::Class)));
        assert_eq!(scan_token("else").unwrap(), ("", Some(TokenType::Else)));
        assert_eq!(scan_token("false").unwrap(), ("", Some(TokenType::False)));
        assert_eq!(scan_token("fun").unwrap(), ("", Some(TokenType::Fun)));
        assert_eq!(scan_token("for").unwrap(), ("", Some(TokenType::For)));
        assert_eq!(scan_token("if").unwrap(), ("", Some(TokenType::If)));
        assert_eq!(scan_token("nil").unwrap(), ("", Some(TokenType::Nil)));
        assert_eq!(scan_token("or").unwrap(), ("", Some(TokenType::Or)));
        assert_eq!(scan_token("print").unwrap(), ("", Some(TokenType::Print)));
        assert_eq!(scan_token("return").unwrap(), ("", Some(TokenType::Return)));
        assert_eq!(scan_token("super").unwrap(), ("", Some(TokenType::Super)));
        assert_eq!(scan_token("this").unwrap(), ("", Some(TokenType::This)));
        assert_eq!(scan_token("true").unwrap(), ("", Some(TokenType::True)));
        assert_eq!(scan_token("var").unwrap(), ("", Some(TokenType::Var)));
        assert_eq!(scan_token("while").unwrap(), ("", Some(TokenType::While)));
    }
}
