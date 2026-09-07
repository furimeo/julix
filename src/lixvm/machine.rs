use crate::bytecode::chunk::Chunk;
use crate::bytecode::instruction::Instruction;
use crate::lixvm::environment::Environment;
use crate::lixvm::operations::arithmetic;
use crate::lixvm::operations::comparison;
use crate::lixvm::operations::logic;
use crate::lixvm::value::Value;

pub struct Machine {
    chunk: Chunk,
    env: Environment,
    stack: Vec<Value>,
    ip: usize,
}

impl Machine {
    pub fn new(chunk: Chunk) -> Self {
        Machine {
            chunk,
            env: Environment::new(),
            stack: Vec::new(),
            ip: 0,
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
