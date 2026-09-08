use crate::lixvm::operations::arithmetic::type_error;
use crate::lixvm::value::Value;

pub fn eq(l: Value, r: Value) -> Value {
    Value::Bool(l == r)
}

pub fn not_eq(l: Value, r: Value) -> Value {
    Value::Bool(l != r)
}

pub fn lt(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a < b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a < b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((a as f64) < b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(a < (b as f64)),
        (a, b) => type_error("lt", a, b),
    }
}

pub fn gt(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a > b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a > b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((a as f64) > b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(a > (b as f64)),
        (a, b) => type_error("gt", a, b),
    }
}

pub fn lt_eq(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a <= b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a <= b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((a as f64) <= b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(a <= (b as f64)),
        (a, b) => type_error("le", a, b),
    }
}

pub fn gt_eq(l: Value, r: Value) -> Value {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => Value::Bool(a >= b),
        (Value::Float(a), Value::Float(b)) => Value::Bool(a >= b),
        (Value::Int(a), Value::Float(b)) => Value::Bool((a as f64) >= b),
        (Value::Float(a), Value::Int(b)) => Value::Bool(a >= (b as f64)),
        (a, b) => type_error("ge", a, b),
    }
}
