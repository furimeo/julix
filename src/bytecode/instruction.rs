#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // literals
    LoadInt(i64),
    LoadNull,
    LoadBool(bool),
    LoadConst(u16),

    // variables
    LoadSlot(u16),
    StoreSlot(u16),

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
    Call(u32, usize),
    Return,

    // objects
    Construct(String, Vec<String>),
    GetField(String),
    SetField(String),
    MethodCall(u32, usize),
    Deinit,

    // error handling
    Throw,
    TryCatch(usize, usize),

    // literals (in constant pool)
    NewList(usize),
    NewMap(usize),
    IndexGet,
    IndexSet,
    Stringify,

    // misc
    Pop,
    Halt,
}
