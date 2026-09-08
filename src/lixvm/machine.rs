use crate::bytecode::chunk::{Chunk, FunctionTable};
use crate::bytecode::instruction::Instruction;
use crate::lixvm::environment::Environment;
use crate::lixvm::operations::arithmetic;
use crate::lixvm::operations::comparison;
use crate::lixvm::operations::logic;
use crate::lixvm::value::Value;

struct Frame {
    return_ip: usize,
    return_env: Environment,
    return_chunk: Chunk,
}

struct CatchHandler {
    catch_ip: usize,
    return_ip: usize,
    return_env: Environment,
    return_chunk: Chunk,
}

pub struct Machine {
    chunk: Chunk,
    env: Environment,
    functions: FunctionTable,
    native: crate::lixvm::native::NativeTable,
    stack: Vec<Value>,
    ip: usize,
    frames: Vec<Frame>,
    catch_stack: Vec<CatchHandler>,
}

impl Machine {
    pub fn new(chunk: Chunk, functions: FunctionTable) -> Self {
        Machine {
            chunk,
            env: Environment::new(),
            functions,
            native: crate::lixvm::native::NativeTable::new(),
            stack: Vec::new(),
            ip: 0,
            frames: Vec::new(),
            catch_stack: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let instr = match self.chunk.get(self.ip) {
                Some(i) => i.clone(),
                None => break,
            };
            self.ip += 1;
            match instr {
                Instruction::LoadInt(n) => self.push(Value::Int(n)),
                Instruction::LoadFloat(n) => self.push(Value::Float(n)),
                Instruction::LoadNull => self.push(Value::Null),
                Instruction::LoadStr(s) => self.push(Value::Str(s)),
                Instruction::LoadBool(b) => self.push(Value::Bool(b)),
                Instruction::LoadConst(_) => {}
                Instruction::LoadVar(name) => match self.env.get(&name) {
                    Some(value) => self.push(value.clone()),
                    None => {
                        eprintln!("error: undefined variable '{}'", name);
                        std::process::exit(1);
                    }
                },
                Instruction::StoreVar(name) => {
                    let value = self.pop();
                    self.env.set(&name, value);
                }
                Instruction::Add => self.binary(arithmetic::add),
                Instruction::Sub => self.binary(arithmetic::sub),
                Instruction::Mul => self.binary(arithmetic::mul),
                Instruction::Div => self.binary(arithmetic::div),
                Instruction::Mod => self.binary(arithmetic::rem),
                Instruction::Eq => self.binary(comparison::eq),
                Instruction::NotEq => self.binary(comparison::not_eq),
                Instruction::Lt => self.binary(comparison::lt),
                Instruction::Gt => self.binary(comparison::gt),
                Instruction::LtEq => self.binary(comparison::lt_eq),
                Instruction::GtEq => self.binary(comparison::gt_eq),
                Instruction::And => self.binary(logic::and),
                Instruction::Or => self.binary(logic::or),
                Instruction::Not => {
                    let value = self.pop();
                    self.push(logic::not(value));
                }
                Instruction::Jump(target) => {
                    self.ip = target;
                }
                Instruction::JumpIfFalse(target) => {
                    let value = self.pop();
                    if !logic::is_truthy(&value) {
                        self.ip = target;
                    }
                }
                Instruction::Print => {
                    let value = self.pop();
                    print!("{}", value.stringify());
                }
                Instruction::PrintLn => {
                    let value = self.pop();
                    println!("{}", value.stringify());
                }
                Instruction::Call(name, argc) => {
                    let mut args = Vec::new();
                    for _ in 0..argc {
                        args.push(self.pop());
                    }
                    args.reverse();
                    if let Some(result) = self.native.call(&name, &args) {
                        self.push(result);
                    } else {
                        let func_def = match self.functions.get(&name) {
                            Some(f) => f.clone(),
                            None => {
                                eprintln!("error: undefined function '{}'", name);
                                std::process::exit(1);
                            }
                        };
                        let mut func_env = Environment::new();
                        for (i, param) in func_def.params.iter().enumerate() {
                            if i < args.len() {
                                func_env.set(param, args[i].clone());
                            }
                        }
                        let frame = Frame {
                            return_ip: self.ip,
                            return_env: std::mem::replace(&mut self.env, func_env),
                            return_chunk: std::mem::replace(&mut self.chunk, func_def.chunk),
                        };
                        self.frames.push(frame);
                        self.ip = 0;
                    }
                }
                Instruction::Return => {
                    let value = self.pop();
                    if let Some(frame) = self.frames.pop() {
                        self.ip = frame.return_ip;
                        self.env = frame.return_env;
                        self.chunk = frame.return_chunk;
                    }
                    self.push(value);
                }
                Instruction::Construct(type_name, field_names) => {
                    let mut fields = std::collections::HashMap::new();
                    for name in field_names.iter().rev() {
                        let value = self.pop();
                        fields.insert(name.clone(), value);
                    }
                    self.push(Value::Object(type_name.clone(), fields));
                }
                Instruction::GetField(field) => {
                    let obj = self.pop();
                    match &obj {
                        Value::Str(s) => {
                            if field == "len" {
                                self.push(Value::Int(s.chars().count() as i64));
                            } else {
                                eprintln!("error: string has no property '{}'", field);
                                std::process::exit(1);
                            }
                        }
                        Value::Bytes(b) => {
                            if field == "len" {
                                self.push(Value::Int(b.len() as i64));
                            } else {
                                eprintln!("error: bytes has no property '{}'", field);
                                std::process::exit(1);
                            }
                        }
                        Value::List(items) => {
                            if field == "len" {
                                self.push(Value::Int(items.len() as i64));
                            } else {
                                eprintln!("error: list has no property '{}'", field);
                                std::process::exit(1);
                            }
                        }
                        Value::Map(map) => {
                            if field == "len" {
                                self.push(Value::Int(map.len() as i64));
                            } else {
                                match map.get(&field) {
                                    Some(v) => self.push(v.clone()),
                                    None => {
                                        eprintln!("error: key '{}' not found in map", field);
                                        std::process::exit(1);
                                    }
                                }
                            }
                        }
                        Value::Object(_, fields) => match fields.get(&field) {
                            Some(v) => self.push(v.clone()),
                            None => {
                                eprintln!("error: no field '{}'", field);
                                std::process::exit(1);
                            }
                        },
                        _ => {
                            eprintln!("error: cannot get field from {:?}", obj);
                            std::process::exit(1);
                        }
                    }
                }
                Instruction::SetField(field) => {
                    let value = self.pop();
                    let obj = self.pop();
                    match obj {
                        Value::Object(type_name, mut fields) => {
                            fields.insert(field, value);
                            self.push(Value::Object(type_name, fields));
                        }
                        _ => {
                            eprintln!("error: cannot set field on non-object");
                            std::process::exit(1);
                        }
                    }
                }
                Instruction::MethodCall(method, argc) => {
                    let mut args = Vec::new();
                    for _ in 0..argc {
                        args.push(self.pop());
                    }
                    args.reverse();
                    let obj = self.pop();
                    if let Value::Object(_, _) = &obj {
                        let type_name = match &obj {
                            Value::Object(name, _) => name.clone(),
                            _ => unreachable!(),
                        };
                        let mangled = format!("{}.{}", type_name, method);
                        let func_def = match self.functions.get(&mangled) {
                            Some(f) => f.clone(),
                            None => {
                                eprintln!("error: undefined method '{}.{}'", type_name, method);
                                std::process::exit(1);
                            }
                        };
                        let mut func_env = Environment::new();
                        func_env.set("self", obj);
                        let params: Vec<&String> =
                            func_def.params.iter().filter(|p| **p != "self").collect();
                        for (i, param) in params.iter().enumerate() {
                            if i < args.len() {
                                func_env.set(param, args[i].clone());
                            }
                        }
                        let frame = Frame {
                            return_ip: self.ip,
                            return_env: std::mem::replace(&mut self.env, func_env),
                            return_chunk: std::mem::replace(&mut self.chunk, func_def.chunk),
                        };
                        self.frames.push(frame);
                        self.ip = 0;
                    } else {
                        let result = native_method(&obj, &method, &args);
                        self.push(result);
                    }
                }
                Instruction::LoadBytes(b) => {
                    self.push(Value::Bytes(b.clone()));
                }
                Instruction::NewList(count) => {
                    let mut items = Vec::new();
                    for _ in 0..count {
                        items.push(self.pop());
                    }
                    items.reverse();
                    self.push(Value::List(items));
                }
                Instruction::IndexGet => {
                    let index = self.pop();
                    let obj = self.pop();
                    match (obj, index) {
                        (Value::List(items), Value::Int(i)) => {
                            if i < 0 || i as usize >= items.len() {
                                eprintln!("error: index {} out of bounds", i);
                                std::process::exit(1);
                            }
                            self.push(items[i as usize].clone());
                        }
                        (Value::Str(s), Value::Int(i)) => {
                            let chars: Vec<char> = s.chars().collect();
                            if i < 0 || i as usize >= chars.len() {
                                eprintln!("error: index {} out of bounds", i);
                                std::process::exit(1);
                            }
                            self.push(Value::Str(chars[i as usize].to_string()));
                        }
                        (Value::Bytes(b), Value::Int(i)) => {
                            if i < 0 || i as usize >= b.len() {
                                eprintln!("error: index {} out of bounds", i);
                                std::process::exit(1);
                            }
                            self.push(Value::Int(b[i as usize] as i64));
                        }
                        (Value::Map(map), Value::Str(key)) => match map.get(&key) {
                            Some(v) => self.push(v.clone()),
                            None => {
                                eprintln!("error: key '{}' not found", key);
                                std::process::exit(1);
                            }
                        },
                        (obj, idx) => {
                            eprintln!("error: cannot index {:?} with {:?}", obj, idx);
                            std::process::exit(1);
                        }
                    }
                }
                Instruction::IndexSet => {
                    let value = self.pop();
                    let index = self.pop();
                    let obj = self.pop();
                    match (obj, index) {
                        (Value::List(mut items), Value::Int(i)) => {
                            if i < 0 || i as usize >= items.len() {
                                eprintln!("error: index {} out of bounds", i);
                                std::process::exit(1);
                            }
                            items[i as usize] = value;
                            self.push(Value::List(items));
                        }
                        (Value::Map(mut map), Value::Str(key)) => {
                            map.insert(key, value);
                            self.push(Value::Map(map));
                        }
                        (obj, idx) => {
                            eprintln!("error: cannot index set {:?} with {:?}", obj, idx);
                            std::process::exit(1);
                        }
                    }
                }
                Instruction::NewMap(count) => {
                    let mut map = std::collections::HashMap::new();
                    for _ in 0..count {
                        let value = self.pop();
                        let key = self.pop();
                        if let Value::Str(k) = key {
                            map.insert(k, value);
                        }
                    }
                    self.push(Value::Map(map));
                }
                Instruction::Stringify => {
                    let value = self.pop();
                    self.push(Value::Str(value.stringify()));
                }
                Instruction::Pop => {
                    self.pop();
                }
                Instruction::Deinit => {
                    // TODO: call deinit on objects going out of scope
                    // requires GC or scope tracking, deferred to phase 1
                }
                Instruction::Throw => {
                    let err = self.pop();
                    if let Some(handler) = self.catch_stack.pop() {
                        self.ip = handler.catch_ip;
                        self.env = handler.return_env;
                        self.chunk = handler.return_chunk;
                        self.frames.truncate(self.frames.len());
                        self.push(err);
                    } else {
                        eprintln!("uncaught error: {}", err.stringify());
                        std::process::exit(1);
                    }
                }
                Instruction::TryCatch(catch_ip, end_ip) => {
                    let handler = CatchHandler {
                        catch_ip,
                        return_ip: self.ip,
                        return_env: self.env.clone(),
                        return_chunk: self.chunk.clone(),
                    };
                    self.catch_stack.push(handler);
                    let _ = end_ip;
                }
                Instruction::Halt => break,
            }
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Bool(false))
    }

