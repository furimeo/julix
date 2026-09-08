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
    Assign {
        name: String,
        expr: Expression,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    Return(Option<Expression>),
    Expr(Expression),
    TypeDef {
        name: String,
        fields: Vec<(String, String)>,
        methods: Vec<FunctionDef>,
    },
    FieldAssign {
        object: Expression,
        field: String,
        expr: Expression,
    },
    IndexAssign {
        object: Expression,
        index: Expression,
        expr: Expression,
    },
    Try {
        body: Vec<Statement>,
        catch_var: Option<String>,
        catch_body: Vec<Statement>,
    },
    Throw(Expression),
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ElifBranch {
    pub condition: Expression,
    pub body: Vec<Statement>,
}
