use crate::lixvm::value::Value;

#[derive(Debug)]
pub enum VmError {
    TypeError(String),
    UndefinedVar(String),
    UndefinedFunc(String),
    IndexError(String),
    FieldError(String),
    DivisionByZero,
    StackUnderflow,
    UncaughtError(Value),
    Other(String),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::TypeError(msg) => write!(f, "type error: {}", msg),
            VmError::UndefinedVar(name) => write!(f, "undefined variable '{}'", name),
            VmError::UndefinedFunc(name) => write!(f, "undefined function '{}'", name),
            VmError::IndexError(msg) => write!(f, "index error: {}", msg),
            VmError::FieldError(msg) => write!(f, "field error: {}", msg),
            VmError::DivisionByZero => write!(f, "division by zero"),
            VmError::StackUnderflow => write!(f, "stack underflow"),
            VmError::UncaughtError(v) => write!(f, "uncaught error: {}", v.stringify()),
            VmError::Other(msg) => write!(f, "{}", msg),
        }
    }
}
