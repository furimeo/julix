use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
}

impl Value {
    fn stringify(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Str(s) => s.clone(),
        }
    }
}

pub fn run(stmts: &[Statement]) {
    let mut env: HashMap<String, Value> = HashMap::new();

    for stmt in stmts {
        match stmt {
            Statement::Print(expr) => {
                let value = eval(expr, &env);
                print!("{}", value.stringify());
            }
            Statement::PrintLn(expr) => {
                let value = eval(expr, &env);
                println!("{}", value.stringify());
            }
            Statement::Let { name, expr } => {
                let value = eval(expr, &env);
                env.insert(name.clone(), value);
            }
            Statement::Const { name, expr } => {
                let value = eval(expr, &env);
                env.insert(name.clone(), value);
            }
        }
    }
}

fn eval(expr: &Expression, env: &HashMap<String, Value>) -> Value {
    match expr {
        Expression::Int(n) => Value::Int(*n),
        Expression::Str(s) => Value::Str(s.clone()),
        Expression::Ident(name) => match env.get(name) {
            Some(value) => value.clone(),
            None => {
                eprintln!("error: undefined variable '{}'", name);
                std::process::exit(1);
            }
        },
        Expression::Add(left, right) => {
            let l = eval(left, env);
            let r = eval(right, env);
            add(l, r)
        }
        Expression::Sub(left, right) => {
            let l = eval(left, env);
            let r = eval(right, env);
            sub(l, r)
        }
        Expression::Mul(left, right) => {
            let l = eval(left, env);
            let r = eval(right, env);
            mul(l, r)
        }
        Expression::Div(left, right) => {
            let l = eval(left, env);
            let r = eval(right, env);
            div(l, r)
        }
        Expression::Mod(left, right) => {
            let l = eval(left, env);
            let r = eval(right, env);
            rem(l, r)
        }
    }
}

fn add(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
        (a, b) => type_error("add", a, b),
    }
}

fn sub(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (a, b) => type_error("sub", a, b),
    }
}

fn mul(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (a, b) => type_error("mul", a, b),
    }
}

fn div(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                eprintln!("error: division by zero");
                std::process::exit(1);
            }
            Value::Int(a / b)
        }
        (a, b) => type_error("div", a, b),
    }
}

fn rem(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                eprintln!("error: modulo by zero");
                std::process::exit(1);
            }
            Value::Int(a % b)
        }
        (a, b) => type_error("mod", a, b),
    }
}

fn type_error(op: &str, a: Value, b: Value) -> ! {
    eprintln!("type error: cannot {} {:?} and {:?}", op, a, b);
    std::process::exit(1);
}
