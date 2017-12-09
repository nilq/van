#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenType {
    Int,
    Str,
    Char,
    Bool,
    Symbol,
    Operator,
    Identifier,
    Keyword,
    Whitespace,
    Indent,
    Dedent,
    EOF,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TokenPosition {
    pub line: usize,
    pub col: usize,
}

impl TokenPosition {
    pub fn new(line: usize, col: usize) -> TokenPosition {
        TokenPosition {
            line,
            col,
        }
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        TokenPosition {
            line: 1,
            col: 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub pos: TokenPosition,
    pub content: String,
}

impl Token {
    pub fn new(token_type: TokenType, pos: TokenPosition, content: String) -> Token {
        Token {
            token_type,
            pos,
            content,
        }
    }
}
