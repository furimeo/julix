#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // literals
    LoadInt(i64),
    LoadFloat(f64),
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

    // error handling
    Throw,
    TryCatch(usize, usize),

    // literals
    LoadBytes(Vec<u8>),
    NewList(usize),
    NewMap(usize),
    IndexGet,
    IndexSet,
    Stringify,

    // misc
    Pop,
    Halt,
}
