use super::tokenizer::Tokenizer;
use super::token::{Token, TokenType};

macro_rules! token {
    ($tokenizer:expr, $token_type:ident, $accum:expr) => {{
        token!($tokenizer , TokenType::$token_type, $accum)
    }};
    ($tokenizer:expr, $token_type:expr, $accum:expr) => {{
        let tokenizer  = $tokenizer  as &$crate::van::lexer::tokenizer::Tokenizer;
        let token_type = $token_type as $crate::van::lexer::token::TokenType;
        Token::new(token_type, tokenizer.last_position(), $accum)
    }};
}

pub struct IntLiteralMatcher;

impl Matcher for IntLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut accum = String::new();
        while let Some(c) = tokenizer.next() {
            if c.is_digit(10) {
                accum.push(c.clone());
            } else {
                break
            }
        }

        if accum.is_empty() {
            None
        } else {
            Some(token!(tokenizer, Int, accum))
        }
    }
}

pub struct StringLiteralMatcher;

impl Matcher for StringLiteralMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut raw_marker = false;
        let delimeter  = match *tokenizer.peek().unwrap() {
            '"'  => Some('"'),
            '\'' => Some('\''),
            'r' => {
                if tokenizer.peek_n(1) == Some(&'"') {
                    raw_marker = true;
                    tokenizer.advance();
                    
                    Some('"')
                } else {
                    None
                }
            },
            _ => return None,
        };

        tokenizer.advance();
        
        let mut string       = String::new();
        let mut found_escape = false;

        while !tokenizer.end() {
            if raw_marker {
                if tokenizer.peek().unwrap() == &'"' {
                    break
                }
                string.push(tokenizer.next().unwrap())
            } else if found_escape {
                string.push(
                    match tokenizer.next().unwrap() {
                        c @ '\\' | c @ '\'' | c @ '"' => c,
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        s => panic!("invalid character escape: {}", s),
                    }
                );
                found_escape = false
            } else {
                match *tokenizer.peek().unwrap() {
                    '\\' => {
                        tokenizer.next();
                        found_escape = true
                    },
                    c if c == delimeter.unwrap() => break,
                    _ => string.push(tokenizer.next().unwrap()),
                }
            }
        }
        tokenizer.advance();
        match delimeter.unwrap() {
            '"'  => {
                Some(token!(tokenizer, Str, string))
            },
            _ => {
                if string.len() == 1 {
                    Some(token!(tokenizer, Char, string))
                } else {
                    panic!("invalid char literal")
                }
            },
        }
    }
}

pub trait Matcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token>;
}

pub struct IdentifierMatcher;

impl Matcher for IdentifierMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut accum = String::new();
        while let Some(c) = tokenizer.next() {
            if c.is_alphabetic() {
                accum.push(c);
            } else {
                break
            }
        }

        if accum.is_empty() {
            None
        } else {
            Some(token!(tokenizer, Identifier, accum))
        }
    }
}

pub struct WhitespaceMatcher;

impl Matcher for WhitespaceMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let mut found = false;
        while !tokenizer.end() && tokenizer.peek().unwrap().is_whitespace() {
            found = true;
            tokenizer.next();
        }
        if found {
            Some(token!(tokenizer, Whitespace, String::new()))
        } else {
            None
        }
    }
}

pub struct ConstantCharMatcher {
    token_type: TokenType,
    constants: &'static [char],
}

impl ConstantCharMatcher {
    pub fn new(token_type: TokenType, constants: &'static [char]) -> Self {
        ConstantCharMatcher {
            token_type,
            constants,
        }
    }
}

impl Matcher for ConstantCharMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        let c = tokenizer.peek().unwrap().clone();
        for constant in self.constants {
            if c == *constant {
                tokenizer.advance();
                return Some(token!(tokenizer, self.token_type, constant.to_string()))
            }
        }
        None
    }
}

pub struct ConstantStringMatcher {
    token_type: TokenType,
    constants: &'static [&'static str],
}

impl ConstantStringMatcher {
    pub fn new(token_type: TokenType, constants: &'static [&'static str]) -> Self {
        ConstantStringMatcher {
            token_type,
            constants,
        }
    }
}

impl Matcher for ConstantStringMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        for constant in self.constants {
            let dat = tokenizer.clone().take(constant.len());
            if dat.size_hint().1.unwrap() != constant.len() {
                return None
            }
            if dat.collect::<String>() == *constant {
                tokenizer.advance_n(constant.len());
                return Some(token!(tokenizer, self.token_type.clone(), constant.to_string()))
            }
        }
        None
    }
}

pub struct KeyMatcher {
    token_type: TokenType,
    constants: Vec<String>,
}

impl KeyMatcher {
    pub fn new(token_type: TokenType, constants: Vec<String>) -> Self {
        KeyMatcher {
            token_type,
            constants,
        }
    }
}

impl Matcher for KeyMatcher {
    fn try_match(&self, tokenizer: &mut Tokenizer) -> Option<Token> {
        for constant in self.constants.clone() {
            let dat = tokenizer.clone().take(constant.len());
            if dat.size_hint().1.unwrap() != constant.len() {
                return None
            } else {
                if dat.collect::<String>() == constant {
                    if let Some(c) = tokenizer.peek_n(constant.len()) {
                        if "_?".contains(*c) || c.is_alphanumeric() {
                            return None
                        }
                    }

                    tokenizer.advance_n(constant.len());
                    return Some(token!(tokenizer, self.token_type.clone(), constant))
                }
            }
        }
        None
    }
}
