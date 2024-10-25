use std::{iter::Peekable, vec};

use crate::{Keyword, Operator, Token, Type};

#[derive(Clone, Debug)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Statement(Box<ASTNode>),
    ReturnStatement(Box<ASTNode>),
    BinaryOperation {
        left: Box<ASTNode>,
        op: Operator,
        right: Box<ASTNode>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<ASTNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
    },
    VariableDeclaration {
        var_type: Type,
        name: String,
        value: Box<ASTNode>,
    },
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
}

#[derive(Clone, Debug)]
pub struct Parameter {
    name: String,
    param_type: Type,
    optional: bool,
}

pub struct Parser {
    tokens: Peekable<vec::IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode, String> {
        match self.tokens.next() {
            Some(Token::Identifier(ident)) => {
                // check if its a function call
                if let Some(Token::LeftParen) = self.tokens.peek() {
                    self.tokens.next();
                    self.parse_function_call(ident)
                } else {
                    Ok(ASTNode::Identifier(ident))
                }
            }
            Some(Token::Number(num)) => Ok(ASTNode::NumberLiteral(num)),
            Some(Token::String(str)) => Ok(ASTNode::StringLiteral(str)),
            Some(Token::Boolean(bool)) => Ok(ASTNode::BooleanLiteral(bool)),
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_binary_operation(&mut self) -> Result<ASTNode, String> {
        let mut result = self.parse_primary()?;
        while let Some(Token::Operator(op)) = self.tokens.peek() {
            let operator = op.clone();
            self.tokens.next();
            let right = self.parse_primary()?;
            result = ASTNode::BinaryOperation {
                left: Box::new(result),
                op: operator,
                right: Box::new(right),
            };
        }
        Ok(result)
    }

    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        match (self.tokens.next(), self.tokens.next()) {
            (Some(Token::Type(param_type)), Some(Token::Identifier(name))) => {
                let mut optional = false;
                if let Some(Token::Star) = self.tokens.peek() {
                    self.tokens.next();
                    optional = true;
                }

                Ok(Parameter {
                    name,
                    param_type,
                    optional,
                })
            }
            (Some(token1), Some(token2)) => Err(format!(
                "Expected type and identifier, found {:?} {:?}",
                token1, token2
            )),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, String> {
        let mut parameters = Vec::new();

        // empty parameter list (no parameters)
        if let Some(Token::RightParen) = self.tokens.peek() {
            self.tokens.next();
            return Ok(parameters);
        };

        parameters.push(self.parse_parameter()?);

        while let Some(Token::Comma) = self.tokens.peek() {
            self.tokens.next();
            parameters.push(self.parse_parameter()?);
        }

        match self.tokens.next() {
            Some(Token::RightParen) => Ok(parameters),
            Some(token) => Err(format!("Expected ')', found {:?}", token)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_function_body(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut body = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if matches!(token, Token::RightBrace) {
                break;
            }
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }

    fn parse_function_declaration(&mut self) -> Result<ASTNode, String> {
        // parse function name
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => name,
            Some(token) => return Err(format!("Expected function name, found {:?}", token)),
            _ => return Err("Expected function name, found end of input".to_string()),
        };

        // parse opening parenthesis
        match self.tokens.next() {
            Some(Token::LeftParen) => (),
            Some(token) => return Err(format!("Expected '(', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        // parse parameters
        let parameters = self.parse_parameter_list()?;

        // parse return type
        let return_type = if let Some(Token::Arrow) = self.tokens.peek() {
            self.tokens.next();
            match self.tokens.next() {
                Some(Token::Type(return_type)) => Some(return_type),
                Some(token) => return Err(format!("Expected return type, found {:?}", token)),
                _ => return Err("Unexpected end of input".to_string()),
            }
        } else {
            None
        };

        // parse opening brace
        match self.tokens.next() {
            Some(Token::LeftBrace) => (),
            Some(token) => return Err(format!("Expected '{{', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        let body = self.parse_function_body()?;

        // parse closing brace
        match self.tokens.next() {
            Some(Token::RightBrace) => (),
            Some(token) => return Err(format!("Expected '}}', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        Ok(ASTNode::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_function_call(&mut self, name: String) -> Result<ASTNode, String> {
        let mut arguments = Vec::new();

        // empty argument list (no arguments)
        if let Some(Token::RightParen) = self.tokens.peek() {
            self.tokens.next();
            return Ok(ASTNode::FunctionCall { name, arguments });
        };

        arguments.push(self.parse_expression()?);

        while let Some(Token::Comma) = self.tokens.peek() {
            self.tokens.next();
            arguments.push(self.parse_expression()?);
        }

        match self.tokens.next() {
            Some(Token::RightParen) => return Ok(ASTNode::FunctionCall { name, arguments }),
            Some(token) => Err(format!("Expected ')', found {:?}", token)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_variable_declaration(&mut self, var_type: Type) -> Result<ASTNode, String> {
        match self.tokens.next() {
            Some(Token::Identifier(name)) => match self.tokens.next() {
                Some(Token::Equals) => {
                    let value = self.parse_expression()?;
                    Ok(ASTNode::VariableDeclaration {
                        name,
                        var_type,
                        value: Box::new(value),
                    })
                }
                Some(token) => Err(format!("Expected '=', found {:?}", token)),
                _ => Err("Unexpected end of input".to_string()),
            },
            Some(token) => Err(format!("Expected identifier, found {:?}", token)),
            _ => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        match self.tokens.peek() {
            Some(Token::Keyword(Keyword::Fun)) => {
                self.tokens.next();
                self.parse_function_declaration()
            }
            Some(Token::Type(t)) => {
                let var_type = t.clone();
                self.tokens.next();
                self.parse_variable_declaration(var_type)
            }
            Some(Token::Keyword(Keyword::Return)) => {
                self.tokens.next();
                let expression = self.parse_expression()?;
                Ok(ASTNode::ReturnStatement(Box::new(expression)))
            }
            _ => {
                let expression = self.parse_expression()?;
                Ok(ASTNode::Statement(Box::new(expression)))
            }
        }
    }

    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        self.parse_binary_operation()
    }

    pub fn parse_program(&mut self) -> Result<ASTNode, String> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(statements))
    }
}
