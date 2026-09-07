use crate::lixer::ast::expression::Expression;

#[derive(Debug, Clone)]
pub enum Statement {
    Print(Expression),
    PrintLn(Expression),
    Let {
        name: String,
        expr: Expression,
    },
    Const {
        name: String,
        expr: Expression,
    },
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        elif_branches: Vec<ElifBranch>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        var: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct ElifBranch {
    pub condition: Expression,
    pub body: Vec<Statement>,
}
