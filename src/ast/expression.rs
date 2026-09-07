#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    Str(String),
    Add(Box<Expression>, Box<Expression>),
}
