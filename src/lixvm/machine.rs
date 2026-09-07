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

pub struct Machine {
    chunk: Chunk,
    env: Environment,
    functions: FunctionTable,
    stack: Vec<Value>,
    ip: usize,
    frames: Vec<Frame>,
}

impl Machine {
    pub fn new(chunk: Chunk, functions: FunctionTable) -> Self {
        Machine {
            chunk,
            env: Environment::new(),
            functions,
            stack: Vec::new(),
            ip: 0,
            frames: Vec::new(),
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
                    match obj {
                        Value::Object(_, ref fields) => match fields.get(&field) {
                            Some(v) => self.push(v.clone()),
                            None => {
                                eprintln!("error: no field '{}'", field);
                                std::process::exit(1);
                            }
                        },
                        _ => {
                            eprintln!("error: cannot get field from non-object");
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
                    let obj = self.pop();
                    let mut args = Vec::new();
                    for _ in 0..argc {
                        args.push(self.pop());
                    }
                    args.reverse();
                    let type_name = match &obj {
                        Value::Object(name, _) => name.clone(),
                        _ => {
                            eprintln!("error: cannot call method on non-object");
                            std::process::exit(1);
                        }
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
                }
                Instruction::Pop => {
                    self.pop();
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
