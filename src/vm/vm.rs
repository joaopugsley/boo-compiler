use std::collections::{HashMap, VecDeque};

use crate::{
    bytecode::Instruction,
    parser::Parameter,
    stdlib::stdlib::{register_stdlib, NativeFn},
};

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
    address: usize,
}

#[derive(Clone, Debug)]
struct CallFrame {
    return_address: usize,
    variables: HashMap<String, Value>,
    scope_index: usize,
}

pub struct VM {
    debug: bool,
    instructions: Vec<Instruction>,
    pc: usize,
    stack: Vec<Value>,
    scopes: Vec<HashMap<String, Value>>,
    call_stack: Vec<CallFrame>,
    functions: HashMap<String, Function>,
    native_functions: HashMap<String, NativeFn>,
    string_methods: HashMap<String, NativeFn>,
    number_methods: HashMap<String, NativeFn>,
    boolean_methods: HashMap<String, NativeFn>,
}

impl VM {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        let mut vm = Self {
            debug: false,
            instructions,
            pc: 0,
            stack: Vec::new(),
            scopes: vec![HashMap::new()], // global scope !
            call_stack: Vec::new(),
            functions: HashMap::new(),

            // stdlib
            native_functions: HashMap::new(),
            string_methods: HashMap::new(),
            number_methods: HashMap::new(),
            boolean_methods: HashMap::new(),
        };

        register_stdlib(&mut vm);

