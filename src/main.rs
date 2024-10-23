use std::{
    fs::{self},
    str::Chars,
};

use parser::Parser;

mod parser;

#[derive(Clone, Debug)]
enum Token {
    Identifier(String),
    Number(f64),
    Operator(Operator),
    Keyword(Keyword),
    Semi,
}

#[derive(Clone, Debug)]
enum Keyword {
    Return,
}

#[derive(Clone, Debug)]
enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
}

struct Lexer<'a> {
    input: Chars<'a>,
    current: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
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

    fn tokenize_number(&mut self) -> Token {
        let mut num_str = String::new();

        // negative sign
        if let Some('-') = self.peek() {
            num_str.push('-');
            self.next();
        };

        // integer part
        num_str.push_str(&self.consume_while(|c| c.is_digit(10)));

        // decimal point
        if let Some('.') = self.peek() {
            num_str.push('.');
            self.next();
            // fractional part
            num_str.push_str(&self.consume_while(|c| c.is_digit(10)));
        }

        Token::Number(num_str.parse::<f64>().unwrap())
    }

    fn tokenize_identifier(&mut self) -> Token {
        let ident_str = self.consume_while(|c| c.is_alphanumeric() || c == '_');
        if ident_str == "return" {
            return Token::Keyword(Keyword::Return);
        };
        Token::Identifier(ident_str)
    }

    fn tokenize_operator(&mut self) -> Token {
        let op_str: Option<char> = self.peek();
        let token = match op_str {
            Some('+') => Token::Operator(Operator::Plus),
            Some('-') => Token::Operator(Operator::Minus),
            Some('*') => Token::Operator(Operator::Multiply),
            Some('/') => Token::Operator(Operator::Divide),
            _ => unimplemented!("Tokenize operator {:?}", op_str),
        };
        self.next();
        token
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek() {
            let token = match c {
                '0'..='9' => self.tokenize_number(),
                '-' => {
                    let mut result = Token::Operator(Operator::Minus);

                    if let Some(next_char) = self.input.clone().next() {
                        if next_char.is_digit(10) {
                            result = self.tokenize_number();
                        }
                    }

                    if matches!(result, Token::Operator(Operator::Minus)) {
                        self.next();
                    }

                    result
                }
                'a'..='z' | 'A'..='Z' | '_' => self.tokenize_identifier(),
                '+' | '*' | '/' => self.tokenize_operator(),
                ';' => {
                    self.next();
                    Token::Semi
                }
                ch => {
                    if ch.is_whitespace() {
                        self.next();
                        continue;
                    }
                    unimplemented!("Tokenize {:?}", c)
                }
            };
            tokens.push(token);
        }

        tokens
    }
}

fn main() {
    let contents = fs::read_to_string("test.boo").expect("Unable to read file to string");
    let mut lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize();
    println!("Tokens: {:?}", tokens);
}
