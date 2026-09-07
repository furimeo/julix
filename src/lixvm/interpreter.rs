use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

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
    for stmt in stmts {
        match stmt {
            Statement::Print(expr) => {
                let value = eval(expr);
                print!("{}", value.stringify());
            }
            Statement::PrintLn(expr) => {
                let value = eval(expr);
                println!("{}", value.stringify());
            }
        }
    }
}

fn eval(expr: &Expression) -> Value {
    match expr {
        Expression::Int(n) => Value::Int(*n),
        Expression::Str(s) => Value::Str(s.clone()),
        Expression::Add(left, right) => {
            let l = eval(left);
            let r = eval(right);
            match (l, r) {
                (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
                (a, b) => {
                    eprintln!("type error: cannot add {:?} and {:?}", a, b);
                    std::process::exit(1);
                }
            }
        }
    }
}
