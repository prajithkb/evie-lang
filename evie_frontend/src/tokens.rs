use evie_common::Writer;
use num_enum::IntoPrimitive;
use std::fmt::Display;

/// All the tokens used in Evie based on Evie Grammer.
/// They are what they are named as.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen = 0,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
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
    // Special end of file keyword
    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

/// Literals
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    String(String),
    Identifier(String),
    Number(f64),
    Bool(bool),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl Literal {
    #[inline]
    pub fn opt_string(s: String) -> Option<Self> {
        Some(Literal::String(s))
    }

    #[inline]
    pub fn opt_identifier(s: String) -> Option<Self> {
        Some(Literal::Identifier(s))
    }

    #[inline]
    pub fn opt_number(s: f64) -> Option<Self> {
        Some(Literal::Number(s))
    }

    #[inline]
    pub fn opt_bool(s: bool) -> Option<Self> {
        Some(Literal::Bool(s))
    }
    #[inline]
    pub fn opt_none() -> Option<Self> {
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        line: usize,
        literal: Option<Literal>,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            literal,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "type {:?} lexeme {} literal {:?}",
            self.token_type, self.lexeme, self.literal
        ))
    }
}

/// Pretty print tokens (for debug)
pub fn pretty_print(tokens: &[Token], writer: Writer) {
    writeln!(writer, "== Tokens ==").expect("Failed to write");
    let mut line = 0;
    for token in tokens {
        if token.line != line {
            write!(writer, "{:04} ", token.line).expect("Failed to write");
            line = token.line;
        } else {
            write!(writer, "   | ").expect("Failed to write");
        }
        writeln!(
            writer,
            "{:4?} '{:width$}'",
            token.token_type,
            token.lexeme,
            width = token.lexeme.len()
        )
        .expect("Failed to write");
    }
    writeln!(writer, "============").expect("Failed to write");
}
