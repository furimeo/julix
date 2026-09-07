#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Object(String, std::collections::HashMap<String, Value>),
}

impl Value {
    pub fn stringify(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Str(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Object(type_name, fields) => {
                let pairs: Vec<String> = fields.iter().map(|(k, v)| format!("{}: {}", k, v.stringify())).collect();
                format!("{}({})", type_name, pairs.join(", "))
            }
        }
    }
}
