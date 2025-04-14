use std::collections::HashMap;

use crate::{
    lexer::{Operator, Type},
    parser::{ASTNode, Parameter},
    stdlib::stdlib::register_stdlib_types,
};

pub struct FunctionSignature {
    parameters: Vec<Parameter>,
    return_type: Option<Type>,
    is_native: bool,
}

pub struct TypeChecker {
    program: ASTNode,
    variables: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionSignature>,

    // stdlib
    native_function_types: HashMap<String, Type>,
    string_method_types: HashMap<String, Type>,
    number_method_types: HashMap<String, Type>,
    boolean_method_types: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new(program: ASTNode) -> Self {
        let mut checker = Self {
            program,
            variables: Vec::new(),
            functions: HashMap::new(),

            // stdlib
            native_function_types: HashMap::new(),
            string_method_types: HashMap::new(),
            number_method_types: HashMap::new(),
            boolean_method_types: HashMap::new(),
        };

        // register stdlib
        register_stdlib_types(&mut checker);

        checker
    }

    pub fn register_native_function_type(&mut self, name: &str, return_type: Type) {
        self.native_function_types
            .insert(name.to_string(), return_type);
    }

    pub fn register_string_method_type(&mut self, name: &str, return_type: Type) {
        self.string_method_types
            .insert(name.to_string(), return_type);
    }

    pub fn register_number_method_type(&mut self, name: &str, return_type: Type) {
        self.number_method_types
            .insert(name.to_string(), return_type);
    }

    pub fn register_boolean_method_type(&mut self, name: &str, return_type: Type) {
        self.boolean_method_types
            .insert(name.to_string(), return_type);
    }

    fn enter_scope(&mut self) {
        if self.variables.is_empty() {
            self.variables.push(HashMap::new());
        }

        // add the new scope
        self.variables.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.variables.pop();
    }

    fn get_current_scope(&mut self) -> &mut HashMap<String, Type> {
        if self.variables.is_empty() {
            self.variables.push(HashMap::new());
        }
        self.variables.last_mut().unwrap()
    }

    pub fn check_program(&mut self) -> Result<(), String> {
        let program = self.program.clone();
        match program {
            ASTNode::Program(nodes) => Ok(for node in nodes {
                self.check_node(node)?;
            }),
            _ => panic!("Unexpected node type, expected program"),
        }
    }

    fn check_node(&mut self, node: ASTNode) -> Result<Type, String> {
        match node {
            ASTNode::Statement(expr) => self.check_node(*expr),
            ASTNode::ReturnStatement(expr) => self.check_node(*expr),
            ASTNode::BinaryOperation { left, op, right } => {
                self.check_binary_operation(*left, op, *right)
            }
            ASTNode::UnaryOperation { op, operand } => {
                let operand_type = self.check_node(*operand)?;
                match op {
                    Operator::UnaryMinus => {
                        if operand_type != Type::Num {
                            return Err(format!(
                                "Type mismatch: expected 'Num', found '{:?}'",
                                operand_type
                            ));
                        }
                        Ok(Type::Num)
                    }
                    Operator::LogicalNot => {
                        if operand_type != Type::Bool {
                            return Err(format!(
                                "Type mismatch: expected 'Bool', found '{:?}'",
                                operand_type
                            ));
                        }
                        Ok(Type::Bool)
                    }
                    _ => Err(format!("Unsupported unary operator: {:?}", op)),
                }
            }
            ASTNode::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => self.check_function_declaration(name, parameters, return_type, body),
            ASTNode::FunctionCall { name, arguments } => self.check_function_call(name, arguments),
            ASTNode::MethodCall {
                object,
                method,
                arguments,
            } => self.check_method_call(*object, method, arguments),
            ASTNode::IfStatement {
                condition,
                then_body,
                else_body,
            } => self.check_if_statement(*condition, then_body, else_body),
            ASTNode::VariableDeclaration {
                var_type,
                name,
                value,
            } => self.check_variable_declaration(var_type, name, *value),
            ASTNode::Identifier(name) => self.check_identifier(name),
            ASTNode::BooleanLiteral(_) => Ok(Type::Bool),
            ASTNode::NumberLiteral(_) => Ok(Type::Num),
            ASTNode::StringLiteral(_) => Ok(Type::Str),
            _ => unimplemented!("Unimplemented node type"),
        }
    }

