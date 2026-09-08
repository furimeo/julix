use crate::lixvm::value::Value;

use std::collections::HashMap;

pub struct NativeTable {
    funcs: HashMap<String, NativeFn>,
}

pub type NativeFn = fn(&[Value]) -> Value;

impl NativeTable {
    pub fn new() -> Self {
        let mut table = NativeTable {
            funcs: HashMap::new(),
        };
        table.register_all();
        table
    }

    pub fn get(&self, name: &str) -> Option<&NativeFn> {
        self.funcs.get(name)
    }

    pub fn call(&self, name: &str, args: &[Value]) -> Option<Value> {
        self.funcs.get(name).map(|f| f(args))
    }

    fn register_all(&mut self) {
        self.funcs
            .insert("println".to_string(), native_println as NativeFn);
        self.funcs
            .insert("print".to_string(), native_print as NativeFn);
        self.funcs.insert("len".to_string(), native_len as NativeFn);
        self.funcs
            .insert("clock".to_string(), native_clock as NativeFn);
        self.funcs
            .insert("exit".to_string(), native_exit as NativeFn);
        self.funcs.insert("env".to_string(), native_env as NativeFn);
        self.funcs
            .insert("args".to_string(), native_args as NativeFn);
    }
}

fn native_println(args: &[Value]) -> Value {
    if let Some(v) = args.first() {
        println!("{}", v.stringify());
    } else {
        println!();
    }
    Value::Bool(true)
}

fn native_print(args: &[Value]) -> Value {
    if let Some(v) = args.first() {
        print!("{}", v.stringify());
    }
    Value::Bool(true)
}

fn native_len(args: &[Value]) -> Value {
    match args.first() {
        Some(Value::Str(s)) => Value::Int(s.chars().count() as i64),
        Some(Value::Bytes(b)) => Value::Int(b.len() as i64),
        Some(Value::List(items)) => Value::Int(items.len() as i64),
        Some(Value::Map(m)) => Value::Int(m.len() as i64),
        _ => Value::Int(0),
    }
}

fn native_clock(_args: &[Value]) -> Value {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Value::Int(secs as i64)
}

fn native_exit(args: &[Value]) -> Value {
    let code = match args.first() {
        Some(Value::Int(c)) => *c as i32,
        _ => 0,
    };
    std::process::exit(code);
}

fn native_env(_args: &[Value]) -> Value {
    let mut map = HashMap::new();
    for (k, v) in std::env::vars() {
        map.insert(k, Value::Str(v));
    }
    Value::Map(map)
}

fn native_args(_args: &[Value]) -> Value {
    let args: Vec<Value> = std::env::args().skip(1).map(Value::Str).collect();
    Value::List(args)
}
