use std::str::Chars;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(f64),
    String(String),
    Boolean(bool),
    Operator(Operator),
    Keyword(Keyword),
    Type(Type),
    Period,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Arrow,
    Comma,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Type {
    Str,
    Num,
    Bool,
    Void,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Keyword {
    Fun,
    Return,
    If,
    Else,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    // math
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Power,    // **
    Modulo,   // %

    // assignment
    AssignEquals, // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    PowAssign,    // **=
    ModAssign,    // %=

    // comparison
    Equals,             // ==
    NotEquals,          // !=
    GreaterThan,        // >
    LessThan,           // <
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
}

pub struct Lexer<'a> {
    input: Chars<'a>,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input: input.chars(),
            current: None,
        };
        lexer.next();
        lexer
    }

    fn next(&mut self) {
        self.current = self.input.next();
    }

    fn peek(&self) -> Option<char> {
        self.current
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if predicate(c) {
                result.push(c);
                self.next();
            } else {
                break;
            }
        }
        result
    }

    fn tokenize_number(&mut self) -> Result<Token, String> {
        let mut num_str = String::new();

        // negative sign
        if let Some('-') = self.peek() {
            num_str.push('-');
            self.next();
        };

        // integer part
        let int_part = self.consume_while(|c| c.is_digit(10));
        if int_part.is_empty() && num_str == "-" {
            return Err("Expected digits after '-'".to_string());
        }
        num_str.push_str(&int_part);

        // decimal point
        if let Some('.') = self.peek() {
            self.next();
            num_str.push('.');
            let dec_part = self.consume_while(|c| c.is_digit(10));
            if dec_part.is_empty() {
                return Err("Expected digits after '.'".to_string());
            }
            num_str.push_str(&dec_part);
        };

        num_str
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse number: {}", e))
            .map(Token::Number)
    }

    fn tokenize_string(&mut self) -> Result<Token, String> {
        // consume the opening quote
        self.next();
        let str_content = self.consume_while(|c| c != '"');
        match self.peek() {
            Some('"') => {
                // consume the closing quote
                self.next();
                Ok(Token::String(str_content))
            }
            Some(c) => Err(format!("Unexpected character in string: {}", c)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn tokenize_identifier(&mut self) -> Result<Token, String> {
        let ident_str = self.consume_while(|c| c.is_alphanumeric() || c == '_');

        let token = match ident_str.as_str() {
            // keywords
            "fun" => Token::Keyword(Keyword::Fun),
            "return" => Token::Keyword(Keyword::Return),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),

            // types
            "str" => Token::Type(Type::Str),
            "num" => Token::Type(Type::Num),
            "bool" => Token::Type(Type::Bool),

            // booleans
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),

            // regular identifier
            _ => Token::Identifier(ident_str),
        };

        Ok(token)
    }

    fn tokenize_operator(&mut self) -> Result<Token, String> {
        let current = self.peek().ok_or("Expected operator, found end of input")?;

        // consume the operator
        self.next();

        let next = self.peek();

        let token = match (current, next) {
            // two char operators
            ('=', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::Equals)
            }
            ('!', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::NotEquals)
            }
            ('-', Some('>')) => {
                self.next(); // consume the second operator
                Token::Arrow
            }
            ('>', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::GreaterThanOrEqual)
            }
            ('<', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::LessThanOrEqual)
            }
            ('+', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::AddAssign)
            }
            ('-', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::SubAssign)
            }
            ('*', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::MulAssign)
            }
            ('*', Some('*')) => {
                self.next(); // consume the second operator

                if let Some('=') = self.peek() {
                    // consume the '='
                    self.next();
                    Token::Operator(Operator::PowAssign)
                } else {
                    Token::Operator(Operator::Power)
                }
            }
            ('%', Some('=')) => {
                self.next(); // consume the second operator
                Token::Operator(Operator::ModAssign)
            }

            // single char operators
            ('+', _) => Token::Operator(Operator::Plus),
            ('-', _) => Token::Operator(Operator::Minus),
            ('*', _) => Token::Operator(Operator::Multiply),
            ('/', _) => Token::Operator(Operator::Divide),
            ('%', _) => Token::Operator(Operator::Modulo),
            ('=', _) => Token::Operator(Operator::AssignEquals),
            ('>', _) => Token::Operator(Operator::GreaterThan),
            ('<', _) => Token::Operator(Operator::LessThan),
            ('(', _) => Token::LeftParen,
            (')', _) => Token::RightParen,
            ('{', _) => Token::LeftBrace,
            ('}', _) => Token::RightBrace,
            (',', _) => Token::Comma,

            c => return Err(format!("Unexpected operator: {:?}", c)),
        };

        Ok(token)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            let token = match c {
                '0'..='9' => self.tokenize_number()?,
                '"' => self.tokenize_string()?,
                '-' => {
                    if let Some(next_char) = self.input.clone().next() {
                        if next_char.is_digit(10) {
                            self.tokenize_number()?
                        } else {
                            self.tokenize_operator()?
                        }
                    } else {
                        self.tokenize_operator()?
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier()?,
                '/' => {
                    // consume the '/'
                    self.next();
                    match self.peek() {
                        Some('/') => {
                            // consume the '/'
                            self.next();
                            self.consume_while(|c| c != '\n');
                            if let Some('\n') = self.peek() {
                                // consume the '\n'
                                self.next();
                            }
                            continue;
                        }
                        Some('=') => {
                            // consume the '='
                            self.next();
                            Token::Operator(Operator::DivAssign)
                        }
                        _ => Token::Operator(Operator::Divide),
                    }
                }
                '+' | '>' | '=' | '*' | '(' | ')' | '{' | '}' | ',' | '!' | '%' => {
                    self.tokenize_operator()?
                }
                '.' => {
                    self.next();
                    Token::Period
                }
                ';' => {
                    self.next();
                    continue;
                }
                c if c.is_whitespace() => {
                    self.next();
                    continue;
                }
                c => return Err(format!("Unexpected character: {}", c)),
            };
            tokens.push(token);
        }

        Ok(tokens)
    }
}