    fn check_if_statement(
        &mut self,
        condition: ASTNode,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    ) -> Result<Type, String> {
        let condition_type = self.check_node(condition)?;

        if condition_type != Type::Bool {
            return Err(format!(
                "Type mismatch: expected 'Bool', found '{:?}'",
                condition_type
            ));
        }

        for node in then_body {
            self.check_node(node)?;
        }

        if let Some(else_body) = else_body {
            for node in else_body {
                self.check_node(node)?;
            }
        }

        Ok(Type::Void)
    }

    fn check_binary_operation(
        &mut self,
        left: ASTNode,
        op: Operator,
        right: ASTNode,
    ) -> Result<Type, String> {
        if let ASTNode::Identifier(name) = &left {
            self.verify_optional_parameter_usage(&name)?;
        }

        if let ASTNode::Identifier(name) = &right {
            self.verify_optional_parameter_usage(&name)?;
        }

        let left_type = self.check_node(left)?;

        let right_type = match op {
            Operator::LogicalAnd | Operator::LogicalOr => {
                if left_type != Type::Bool {
                    return Err(format!(
                        "Type mismatch: expected 'Bool', found '{:?}'",
                        left_type
                    ));
                }
                self.check_node(right)?
            }
            _ => self.check_node(right)?,
        };

        match op {
            Operator::Plus
            | Operator::Minus
            | Operator::Multiply
            | Operator::Divide
            | Operator::Power
            | Operator::Modulo => {
                if left_type != Type::Num {
                    return Err(format!(
                        "Type mismatch: expected 'Num', found '{:?}'",
                        left_type
                    ));
                }

                if right_type != Type::Num {
                    return Err(format!(
                        "Type mismatch: expected 'Num', found '{:?}'",
                        right_type
                    ));
                }

                Ok(Type::Num)
            }
            Operator::LogicalAnd | Operator::LogicalOr => {
                if right_type != Type::Bool {
                    return Err(format!(
                        "Type mismatch: expected 'Bool', found '{:?}'",
                        right_type
                    ));
                }

                Ok(Type::Bool)
            }
            Operator::Concat => {
                if left_type == Type::Void || right_type == Type::Void {
                    return Err("Cannot concatenate void".to_string());
                }

                Ok(Type::Str)
            }
            Operator::Equals | Operator::NotEquals => {
                if left_type != right_type {
                    return Err(format!(
                        "Type mismatch: expected '{:?}', found '{:?}'",
                        left_type, right_type
                    ));
                }

                Ok(Type::Bool)
            }
            Operator::GreaterThan
            | Operator::LessThan
            | Operator::GreaterThanOrEqual
            | Operator::LessThanOrEqual
            | Operator::AddAssign
            | Operator::DivAssign
            | Operator::MulAssign
            | Operator::SubAssign
            | Operator::ModAssign
            | Operator::PowAssign => {
                if left_type != Type::Num || right_type != Type::Num {
                    return Err(format!(
                        "Type mismatch: expected 'Num' and 'Num', found '{:?}' and '{:?}'",
                        left_type, right_type
                    ));
                }

                Ok(Type::Bool)
            }
            Operator::AssignEquals => {
                if left_type != right_type {
                    return Err(format!(
                        "Type mismatch: expected '{:?}', found '{:?}'",
                        left_type, right_type
                    ));
                }

                Ok(Type::Void)
            }
            Operator::UnaryMinus | Operator::LogicalNot => {
                unreachable!("{:?} is not a binary operator", op)
            }
        }
    }

    fn verify_optional_parameter_usage(&self, name: &str) -> Result<(), String> {
        for signature in self.functions.values() {
            if let Some(param) = signature
                .parameters
                .iter()
                .find(|p| p.name == name && p.optional)
            {
                return Err(format!(
                    "Warning: Operation uses optional parameter '{}' without null check",
                    param.name
                ));
            }
        }
        Ok(())
    }

    fn check_variable_declaration(
        &mut self,
        var_type: Type,
        name: String,
        value: ASTNode,
    ) -> Result<Type, String> {
        let value_type = self.check_node(value)?;

        if value_type != var_type {
            return Err(format!(
                "Type mismatch: expected '{:?}', found '{:?}'",
                var_type, value_type
            ));
        }

        // get the current scope
        let current_scope = self.get_current_scope();

        // check if the variable is already declared in the current scope
        if current_scope.contains_key(&name) {
            return Err(format!(
                "Variable '{}' already declared in this scope",
                name
            ));
        }

        current_scope.insert(name, var_type);
        Ok(Type::Void)
    }

