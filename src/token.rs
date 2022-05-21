#[derive(Debug, PartialEq)]
pub enum TokenValue {
    // Groups
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,

    // Access operator
    Dot,

    // Operators
    Equals,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,

    // Comparison operators
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    EqualEqual,
    BangEqual,
    And,
    Or,
    
    // Literals
    Identifier(String),
    String(String),
    UnterminatedString(String),
    Number(i32),
    InvalidNumber(String),
    Boolean(bool),

    // Types
    Colon,

    // Control flow keywords
    If,
    Else,
    For,
    In,

    // Lambda
    BackSlash,

    InvalidCharacter(char),
    Newline,
}

#[derive(Debug)]
pub struct Token {
    pub value: TokenValue,
    pub line: usize,
}