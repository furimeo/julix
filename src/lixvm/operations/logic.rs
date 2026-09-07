use crate::lixvm::operations::arithmetic::type_error;
use crate::lixvm::value::Value;

pub fn and(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a && b),
        (a, b) => type_error("and", a, b),
    }
}

pub fn or(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a || b),
        (a, b) => type_error("or", a, b),
    }
}

pub fn not(v: Value) -> Value {
    match v {
        Value::Bool(b) => Value::Bool(!b),
        a => type_error("not", a, Value::Bool(false)),
    }
}

pub fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        _ => {
            eprintln!("error: expected bool, got {:?}", v);
            std::process::exit(1);
        }
    }
}
