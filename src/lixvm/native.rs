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
        self.funcs
            .insert("read_file".to_string(), native_read_file as NativeFn);
        self.funcs
            .insert("write_file".to_string(), native_write_file as NativeFn);
        self.funcs
            .insert("file_exists".to_string(), native_file_exists as NativeFn);
        self.funcs
            .insert("bytes_new".to_string(), native_bytes_new as NativeFn);
        self.funcs
            .insert("bytes_set".to_string(), native_bytes_set as NativeFn);
        self.funcs
            .insert("bytes_get".to_string(), native_bytes_get as NativeFn);
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

fn native_read_file(args: &[Value]) -> Value {
    if let Some(Value::Str(path)) = args.first() {
        match std::fs::read_to_string(path) {
            Ok(content) => Value::Str(content),
            Err(_) => Value::Null,
        }
    } else {
        Value::Null
    }
}

fn native_write_file(args: &[Value]) -> Value {
    if args.len() >= 2 {
        if let Value::Str(path) = &args[0] {
            let data: Vec<u8> = match &args[1] {
                Value::Str(s) => s.as_bytes().to_vec(),
                Value::Bytes(b) => b.clone(),
                v => v.stringify().into_bytes(),
            };
            match std::fs::write(path, data) {
                Ok(_) => Value::Bool(true),
                Err(_) => Value::Bool(false),
            }
        } else {
            Value::Bool(false)
        }
    } else {
        Value::Bool(false)
    }
}

fn native_file_exists(args: &[Value]) -> Value {
    if let Some(Value::Str(path)) = args.first() {
        Value::Bool(std::path::Path::new(path).exists())
    } else {
        Value::Bool(false)
    }
}

fn native_bytes_new(args: &[Value]) -> Value {
    let size = match args.first() {
        Some(Value::Int(n)) => *n as usize,
        _ => 0,
    };
    Value::Bytes(vec![0u8; size])
}

fn native_bytes_set(args: &[Value]) -> Value {
    if args.len() >= 3
        && let Value::Bytes(b) = &args[0]
    {
        let mut b = b.clone();
        if let (Value::Int(i), Value::Int(v)) = (&args[1], &args[2]) {
            let idx = *i as usize;
            if idx < b.len() {
                b[idx] = *v as u8;
                return Value::Bytes(b);
            }
        }
    }
    args.first().cloned().unwrap_or(Value::Null)
}

fn native_bytes_get(args: &[Value]) -> Value {
    if args.len() >= 2
        && let (Value::Bytes(b), Value::Int(i)) = (&args[0], &args[1])
    {
        let idx = *i as usize;
        if idx < b.len() {
            return Value::Int(b[idx] as i64);
        }
    }
    Value::Null
}