        vm
    }

    pub fn register_native_function(&mut self, name: &str, fun: NativeFn) {
        self.native_functions.insert(name.to_string(), fun);
    }

    pub fn register_string_method(&mut self, name: &str, fun: NativeFn) {
        self.string_methods.insert(name.to_string(), fun);
    }

    pub fn register_number_method(&mut self, name: &str, fun: NativeFn) {
        self.number_methods.insert(name.to_string(), fun);
    }

    pub fn register_boolean_method(&mut self, name: &str, fun: NativeFn) {
        self.boolean_methods.insert(name.to_string(), fun);
    }

    fn debug_print(&self, message: String) {
        if self.debug {
            println!("{}", message);
        }
    }

    #[inline]
    fn current_scope(&mut self) -> &mut HashMap<String, Value> {
        self.scopes.last_mut().unwrap()
    }

    #[inline]
    fn push(&mut self, value: Value) {
        self.debug_print(format!("Pushing value: {:?}", value));
        self.stack.push(value);
    }

    #[inline]
    fn pop(&mut self) -> Result<Value, String> {
        self.stack
            .pop()
            .ok_or_else(|| "Stack underflow".to_string())
    }

    #[inline]
    fn get_variable(&mut self, name: &str) -> Result<Value, String> {
        // fast path -> check the topmost scope first (most common case)
        if let Some(scope) = self.scopes.last() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // search other scopes from top? to bottom? (like javascript)
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
            self.debug_print(format!("Executing instruction: {:?}", ix));

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
                Instruction::Power => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        self.push(Value::Number(a.powf(b)));
                    }
                }
                Instruction::Modulo => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    if let (Value::Number(a), Value::Number(b)) = (left, right) {
                        if b == 0.0 {
                            return Err("Cannot calculate modulo by zero".to_string());
                        }
                        self.push(Value::Number(a % b));
                    }
                }

                // string operations
                Instruction::Concat => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    match (left, right) {
                        (Value::Void, _) | (_, Value::Void) => {
                            return Err("Cannot concatenate void".to_string());
                        }
                        (Value::String(mut a), Value::String(b)) => {
                            a.reserve(b.len());
                            a.push_str(&b);
                            self.push(Value::String(a));
                        }
                        (Value::String(mut a), b) => {
                            let b_str = match b {
                                Value::String(s) => s,
                                Value::Boolean(b) => b.to_string(),
                                Value::Number(n) => n.to_string(),
                                _ => {
                                    return Err(format!("Cannot concatenate {:?} to string", b));
                                }
                            };
                            a.push_str(&b_str);
                            self.push(Value::String(a));
                        }
                        (a, Value::String(b)) => {
                            let a_str = match a {
                                Value::String(s) => s,
                                Value::Boolean(b) => b.to_string(),
                                Value::Number(n) => n.to_string(),
                                _ => {
                                    return Err(format!("Cannot concatenate {:?} to string", a));
                                }
                            };
                            let mut result = a_str;
                            result.push_str(&b);
                            self.push(Value::String(result));
                        }
                        _ => {
                            return Err("Type mismatch in concatenation".to_string());
                        }
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
                Instruction::DeclareFunction(name, parameters, _return_type) => {
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
                            address: body_address,
                        },
                    );
                }
                Instruction::Call(name, arg_count) => {
                    // check for native functions
                    if self.native_functions.contains_key(&name) {
                        let native_fn = self.native_functions.get(&name).unwrap().clone();

                        let mut args = Vec::with_capacity(arg_count);
                        for _ in 0..arg_count {
                            let value = self.pop()?;
                            args.insert(0, value);
                        }

                        // call the native function
                        let result = native_fn(self, args)?;
                        self.push(result);
                        self.pc += 1;
                        continue;
                    }

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

                    // create a new scope for the function
                    self.scopes.push(HashMap::new());
                    let scope_index = self.scopes.len() - 1;

                    // create new call frame
                    let mut cf = CallFrame {
                        return_address: self.pc + 1,
                        variables: HashMap::new(),
                        scope_index,
                    };

                    // pop arguments in reverse (last arg first)
                    let mut args = VecDeque::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push_front(self.pop()?);
                    }

                    // bind args to function parameters
                    for (i, param) in function.parameters.iter().enumerate() {
                        if i < args.len() {
                            self.scopes[scope_index].insert(param.name.clone(), args[i].clone());
                            cf.variables.insert(param.name.clone(), args[i].clone());
                        } else {
                            // optional parameters are set to void
                            self.scopes[scope_index].insert(param.name.clone(), Value::Void);
                            cf.variables.insert(param.name.clone(), Value::Void);
                        }
                    }

                    // save call frame
                    self.call_stack.push(cf);

                    // jump to function body
                    self.pc = function.address;
                    continue;
                }
                Instruction::CallMethod(name, arg_count) => {
                    // collect arguments
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        let value = self.pop()?;
                        args.insert(0, value);
                    }

                    // get the object
                    let object = self.pop()?;

                    // add the object as the first argument for our native method handler
                    let mut full_args = vec![object.clone()];
                    full_args.extend(args);

                    match object {
                        Value::String(_) => {
                            if self.string_methods.contains_key(&name) {
                                let native_fn = self.string_methods.get(&name).unwrap().clone();
                                let result = native_fn(self, full_args)?;
                                self.push(result);
                                self.pc += 1;
                                continue;
                            }
                        }
                        Value::Number(_) => {
                            if self.number_methods.contains_key(&name) {
                                let native_fn = self.number_methods.get(&name).unwrap().clone();
                                let result = native_fn(self, full_args)?;
                                self.push(result);
                                self.pc += 1;
                                continue;
                            }
                        }
                        Value::Boolean(_) => {
                            if self.boolean_methods.contains_key(&name) {
                                let native_fn = self.boolean_methods.get(&name).unwrap().clone();
                                let result = native_fn(self, full_args)?;
                                self.push(result);
                                self.pc += 1;
                                continue;
                            }
                        }
                        _ => {
                            return Err(format!("Cannot call method '{}' on {:?}", name, object));
                        }
                    }
                }
                Instruction::Return => {
                    let return_value = if !self.stack.is_empty() {
                        self.pop()?
                    } else {
                        Value::Void
                    };

                    // check if were in a function call frame
                    if let Some(cf) = self.call_stack.pop() {
                        // make sure we pop exactly the scope associated with this call frame
                        while self.scopes.len() > cf.scope_index {
                            self.scopes.pop();
                        }

                        // jump back to caller
                        self.pc = cf.return_address;

                        // push return value
                        self.push(return_value);

                        // continue execution
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