    fn binary<F>(&mut self, op: F)
    where
        F: FnOnce(Value, Value) -> Value,
    {
        let r = self.pop();
        let l = self.pop();
        self.push(op(l, r));
    }
}

fn native_method(obj: &Value, method: &str, args: &[Value]) -> Value {
    match obj {
        Value::Str(s) => string_method(s, method, args),
        Value::Bytes(b) => bytes_method(b, method, args),
        Value::List(items) => list_method(items, method, args),
        Value::Map(map) => map_method(map, method, args),
        _ => {
            eprintln!("error: no method '{}' on {:?}", method, obj);
            std::process::exit(1);
        }
    }
}

fn string_method(s: &str, method: &str, args: &[Value]) -> Value {
    match method {
        "upper" => Value::Str(s.to_uppercase()),
        "lower" => Value::Str(s.to_lowercase()),
        "trim" => Value::Str(s.trim().to_string()),
        "contains" => {
            if let Some(Value::Str(sub)) = args.first() {
                Value::Bool(s.contains(sub))
            } else {
                type_error("contains", args.first())
            }
        }
        "find" => {
            if let Some(Value::Str(sub)) = args.first() {
                Value::Int(s.find(sub).map(|i| i as i64).unwrap_or(-1))
            } else {
                type_error("find", args.first())
            }
        }
        "replace" => {
            if args.len() >= 2 {
                if let (Value::Str(old), Value::Str(new)) = (&args[0], &args[1]) {
                    Value::Str(s.replace(old, new))
                } else {
                    type_error("replace", args.first())
                }
            } else {
                Value::Str(s.to_string())
            }
        }
        "starts_with" => {
            if let Some(Value::Str(prefix)) = args.first() {
                Value::Bool(s.starts_with(prefix))
            } else {
                type_error("starts_with", args.first())
            }
        }
        "ends_with" => {
            if let Some(Value::Str(suffix)) = args.first() {
                Value::Bool(s.ends_with(suffix))
            } else {
                type_error("ends_with", args.first())
            }
        }
        "split" => {
            if let Some(Value::Str(sep)) = args.first() {
                Value::List(s.split(sep).map(|p| Value::Str(p.to_string())).collect())
            } else {
                type_error("split", args.first())
            }
        }
        "slice" => {
            if args.len() >= 2 {
                if let (Value::Int(start), Value::Int(end)) = (&args[0], &args[1]) {
                    let chars: Vec<char> = s.chars().collect();
                    let start = (*start as usize).min(chars.len());
                    let end = (*end as usize).min(chars.len());
                    Value::Str(chars[start..end].iter().collect())
                } else {
                    type_error("slice", args.first())
                }
            } else {
                Value::Str(s.to_string())
            }
        }
        _ => {
            eprintln!("error: undefined string method '{}'", method);
            std::process::exit(1);
        }
    }
}

