use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::lixvm::environment::Environment;
use crate::lixvm::operations;
use crate::lixvm::value::Value;

pub fn run(stmts: &[Statement]) {
    let mut env = Environment::new();

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
                env.set(name, value);
            }
            Statement::Const { name, expr } => {
                let value = eval(expr, &env);
                env.set(name, value);
            }
        }
    }
}

fn eval(expr: &Expression, env: &Environment) -> Value {
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
        Expression::Add(left, right) => operations::add(eval(left, env), eval(right, env)),
        Expression::Sub(left, right) => operations::sub(eval(left, env), eval(right, env)),
        Expression::Mul(left, right) => operations::mul(eval(left, env), eval(right, env)),
        Expression::Div(left, right) => operations::div(eval(left, env), eval(right, env)),
        Expression::Mod(left, right) => operations::rem(eval(left, env), eval(right, env)),
    }
}
