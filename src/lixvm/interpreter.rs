use crate::ast::expression::Expression;
use crate::ast::statement::Statement;
use crate::lixvm::environment::Environment;
use crate::lixvm::operations::arithmetic;
use crate::lixvm::operations::comparison;
use crate::lixvm::operations::logic;
use crate::lixvm::value::Value;

pub fn run(stmts: &[Statement]) {
    let mut env = Environment::new();
    exec_block(stmts, &mut env);
}

fn exec_block(stmts: &[Statement], env: &mut Environment) {
    for stmt in stmts {
        match stmt {
            Statement::Print(expr) => {
                let value = eval(expr, env);
                print!("{}", value.stringify());
            }
            Statement::PrintLn(expr) => {
                let value = eval(expr, env);
                println!("{}", value.stringify());
            }
            Statement::Let { name, expr } => {
                let value = eval(expr, env);
                env.set(name, value);
            }
            Statement::Const { name, expr } => {
                let value = eval(expr, env);
                env.set(name, value);
            }
            Statement::If {
                condition,
                then_body,
                elif_branches,
                else_body,
            } => {
                let cond = eval(condition, env);
                if logic::is_truthy(&cond) {
                    exec_block(then_body, env);
                } else {
                    let mut matched = false;
                    for branch in elif_branches {
                        let elif_cond = eval(&branch.condition, env);
                        if logic::is_truthy(&elif_cond) {
                            exec_block(&branch.body, env);
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        if let Some(else_body) = else_body {
                            exec_block(else_body, env);
                        }
                    }
                }
            }
        }
    }
}

fn eval(expr: &Expression, env: &Environment) -> Value {
    match expr {
        Expression::Int(n) => Value::Int(*n),
        Expression::Str(s) => Value::Str(s.clone()),
        Expression::Bool(b) => Value::Bool(*b),
        Expression::Ident(name) => match env.get(name) {
            Some(value) => value.clone(),
            None => {
                eprintln!("error: undefined variable '{}'", name);
                std::process::exit(1);
            }
        },
        Expression::Add(l, r) => arithmetic::add(eval(l, env), eval(r, env)),
        Expression::Sub(l, r) => arithmetic::sub(eval(l, env), eval(r, env)),
        Expression::Mul(l, r) => arithmetic::mul(eval(l, env), eval(r, env)),
        Expression::Div(l, r) => arithmetic::div(eval(l, env), eval(r, env)),
        Expression::Mod(l, r) => arithmetic::rem(eval(l, env), eval(r, env)),
        Expression::Eq(l, r) => comparison::eq(eval(l, env), eval(r, env)),
        Expression::NotEq(l, r) => comparison::not_eq(eval(l, env), eval(r, env)),
        Expression::Lt(l, r) => comparison::lt(eval(l, env), eval(r, env)),
        Expression::Gt(l, r) => comparison::gt(eval(l, env), eval(r, env)),
        Expression::LtEq(l, r) => comparison::lt_eq(eval(l, env), eval(r, env)),
        Expression::GtEq(l, r) => comparison::gt_eq(eval(l, env), eval(r, env)),
        Expression::And(l, r) => logic::and(eval(l, env), eval(r, env)),
        Expression::Or(l, r) => logic::or(eval(l, env), eval(r, env)),
        Expression::Not(e) => logic::not(eval(e, env)),
    }
}