    fn check_identifier(&mut self, name: String) -> Result<Type, String> {
        for scope in self.variables.iter().rev() {
            if let Some(var_type) = scope.get(&name) {
                return Ok(*var_type);
            }
        }
        Err(format!("Unknown identifier '{}'", name))
    }

    fn check_function_declaration(
        &mut self,
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<ASTNode>,
    ) -> Result<Type, String> {
        let param_types: Vec<(String, Type)> = parameters
            .iter()
            .map(|p| (p.name.clone(), p.param_type.clone()))
            .collect();

        self.functions.insert(
            name.to_string(),
            FunctionSignature {
                parameters,
                return_type,
                is_native: false,
            },
        );

        // enter a new scope for function body
        self.enter_scope();

        // add parameters to the current scope
        for (param_name, param_type) in param_types {
            self.get_current_scope().insert(param_name, param_type);
        }

        // check function body
        let mut last_type = Type::Void;
        for stmt in body {
            last_type = self.check_node(stmt)?;
        }

        // verify return type matches declaration
        if let Some(expected_return_type) = return_type {
            if last_type != expected_return_type {
                return Err(format!(
                    "Function '{}' return type mismatch, expected type '{:?}', got '{:?}'",
                    name, expected_return_type, last_type
                ));
            }
        }

        // exit the scope
        self.exit_scope();
        Ok(Type::Void)
    }

    fn check_function_call(
        &mut self,
        name: String,
        arguments: Vec<ASTNode>,
    ) -> Result<Type, String> {
        // first check for native functions
        if self.native_function_types.contains_key(&name) {
            for arg in &arguments {
                let arg_type = self.check_node(arg.clone())?;
                if arg_type == Type::Void {
                    return Err(format!(
                        "Native function '{}' requires a non-void argument",
                        name
                    ));
                }
            }

            return match self.native_function_types.get(&name) {
                Some(return_type) => Ok(*return_type),
                None => Ok(Type::Void),
            };
        }

        let signature = match self.functions.get(&name) {
            Some(signature) => FunctionSignature {
                parameters: signature.parameters.clone(),
                return_type: signature.return_type.clone(),
                is_native: signature.is_native,
            },
            _ => return Err(format!("Unknown function '{}'", name)),
        };

        // check argument count (and for optional arguments)
        let required_parameters_count = signature
            .parameters
            .iter()
            .filter(|p| p.optional == false)
            .count();

        if arguments.len() < required_parameters_count {
            return Err(format!(
                "Function '{}' expects at least {} arguments, got {}",
                name,
                required_parameters_count,
                arguments.len()
            ));
        }

        // check argument types
        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.check_node(arg.clone())?;
            let param_type = &signature.parameters[i].param_type;
            if arg_type != *param_type {
                return Err(format!(
                    "Argument '{}' of function '{}' has type mismatch: expected type '{:?}', got '{:?}'",
                    &signature.parameters[i].name, name, param_type, arg_type
                ));
            }
        }

        Ok(signature.return_type.clone().unwrap_or(Type::Void))
    }

    fn check_method_call(
        &mut self,
        object: ASTNode,
        method_name: String,
        arguments: Vec<ASTNode>,
    ) -> Result<Type, String> {
        let object_type = self.check_node(object)?;

        // check if the method exists for this type
        let method_exists = match object_type {
            Type::Str => self.string_method_types.contains_key(&method_name),
            Type::Num => self.number_method_types.contains_key(&method_name),
            Type::Bool => self.boolean_method_types.contains_key(&method_name),
            _ => false,
        };

        if !method_exists {
            return Err(format!(
                "Method '{}' does not exist for type '{:?}'",
                method_name, object_type
            ));
        }

        // check arguments
        for arg in arguments {
            let arg_type = self.check_node(arg.clone())?;
            if arg_type == Type::Void {
                return Err(format!(
                    "Method '{}' requires a non-void argument",
                    method_name
                ));
            }
        }

        match object_type {
            Type::Str => match self.string_method_types.get(&method_name) {
                Some(return_type) => Ok(*return_type),
                None => Ok(Type::Void),
            },
            Type::Num => match self.number_method_types.get(&method_name) {
                Some(return_type) => Ok(*return_type),
                None => Ok(Type::Void),
            },
            Type::Bool => match self.boolean_method_types.get(&method_name) {
                Some(return_type) => Ok(*return_type),
                None => Ok(Type::Void),
            },
            _ => Ok(Type::Void),
        }
    }
}
