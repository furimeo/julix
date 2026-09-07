use crate::lixvm::value::Value;

pub fn add(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Str(a), Value::Str(b)) => Value::Str(a + &b),
        (a, b) => type_error("add", a, b),
    }
}

pub fn sub(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (a, b) => type_error("sub", a, b),
    }
}

pub fn mul(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (a, b) => type_error("mul", a, b),
    }
}

pub fn div(l: Value, r: Value) -> Value {
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

pub fn rem(l: Value, r: Value) -> Value {
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
