#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Str(String),
}

impl Value {
    pub fn stringify(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Str(s) => s.clone(),
        }
    }
}
