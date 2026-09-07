#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // literals
    LoadInt(i64),
    LoadStr(String),
    LoadBool(bool),
    LoadConst(usize),

    // variables
    LoadVar(String),
    StoreVar(String),

    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // logic
    And,
    Or,
    Not,

    // control flow
    Jump(usize),
    JumpIfFalse(usize),

    // io
    Print,
    PrintLn,

    // functions
    Call(String, usize),
    Return,

    // objects
    Construct(String, Vec<String>),
    GetField(String),
    SetField(String),
    MethodCall(String, usize),
    Deinit,

    // misc
    Pop,
    Halt,
}
