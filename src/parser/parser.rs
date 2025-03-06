use std::{iter::Peekable, vec};

use crate::lexer::{Keyword, Operator, Token, Type};

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
    IfStatement {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
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
    pub name: String,
    pub param_type: Type,
    pub optional: bool,
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
            Some(Token::LeftParen) => {
                let expr = self.parse_expression()?;

                // make sure we have a closing parenthensis
                match self.tokens.next() {
                    Some(Token::RightParen) => Ok(expr),
                    Some(token) => Err(format!("Expected ')', found {:?}", token)),
                    _ => Err("Unexpected end of input".to_string()),
                }
            }
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

    fn parse_parameter(&mut self) -> Result<Parameter, String> {
        match (self.tokens.next(), self.tokens.next()) {
            (Some(Token::Type(param_type)), Some(Token::Identifier(name))) => {
                let mut optional = false;
                if let Some(Token::Operator(Operator::Multiply)) = self.tokens.peek() {
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
        let mut statements = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if matches!(token, Token::RightBrace) {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_block(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if matches!(token, Token::RightBrace) {
                break;
            }
            statements.push(self.parse_statement()?);
        }

        match self.tokens.next() {
            Some(Token::RightBrace) => Ok(statements),
            Some(token) => return Err(format!("Expected '}}', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        }
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

    fn parse_if_statement(&mut self) -> Result<ASTNode, String> {
        // parse condition
        match self.tokens.next() {
            Some(Token::LeftParen) => (),
            Some(token) => return Err(format!("Expected '(' after 'if', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        let condition = self.parse_expression()?;

        match self.tokens.next() {
            Some(Token::RightParen) => (),
            Some(token) => return Err(format!("Expected ')', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        // parse then body
        match self.tokens.next() {
            Some(Token::LeftBrace) => (),
            Some(token) => return Err(format!("Expected '{{', found {:?}", token)),
            _ => return Err("Unexpected end of input".to_string()),
        };

        let then_body = self.parse_block()?;

        let else_body = if let Some(Token::Keyword(Keyword::Else)) = self.tokens.peek() {
            self.tokens.next(); // consume the keyword (else)

            match self.tokens.next() {
                Some(Token::LeftBrace) => (),
                Some(token) => return Err(format!("Expected '{{', found {:?}", token)),
                _ => return Err("Unexpected end of input".to_string()),
            };

            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(ASTNode::IfStatement {
            condition: Box::new(condition),
            then_body,
            else_body,
        })
    }

    fn parse_variable_declaration(&mut self, var_type: Type) -> Result<ASTNode, String> {
        match self.tokens.next() {
            Some(Token::Identifier(name)) => match self.tokens.next() {
                Some(Token::Operator(Operator::AssignEquals)) => {
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
            Some(Token::Keyword(Keyword::If)) => {
                self.tokens.next();
                self.parse_if_statement()
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

    fn parse_expression_with_precedence(&mut self, prec: usize) -> Result<ASTNode, String> {
        let precedence_order = [
            // assignment operators (lowest precedence)
            vec![
                Operator::AssignEquals,
                Operator::AddAssign,
                Operator::SubAssign,
                Operator::MulAssign,
                Operator::DivAssign,
            ],
            // comparison operators (next lowest precedence)
            vec![
                Operator::Equals,
                Operator::NotEquals,
                Operator::GreaterThan,
                Operator::LessThan,
                Operator::GreaterThanOrEqual,
                Operator::LessThanOrEqual,
            ],
            // add and subtract operators
            vec![Operator::Plus, Operator::Minus],
            // multiplication and division operators (highest precedence)
            vec![Operator::Multiply, Operator::Divide],
        ];

        // highest precedence (primary expressions)
        if prec >= precedence_order.len() {
            return self.parse_primary();
        }

        let mut left = if prec == precedence_order.len() - 1 {
            self.parse_primary()?
        } else {
            self.parse_expression_with_precedence(prec + 1)?
        };

        while let Some(Token::Operator(op)) = self.tokens.peek() {
            if precedence_order[prec].contains(op) {
                let op = op.clone();
                self.tokens.next();

                let right = if prec == 0 {
                    self.parse_expression_with_precedence(prec)?
                } else {
                    self.parse_expression_with_precedence(prec + 1)?
                };

                left = ASTNode::BinaryOperation {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        self.parse_expression_with_precedence(0)
    }

    pub fn parse_program(&mut self) -> Result<ASTNode, String> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(statements))
    }
}
