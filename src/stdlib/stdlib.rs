use crate::{
    analyzer::TypeChecker,
    lexer::Type,
    vm::{Value, VM},
};

pub type NativeFn = fn(&mut VM, Vec<Value>) -> Result<Value, String>;

pub fn print(_vm: &mut VM, args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        println!();
        return Ok(Value::Void);
    }

    for arg in args {
        match arg {
            Value::Number(num) => println!("{}", num),
            Value::String(s) => println!("{}", s),
            Value::Boolean(b) => println!("{}", b),
            Value::Void => println!("void"),
        }
    }

    Ok(Value::Void)
}

pub fn string_len(_vm: &mut VM, args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("method: len() requires exactly one argument".to_string());
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.len() as f64)),
        _ => Err("method: len() argument must be a string".to_string()),
    }
}

pub fn to_string(_vm: &mut VM, args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("method: to_string() requires exactly one argument".to_string());
    }

    match &args[0] {
        Value::String(s) => Ok(Value::String(s.clone())),
        Value::Number(num) => Ok(Value::String(num.to_string())),
        Value::Boolean(b) => Ok(Value::String(b.to_string())),
        _ => Err("Cannot convert to string".to_string()),
    }
}

pub fn register_stdlib(vm: &mut VM) {
    // register native functions
    vm.register_native_function("print", print);

    // register string methods
    vm.register_string_method("len", string_len);
    vm.register_string_method("to_string", to_string);

    // register number methods
    vm.register_number_method("to_string", to_string);

    // register boolean methods
    vm.register_boolean_method("to_string", to_string);
}

pub fn register_stdlib_types(checker: &mut TypeChecker) {
    // register native functions
    checker.register_native_function_type("print", Type::Void);

    // register string methods
    checker.register_string_method_type("len", Type::Num);
    checker.register_string_method_type("to_string", Type::Str);

    // register number methods
    checker.register_number_method_type("to_string", Type::Str);

    // register boolean methods
    checker.register_boolean_method_type("to_string", Type::Str);
}
