#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Object(String, std::collections::HashMap<String, Value>),
}

fn format_float(n: f64) -> String {
    if n == n.trunc() {
        format!("{}.0", n as i64)
    } else {
        format!("{}", n)
    }
}

impl Value {
    pub fn stringify(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(n) => format_float(*n),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Bytes(b) => {
                let parts: Vec<String> = b.iter().map(|byte| byte.to_string()).collect();
                format!("b[{}]", parts.join(", "))
            }
            Value::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.stringify()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Object(type_name, fields) => {
                let pairs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.stringify()))
                    .collect();
                format!("{}({})", type_name, pairs.join(", "))
            }
            Value::Map(fields) => {
                let pairs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.stringify()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }
}
