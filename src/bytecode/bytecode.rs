use std::collections::HashMap;

use crate::{
    lexer::{Operator, Type},
    parser::{ASTNode, Parameter},
};

#[derive(Clone, Debug)]
pub enum Instruction {
    // stack operations
    PushNumber(f64),
    PushString(String),
    PushBoolean(bool),
    PushVoid,
    Pop,

    // variables
    LoadVariable(String),
    StoreVariable(String),
    DeclareVariable(String, Type),

    // math
    Add,
    Subtract,
    Multiply,
    Divide,

    // comparison
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,

    // control flow
    Jump(usize),        // jump to ix index
    JumpIfFalse(usize), // conditional jump

    // functions
    DeclareFunction(String, Vec<Parameter>, Option<Type>),
    Call(String, usize), // function name, arg count
    Return,

    // environment
    EnterScope,
    ExitScope,

    // end of program
    End,
}

pub struct Bytecode {
    instructions: Vec<Instruction>,
    jump_points: Vec<(usize, String)>,
    labels: HashMap<String, usize>,
    label_counter: usize,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            jump_points: Vec::new(),
            labels: HashMap::new(),
            label_counter: 0,
        }
    }

    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn create_label(&mut self, name: &str) {
        self.labels
            .insert(name.to_string(), self.instructions.len());
    }

    fn add_jump(&mut self, instruction: Instruction, label: &str) {
        let pos = self.instructions.len();
        self.instructions.push(instruction);
        self.jump_points.push((pos, label.to_string()));
    }

    fn resolve_jumps(&mut self) {
        for (pos, label) in self.jump_points.clone() {
            if let Some(&target) = self.labels.get(&label) {
                match self.instructions[pos] {
                    Instruction::Jump(_) => {
                        self.instructions[pos] = Instruction::Jump(target);
                    }
                    Instruction::JumpIfFalse(_) => {
                        self.instructions[pos] = Instruction::JumpIfFalse(target);
                    }
                    _ => panic!("Non jump instruction in jump points"),
                }
            } else {
                panic!("Unresolved label: {}", label);
            }
        }
    }

    pub fn from_program(&mut self, node: ASTNode) -> Result<Vec<Instruction>, String> {
        match node {
            ASTNode::Program(statements) => {
                for stmt in statements {
                    self.compile_node(stmt)?;
                }

                self.instructions.push(Instruction::End);

                self.resolve_jumps();
            }
            _ => unreachable!("Unexpected node type, expected program"),
        }

        Ok(self.instructions.clone())
    }

    fn is_return_statement(&self, node: &ASTNode) -> bool {
        match node {
            ASTNode::ReturnStatement(_) => true,
            ASTNode::IfStatement {
                then_body,
                else_body,
                ..
            } => {
                let then_returns = !then_body.is_empty()
                    && self.is_return_statement(&then_body[then_body.len() - 1]);
                let else_returns = match else_body {
                    Some(else_body) => {
                        !else_body.is_empty()
                            && self.is_return_statement(&else_body[else_body.len() - 1])
                    }
                    None => false,
                };

                then_returns && else_returns
            }
            _ => false,
        }
    }

    fn compile_node(&mut self, node: ASTNode) -> Result<(), String> {
        match node {
            ASTNode::Statement(expr) => {
                self.compile_node(*expr)?;
                self.instructions.push(Instruction::Pop);
            }
            ASTNode::ReturnStatement(expr) => {
                self.compile_node(*expr)?;
                self.instructions.push(Instruction::Return);
            }
            ASTNode::BinaryOperation { left, op, right } => match op {
                Operator::AssignEquals => {
                    if let ASTNode::Identifier(name) = *left {
                        self.compile_node(*right)?;
                        self.instructions
                            .push(Instruction::StoreVariable(name.clone()));
                        self.instructions.push(Instruction::LoadVariable(name));
                    } else {
                        return Err("Left side of assignment must be an identifier".to_string());
                    }
                }
                _ => {
                    self.compile_node(*left)?;
                    self.compile_node(*right)?;

                    match op {
                        Operator::Plus => self.instructions.push(Instruction::Add),
                        Operator::Minus => self.instructions.push(Instruction::Subtract),
                        Operator::Multiply => self.instructions.push(Instruction::Multiply),
                        Operator::Divide => self.instructions.push(Instruction::Divide),
                        Operator::Equals => self.instructions.push(Instruction::Equals),
                        Operator::NotEquals => self.instructions.push(Instruction::NotEquals),
                        Operator::GreaterThan => self.instructions.push(Instruction::GreaterThan),
                        Operator::LessThan => self.instructions.push(Instruction::LessThan),
                        Operator::GreaterThanOrEqual => {
                            self.instructions.push(Instruction::GreaterThanOrEqual)
                        }
                        Operator::LessThanOrEqual => {
                            self.instructions.push(Instruction::LessThanOrEqual)
                        }
                        _ => unreachable!("Unexpected binary operator: {:?}", op),
                    }
                }
            },
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let function_label = format!("function_{}", name);
                let end_label = format!("{}_end", function_label);

                // declare function
                self.instructions.push(Instruction::DeclareFunction(
                    name,
                    parameters.clone(),
                    return_type,
                ));

                // jump over function body during normal execution
                self.add_jump(Instruction::Jump(0), &end_label);

                // create function label
                self.create_label(&function_label);

                // create new scope for function body
                self.instructions.push(Instruction::EnterScope);

                // check if function has an explicit return
                let has_explicit_return =
                    !body.is_empty() && self.is_return_statement(&body[body.len() - 1]);

                // compile function body
                for stmt in body {
                    self.compile_node(stmt)?;
                }

                // if no explicit return, return void
                if !has_explicit_return {
                    self.instructions.push(Instruction::PushVoid);
                    self.instructions.push(Instruction::Return);
                }

                // exit scope
                self.instructions.push(Instruction::ExitScope);

                // label for end of function
                self.create_label(&end_label);
            }
            ASTNode::FunctionCall { name, arguments } => {
                for arg in &arguments {
                    self.compile_node(arg.clone())?;
                }

                // call function with number of arguments
                self.instructions
                    .push(Instruction::Call(name, arguments.len()));
            }
            ASTNode::IfStatement {
                condition,
                then_body,
                else_body,
            } => {
                let else_label = self.generate_label("else");
                let end_label = self.generate_label("endif");

                // compile_condition
                self.compile_node(*condition)?;

                // jump to else body if condition is false
                self.add_jump(Instruction::JumpIfFalse(0), &else_label);

                // enter scope for then body
                self.instructions.push(Instruction::EnterScope);

                // compile then body
                for stmt in then_body {
                    self.compile_node(stmt)?;
                }

                // exit then scope
                self.instructions.push(Instruction::ExitScope);

                // jump to end after then block
                self.add_jump(Instruction::Jump(0), &end_label);

                // label for else body
                self.create_label(&else_label);

                // compile else body if it exists
                if let Some(else_body) = else_body {
                    // enter scope for else body
                    self.instructions.push(Instruction::EnterScope);

                    // compile else body
                    for stmt in else_body {
                        self.compile_node(stmt)?;
                    }

                    // exit else scope
                    self.instructions.push(Instruction::ExitScope);
                }

                // label for end of if statement
                self.create_label(&end_label);
            }
            ASTNode::VariableDeclaration {
                var_type,
                name,
                value,
            } => {
                self.instructions
                    .push(Instruction::DeclareVariable(name.clone(), var_type));
                self.compile_node(*value)?;
                self.instructions.push(Instruction::StoreVariable(name));
            }
            ASTNode::Identifier(name) => {
                self.instructions.push(Instruction::LoadVariable(name));
            }
            ASTNode::NumberLiteral(value) => {
                self.instructions.push(Instruction::PushNumber(value));
            }
            ASTNode::StringLiteral(value) => {
                self.instructions.push(Instruction::PushString(value));
            }
            ASTNode::BooleanLiteral(value) => {
                self.instructions.push(Instruction::PushBoolean(value));
            }
            _ => unreachable!("Unexpected node type, expected statement"),
        };

        Ok(())
    }
}
