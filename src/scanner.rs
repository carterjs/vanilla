use crate::token::{Token, TokenValue};

pub struct Scanner {
    source: String,
    start: usize,
    cursor: usize,
    line: usize,
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            start: 0,
            cursor: 0,
            line: 1,
        }
    }
    
    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.cursor)
    }

    fn next(&mut self) -> Option<char> {
        let next = self.peek();
        self.cursor += 1;
        next
    }

    fn back_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }   

    fn token(&mut self, value: TokenValue) -> Token {
        // TODO: fix line and column for newline tokens
        let line = self.line;
        if value == TokenValue::Newline {
            self.line += 1;
        }

        Token {
            value,
            line,
        }
    }

    fn lexeme(&self) -> String {
        return self.source.chars().into_iter().skip(self.start).take(self.cursor - self.start).collect();
    }

    fn next_token(&mut self) -> Option<Token> {
        self.start = self.cursor;

        // Loop until getting a token
        while let Some(c) = self.next() {
            return match c {
                '(' => Some(self.token(TokenValue::LeftParen)),
                ')' => Some(self.token(TokenValue::RightParen)),
                '[' => Some(self.token(TokenValue::LeftBracket)),
                ']' => Some(self.token(TokenValue::RightBracket)),
                '{' => Some(self.token(TokenValue::LeftBrace)),
                '}' => Some(self.token(TokenValue::RightBrace)),
                '.' => Some(self.token(TokenValue::Dot)),
                '=' => {
                    if self.peek() == Some('=') {
                        self.next();
                        Some(self.token(TokenValue::EqualEqual))
                    } else {
                        Some(self.token(TokenValue::Equals))
                    }
                },
                '+' => Some(self.token(TokenValue::Plus)),
                '-' => Some(self.token(TokenValue::Minus)),
                '*' => Some(self.token(TokenValue::Star)),
                '#' => {
                    self.next();
                    while let Some(c) = self.next() {
                        if c == '\n' {
                            self.line += 1;
                            break;
                        }
                    }

                    self.start = self.cursor;
                    continue;
                },
                '/' => Some(self.token(TokenValue::Slash)),
                '\\' => Some(self.token(TokenValue::BackSlash)),
                '<' => {
                    if self.peek() == Some('=') {
                        self.next();
                        Some(self.token(TokenValue::LessThanEqual))
                    } else {
                        Some(self.token(TokenValue::LessThan))
                    }
                },
                '>' => {
                    if self.peek() == Some('=') {
                        self.next();
                        Some(self.token(TokenValue::GreaterThanEqual))
                    } else {
                        Some(self.token(TokenValue::GreaterThan))
                    }
                },
                '!' => {
                    if self.peek() == Some('=') {
                        self.next();
                        Some(self.token(TokenValue::BangEqual))
                    } else {
                        Some(self.token(TokenValue::Bang))
                    }
                },
                '&' => {
                    if self.peek() == Some('&') {
                        self.next();
                        Some(self.token(TokenValue::And))
                    } else {
                        Some(self.token(TokenValue::InvalidCharacter(c)))
                    }
                },
                '|' => {
                    if self.peek() == Some('|') {
                        self.next();
                        Some(self.token(TokenValue::Or))
                    } else {
                        Some(self.token(TokenValue::InvalidCharacter(c)))
                    }
                },
                ':' => Some(self.token(TokenValue::Colon)),

                n if n.is_numeric() => Some(self.number()),

                '"' | '`' => {
                    self.back_up();
                    let delim = self.next().unwrap();
                    Some(self.string(delim))
                },

                w if w.is_whitespace() => {
                    if w == '\n' {
                        return Some(self.token(TokenValue::Newline));
                    }

                    self.start = self.cursor;
                    continue;
                },

                c if c.is_alphabetic() || c == '_' => Some(self.item()),

                _ => Some(self.token(TokenValue::InvalidCharacter(c))),
            }
        }

        // EOF
        return None;
    }

    fn number(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if c.is_numeric() {
                self.next();
            } else {
                break;  
            }
        }

        // Parse and return
        match self.lexeme().parse::<i32>() {
            Ok(n) => self.token(TokenValue::Number(n)),
            Err(_e) => self.token(TokenValue::InvalidNumber(self.lexeme())),
        }
    }

    fn string(&mut self, delim: char) -> Token {
        let mut s = String::new();
        while let Some(c) = self.next() {
            if c == '\\' {
                match self.next() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some('`') => s.push('`'),
                    Some(c) => s.push(c),
                    None => {
                        s.push('\\');
                        break;
                    },
                }
            } else if c == delim {
                return self.token(TokenValue::String(s));
            } else if c == '\n' {
                self.line += 1;
                s.push('\n');
            } else {
                s.push(c);
            }
        }

        return self.token(TokenValue::UnterminatedString(s));
    }

    /// identifier and keywords
    fn item(&mut self) -> Token {
        while let Some(c) = self.next() {
            if !(c.is_alphabetic() || c.is_numeric() || c == '_' || c == '-' || c == '\'') {
                self.back_up();
                break;
            }
        }

        let lexeme = self.lexeme();

        // Match keywords
        match lexeme.as_str() {
            "true" => return self.token(TokenValue::Boolean(true)),
            "false" => return self.token(TokenValue::Boolean(false)),
            "for" => return self.token(TokenValue::For),
            "in" => return self.token(TokenValue::In),
            "if" => return self.token(TokenValue::If),
            "else" => return self.token(TokenValue::Else),
            _ => self.token(TokenValue::Identifier(lexeme))
        }
    }
}