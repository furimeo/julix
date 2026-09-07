use crate::ast::expression::Expression;

#[derive(Debug, Clone)]
pub enum Statement {
    Print(Expression),
    PrintLn(Expression),
}
