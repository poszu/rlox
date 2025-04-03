use std::borrow::BorrowMut;

use anyhow::anyhow;
use itertools::Itertools;

use crate::token::{Token, TokenType};

pub fn scan_tokens(mut source: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut line = 1;
    while !source.is_empty() {
        let (processed_lines, remainder, result) = scan_token(source);
        match result {
            Ok(token) => {
                if let Some(token) = token {
                    tokens.push(Token { ty: token, line });
                } else {
                    break;
                }
                line += processed_lines;
            }
            Err(err) => {
                eprintln!("{} | failed to process token: {}", line, err);
            }
        }
        source = remainder;
    }

    tokens.push(Token {
        ty: crate::token::TokenType::Eof,
        line,
    });
    Ok(tokens)
}

fn scan_token(input: &str) -> (usize, &str, anyhow::Result<Option<TokenType>>) {
    let mut lines = 0;
    let input = input.trim_start();
    if input.is_empty() {
        return (0, input, Ok(None));
    }
    let mut chars = input.chars();

    if let Some(c) = chars.next() {
        let token = match c {
            '\n' => {
                lines += 1;
                None
            }
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
                        let _ = chars.find(|c| *c == '\n');
                        return (1, chars.as_str(), Ok(None));
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
                let mut closed = false;
                let str_content: String = chars
                    .borrow_mut()
                    .take_while(|c| {
                        if *c == '\n' {
                            lines += 1;
                        }
                        closed = *c == '"';
                        !closed
                    })
                    .collect();
                if !closed {
                    return (
                        lines,
                        chars.as_str(),
                        Err(anyhow::anyhow!("Unterminated string")),
                    );
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
                for _ in 0..fractional_chars {
                    chars.next();
                }

                let number_str = &input[..digits + fractional_chars];
                let num = match number_str.parse::<f64>() {
                    Ok(n) => n,
                    Err(e) => {
                        return (
                            0,
                            chars.as_str(),
                            Err(anyhow!("failed to parse number from '{number_str}': {e:?}")),
                        );
                    }
                };
                Some(TokenType::Number(num))
            }
            c if !(c.is_alphanumeric() || c == '_') => {
                return (
                    0,
                    chars.as_str(),
                    Err(anyhow::anyhow!("Invalid character: '{c}'")),
                );
            }
            _ => None,
        };

        if let Some(token) = token {
            return (lines, chars.as_str(), Ok(Some(token)));
        }
    }

    // Try as a keyword or an identifier:
    let pos_word_end = input
        .chars()
        .position(|c| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(input.len());

    let (word, remainder) = input.split_at(pos_word_end);
    match word.parse::<TokenType>() {
        Ok(token) => (lines, remainder, Ok(Some(token))),
        Err(e) => (lines, remainder, Err(e.into())),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        scanner::{scan_token, scan_tokens},
        token::{Token, TokenType},
    };

    #[test]
    fn scanning_line() {
        let expected = &[
            Token {
                line: 1,
                ty: TokenType::LeftParen,
            },
            Token {
                line: 1,
                ty: TokenType::RightParen,
            },
            Token {
                line: 1,
                ty: TokenType::Eof,
            },
        ];
        assert_eq!(scan_tokens("()").unwrap(), expected);
        assert_eq!(scan_tokens("(    )").unwrap(), expected);

        let mut input = "!*+-/=<> <= === ";
        let expected_tokens = &[
            Some(TokenType::Bang),
            Some(TokenType::Star),
            Some(TokenType::Plus),
            Some(TokenType::Minus),
            Some(TokenType::Slash),
            Some(TokenType::Equal),
            Some(TokenType::Less),
            Some(TokenType::Greater),
            Some(TokenType::LessEqual),
            Some(TokenType::EqualEqual),
            Some(TokenType::Equal),
            None,
        ];
        let mut idx = 0;
        while !input.is_empty() {
            let (processed_line, remainder, res) = scan_token(input);
            assert_eq!(0, processed_line);
            assert_eq!(expected_tokens[idx], res.unwrap());
            input = remainder;
            idx += 1;
        }
    }

    #[test]
    fn continues_scanning_on_errors() {
        let input = "123 + @200"; // '@' is invalid
        let expected = &[
            Token {
                line: 1,
                ty: TokenType::Number(123.0),
            },
            Token {
                line: 1,
                ty: TokenType::Plus,
            },
            Token {
                line: 1,
                ty: TokenType::Number(200.0),
            },
            Token {
                line: 1,
                ty: TokenType::Eof,
            },
        ];
        assert_eq!(scan_tokens(input).unwrap(), expected);
    }

    #[test]
    fn scan_empty() {
        let (lines, _, res) = scan_token("");
        assert_eq!(0, lines);
        assert_eq!(res.unwrap(), None);
    }

    #[test]
    fn scan_invalid_characters() {
        for c in &[":", "@", "#", "$", "%", "^", "&", "[", "]"] {
            let (lines, _, res) = scan_token(c);
            assert_eq!(0, lines);
            assert!(res.is_err());
        }
    }

    #[test]
    fn scan_left_paren() {
        let (_, _, res) = scan_token("(");
        assert_eq!(res.unwrap(), Some(TokenType::LeftParen));

        let (_, rem, res) = scan_token("(foo");
        assert_eq!("foo", rem);
        assert_eq!(res.unwrap(), Some(TokenType::LeftParen));
    }

    #[test]
    fn scan_right_paren() {
        let (_, rem, token) = scan_token(")foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::RightParen));
    }

    #[test]
    fn scan_left_brace() {
        let (_, rem, token) = scan_token("{foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::LeftBrace));
    }

    #[test]
    fn scan_right_brace() {
        let (_, rem, token) = scan_token("}foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::RightBrace));
    }

    #[test]
    fn scan_dot() {
        let (_, rem, token) = scan_token(".foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Dot));
    }
    #[test]
    fn scan_comma() {
        let (_, rem, token) = scan_token(",foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Comma));
    }
    #[test]
    fn scan_minus() {
        let (_, rem, token) = scan_token("-foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Minus));
    }
    #[test]
    fn scan_plus() {
        let (_, rem, token) = scan_token("+foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Plus));
    }
    #[test]
    fn scan_star() {
        let (_, rem, token) = scan_token("*foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Star));
    }
    #[test]
    fn scan_slash() {
        let (_, rem, token) = scan_token("/foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Slash));
    }

    #[test]
    fn scan_comment() {
        let (lines, rem, token) = scan_token("//foo");
        assert_eq!(1, lines);
        assert_eq!("", rem);
        assert_eq!(token.unwrap(), None);
    }

    #[test]
    fn scan_bang() {
        let (_, rem, token) = scan_token("!foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Bang));

        let (_, rem, token) = scan_token("!=foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::BangEqual));

        let (_, rem, token) = scan_token("! =foo");
        assert_eq!(" =foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Bang));
    }
    #[test]
    fn scan_less() {
        let (_, rem, token) = scan_token("<foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Less));

        let (_, rem, token) = scan_token("<=foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::LessEqual));
    }
    #[test]
    fn scan_greater() {
        let (_, rem, token) = scan_token(">foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Greater));

        let (_, rem, token) = scan_token(">=foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::GreaterEqual));
    }

    #[test]
    fn scan_equal() {
        let (_, rem, token) = scan_token("=foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::Equal));

        let (_, rem, token) = scan_token("==foo");
        assert_eq!("foo", rem);
        assert_eq!(token.unwrap(), Some(TokenType::EqualEqual));
    }

    #[test]
    fn scan_string() {
        let (_, rem, token) = scan_token(r#""some string" some leftovers"#);
        assert_eq!(" some leftovers", rem);
        assert_eq!(
            token.unwrap(),
            Some(TokenType::String("some string".to_string()))
        );

        let (_, _, token) = scan_token(r#""an unterminated string"#);
        assert!(token.is_err());
    }

    #[test]
    fn scan_multiline_string() {
        let (new_lines, _, res) = scan_token(
            r#""some
multiline
string""#,
        );
        assert_eq!(2, new_lines);
        assert_eq!(
            Some(TokenType::String("some\nmultiline\nstring".to_string())),
            res.unwrap()
        );
    }

    #[test]
    fn scan_number() {
        let (_, _, res) = scan_token("12.34");
        assert_eq!(res.unwrap(), Some(TokenType::Number(12.34)));
        let (_, _, res) = scan_token("10");
        assert_eq!(res.unwrap(), Some(TokenType::Number(10.0)));
        let (_, _, res) = scan_token("10.");
        assert_eq!(res.unwrap(), Some(TokenType::Number(10.0)));
        let (_, _, res) = scan_token("12..34");
        assert_eq!(res.unwrap(), Some(TokenType::Number(12.0)));
        let (_, _, res) = scan_token("1.2.3.4");
        assert_eq!(res.unwrap(), Some(TokenType::Number(1.2)));
        let (_, _, res) = scan_token(".1234");
        assert_eq!(res.unwrap(), Some(TokenType::Dot));
    }

    #[test]
    fn scan_identifier() {
        let (_, _, res) = scan_token("eof");
        assert_eq!(res.unwrap(), Some(TokenType::Identifier("eof".to_string())));
        let (_, _, res) = scan_token("foo");
        assert_eq!(res.unwrap(), Some(TokenType::Identifier("foo".to_string())));
        let (_, _, res) = scan_token("_");
        assert_eq!(res.unwrap(), Some(TokenType::Identifier("_".to_string())));
        let (_, _, res) = scan_token("   _123");
        assert_eq!(
            res.unwrap(),
            Some(TokenType::Identifier("_123".to_string()))
        );
        let (_, _, res) = scan_token("_for");
        assert_eq!(
            res.unwrap(),
            Some(TokenType::Identifier("_for".to_string()))
        );
    }
    #[test]
    fn scan_keyword() {
        let tests = [
            ("class", TokenType::Class),
            ("and", TokenType::And),
            ("or", TokenType::Or),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("true", TokenType::True),
            ("fun", TokenType::Fun),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("this", TokenType::This),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ];
        for (input, token) in tests {
            let (lines, rem, res) = scan_token(input);
            assert_eq!("", rem);
            assert_eq!(0, lines);
            assert_eq!(Some(token), res.unwrap());
        }
    }
}
