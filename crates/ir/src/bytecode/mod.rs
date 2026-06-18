use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub constants: Vec<Const>,
    pub instructions: Vec<Instr>,
    pub registers: u16,
    pub params: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Const {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function {
        chunk: Box<Chunk>,
        captures: Vec<Capture>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capture {
    Local(u16),
    Upvalue(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Op {
    Halt,
    LoadK,
    Move,
    GetGlobal,
    SetGlobal,
    NewTable,
    GetTable,
    SetTable,
    Add,
    AddK,
    AddModK,
    AddSelectLt,
    Sub,
    SubK,
    Mul,
    MulK,
    MulKAddModK,
    Div,
    DivK,
    FloorDiv,
    FloorDivK,
    Mod,
    ModK,
    Pow,
    PowK,
    Eq,
    Lt,
    Le,
    And,
    Or,
    BitAnd,
    BitXor,
    BitOr,
    Shl,
    Shr,
    Concat,
    Not,
    Neg,
    Len,
    BitNot,
    Cell,
    GetCell,
    SetCell,
    GetUp,
    SetUp,
    Jmp,
    JmpFalse,
    JmpNotEq,
    JmpNotLt,
    JmpNotLe,
    ForCheck,
    ForCheckPos,
    ForStep,
    ForStepPos,
    ForStepAddPos,
    Call,
    CallGlobal,
    TailCallGlobal,
    TailCallGlobalR,
    TailCallGlobalRR,
    TailCallGlobalK,
    TailCallGlobalKK,
    TailCallGlobalKR,
    CallN,
    Call3,
    ReturnCall,
    SetTableCall,
    GenericFor,
    GenericFor2Jmp,
    SuperBlock,
    Return,
    ReturnVarArg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instr {
    pub op: Op,
    pub a: u16,
    pub b: u16,
    pub c: u16,
}

impl Instr {
    pub fn new(op: Op, a: u16, b: u16, c: u16) -> Self {
        Self { op, a, b, c }
    }
}
