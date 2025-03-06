use std::collections::{HashMap, VecDeque};

use crate::{bytecode::Instruction, lexer::Type, parser::Parameter};

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Void,
}

#[derive(Clone, Debug)]
struct Function {
    parameters: Vec<Parameter>,
    return_type: Option<Type>,
    address: usize,
}

#[derive(Clone, Debug)]
struct CallFrame {
    return_address: usize,
    variables: HashMap<String, Value>,
}

pub struct VM {
    instructions: Vec<Instruction>,
    pc: usize,
    stack: Vec<Value>,
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, Function>,
    call_stack: Vec<CallFrame>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            pc: 0,
            stack: Vec::new(),
            scopes: vec![HashMap::new()], // global scope !
            functions: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    fn current_scope(&mut self) -> &mut HashMap<String, Value> {
        self.scopes.last_mut().unwrap()
    }

    fn push(&mut self, value: Value) {
        println!("Pushing value: {:?}", value);
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }

    fn get_variable(&mut self, name: &str) -> Result<Value, String> {
        // search scopes from top? to bottom? (like javascript)
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        Err(format!("Variable '{}' not found", name))
    }

    pub fn run(&mut self) -> Result<Option<Value>, String> {
        self.pc = 0;

        while self.pc < self.instructions.len() {
            let ix = self.instructions[self.pc].clone();
            println!("Executing instruction: {:?}", ix);

            match ix {
                // stack oeprations
                Instruction::PushNumber(num) => {
                    self.push(Value::Number(num));
                }
                Instruction::PushString(string) => {
                    self.push(Value::String(string));
                }
                Instruction::PushBoolean(boolean) => {
                    self.push(Value::Boolean(boolean));
                }
                Instruction::PushVoid => {
                    self.push(Value::Void);
                }
                Instruction::Pop => {
                    self.pop()?;
                }

                // variable operations
                Instruction::LoadVariable(name) => {
                    let value = self.get_variable(&name)?;
                    self.push(value);
                }
                Instruction::StoreVariable(name) => {
                    let value = self.pop()?;

                    // find and update variable in scopes
                    let mut found = false;
                    for scope in self.scopes.iter_mut().rev() {
                        if scope.contains_key(&name) {
                            scope.insert(name.clone(), value.clone());
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        return Err(format!("Assignment to undeclared variable '{}'", name));
                    }

                    self.push(value);
                }
                Instruction::DeclareVariable(name, _type) => {
                    let current_scope = self.current_scope();

                    if current_scope.contains_key(&name) {
                        return Err(format!(
                            "Variable '{}' already declared in this scope",
                            name
                        ));
                    }

                    // this will be overwritten by the StoreVariable ix
                    current_scope.insert(name, Value::Void);
                }

                // math
                Instruction::Add => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left.clone(), right.clone()) {
                        self.push(Value::Number(a + b));
                    } else {
                        return Err(format!("Cannot add {:?} to {:?}", left, right));
                    }
                }
                Instruction::Subtract => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Number(a - b));
                    } else {
                        return Err("Type mismatch in subtraction".to_string());
                    }
                }
                Instruction::Multiply => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Number(a * b));
                    } else {
                        return Err("Type mismatch in multiplication".to_string());
                    }
                }
                Instruction::Divide => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        if b == 0.0 {
                            return Err("Cannot divide by zero".to_string());
                        }
                        self.push(Value::Number(a / b));
                    } else {
                        return Err("Type mismatch in division".to_string());
                    }
                }

                // comparison
                Instruction::Equals => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.push(Value::Boolean(a == b));
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::Boolean(a == b));
                        }
                        (Value::Boolean(a), Value::Boolean(b)) => {
                            self.push(Value::Boolean(a == b));
                        }
                        _ => {
                            return Err("Type mismatch in equality comparison".to_string());
                        }
                    }
                }
                Instruction::NotEquals => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.push(Value::Boolean(a != b));
                        }
                        (Value::String(a), Value::String(b)) => {
                            self.push(Value::Boolean(a != b));
                        }
                        (Value::Boolean(a), Value::Boolean(b)) => {
                            self.push(Value::Boolean(a != b));
                        }
                        _ => {
                            return Err("Type mismatch in equality comparison".to_string());
                        }
                    }
                }
                Instruction::GreaterThan => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Boolean(a > b));
                    }
                }
                Instruction::LessThan => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Boolean(a < b));
                    }
                }
                Instruction::GreaterThanOrEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Boolean(a >= b));
                    }
                }
                Instruction::LessThanOrEqual => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Boolean(a <= b));
                    }
                }

                // control flow
                Instruction::Jump(address) => {
                    self.pc = address;
                    continue;
                }
                Instruction::JumpIfFalse(address) => {
                    if let Value::Boolean(condition) = self.pop()? {
                        if !condition {
                            self.pc = address;
                            continue;
                        }
                    } else {
                        return Err("Non bool value in condition".to_string());
                    }
                }

                // functions
                Instruction::DeclareFunction(name, parameters, return_type) => {
                    let mut body_address = self.pc + 1;
                    if body_address < self.instructions.len() {
                        if let Instruction::Jump(_) = self.instructions[body_address] {
                            body_address += 1;
                        }
                    }

                    self.functions.insert(
                        name,
                        Function {
                            parameters,
                            return_type,
                            address: body_address,
                        },
                    );
                }
                Instruction::Call(name, arg_count) => {
                    let function = match self.functions.get(&name) {
                        Some(f) => f.clone(),
                        None => return Err(format!("Usage of undeclared function '{}'", name)),
                    };

                    // check arg count
                    let required_args = function.parameters.iter().filter(|p| p.optional).count();

                    if arg_count < required_args || arg_count > function.parameters.len() {
                        return Err(format!(
                            "Function '{}' requires {} arguments, but {} were provided",
                            name, required_args, arg_count
                        ));
                    }

                    // create new call frame
                    let mut cf = CallFrame {
                        return_address: self.pc + 1,
                        variables: HashMap::new(),
                    };

                    // pop arguments in reverse (last arg first)
                    let mut args = VecDeque::new();
                    for _ in 0..arg_count {
                        args.push_front(self.pop()?);
                    }

                    // bind args to function parameters
                    for (i, param) in function.parameters.iter().enumerate() {
                        if i < args.len() {
                            cf.variables.insert(param.name.clone(), args[i].clone());
                        } else {
                            // optional parameters are set to void
                            cf.variables.insert(param.name.clone(), Value::Void);
                        }
                    }

                    // save call frame
                    self.call_stack.push(cf);

                    // enter new scope for function
                    self.scopes
                        .push(self.call_stack.last().unwrap().variables.clone());

                    // jump to function body
                    self.pc = function.address;
                    continue;
                }
                Instruction::Return => {
                    let return_value = if !self.stack.is_empty() {
                        self.pop()?
                    } else {
                        Value::Void
                    };

                    // check if were in a function call frame
                    if let Some(cf) = self.call_stack.pop() {
                        // exit function scope
                        self.scopes.pop();

                        // jump back to caller
                        self.pc = cf.return_address;

                        // push return value
                        self.push(return_value);
                        continue;
                    } else {
                        return Ok(Some(return_value));
                    }
                }

                // environment
                Instruction::EnterScope => {
                    self.scopes.push(HashMap::new());
                }
                Instruction::ExitScope => {
                    self.scopes.pop();
                    if self.scopes.is_empty() {
                        self.scopes.push(HashMap::new()); // keep global scope
                    }
                }

                // end program
                Instruction::End => {
                    if !self.stack.is_empty() {
                        return Ok(Some(self.pop()?));
                    }

                    return Ok(None);
                }
            }

            self.pc += 1;
        }

        Ok(None)
    }
}