fn bytes_method(b: &[u8], method: &str, args: &[Value]) -> Value {
    match method {
        "slice" => {
            if args.len() >= 2 {
                if let (Value::Int(start), Value::Int(end)) = (&args[0], &args[1]) {
                    let start = (*start as usize).min(b.len());
                    let end = (*end as usize).min(b.len());
                    Value::Bytes(b[start..end].to_vec())
                } else {
                    type_error("slice", args.first())
                }
            } else {
                Value::Bytes(b.to_vec())
            }
        }
        "to_string" => Value::Str(String::from_utf8_lossy(b).to_string()),
        _ => {
            eprintln!("error: undefined bytes method '{}'", method);
            std::process::exit(1);
        }
    }
}

fn list_method(items: &[Value], method: &str, args: &[Value]) -> Value {
    match method {
        "push" => {
            let mut new_items = items.to_vec();
            if let Some(item) = args.first() {
                new_items.push(item.clone());
            }
            Value::List(new_items)
        }
        "concat" => {
            let mut new_items = items.to_vec();
            if let Some(Value::List(other)) = args.first() {
                new_items.extend(other.iter().cloned());
            }
            Value::List(new_items)
        }
        "slice" => {
            if args.len() >= 2 {
                if let (Value::Int(start), Value::Int(end)) = (&args[0], &args[1]) {
                    let start = (*start as usize).min(items.len());
                    let end = (*end as usize).min(items.len());
                    Value::List(items[start..end].to_vec())
                } else {
                    type_error("slice", args.first())
                }
            } else {
                Value::List(items.to_vec())
            }
        }
        "contains" => {
            if let Some(item) = args.first() {
                Value::Bool(items.contains(item))
            } else {
                Value::Bool(false)
            }
        }
        "index_of" => {
            if let Some(item) = args.first() {
                Value::Int(
                    items
                        .iter()
                        .position(|v| v == item)
                        .map(|i| i as i64)
                        .unwrap_or(-1),
                )
            } else {
                Value::Int(-1)
            }
        }
        "reverse" => {
            let mut new_items = items.to_vec();
            new_items.reverse();
            Value::List(new_items)
        }
        "join" => {
            if let Some(Value::Str(sep)) = args.first() {
                let parts: Vec<String> = items.iter().map(|v| v.stringify()).collect();
                Value::Str(parts.join(sep))
            } else {
                let parts: Vec<String> = items.iter().map(|v| v.stringify()).collect();
                Value::Str(parts.join(""))
            }
        }
        _ => {
            eprintln!("error: undefined list method '{}'", method);
            std::process::exit(1);
        }
    }
}

fn map_method(
    map: &std::collections::HashMap<String, Value>,
    method: &str,
    args: &[Value],
) -> Value {
    match method {
        "contains" => {
            if let Some(Value::Str(key)) = args.first() {
                Value::Bool(map.contains_key(key))
            } else {
                Value::Bool(false)
            }
        }
        "keys" => Value::List(map.keys().cloned().map(Value::Str).collect()),
        "values" => Value::List(map.values().cloned().collect()),
        "get" => {
            if let Some(Value::Str(key)) = args.first() {
                match map.get(key) {
                    Some(v) => v.clone(),
                    None => Value::Bool(false),
                }
            } else {
                Value::Bool(false)
            }
        }
        "remove" => {
            let mut new_map = map.clone();
            if let Some(Value::Str(key)) = args.first() {
                new_map.remove(key);
            }
            Value::Map(new_map)
        }
        _ => {
            eprintln!("error: undefined map method '{}'", method);
            std::process::exit(1);
        }
    }
}

fn type_error(method: &str, arg: Option<&Value>) -> Value {
    eprintln!("type error: '{}' expects str, got {:?}", method, arg);
    std::process::exit(1);
}
