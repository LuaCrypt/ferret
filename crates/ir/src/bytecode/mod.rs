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

pub const OPCODE_DEFS: &[(Op, &str)] = &[
    (Op::Halt, "OP_HALT"),
    (Op::LoadK, "OP_LOADK"),
    (Op::Move, "OP_MOVE"),
    (Op::GetGlobal, "OP_GETGLOBAL"),
    (Op::SetGlobal, "OP_SETGLOBAL"),
    (Op::NewTable, "OP_NEWTABLE"),
    (Op::GetTable, "OP_GETTABLE"),
    (Op::SetTable, "OP_SETTABLE"),
    (Op::Add, "OP_ADD"),
    (Op::AddK, "OP_ADDK"),
    (Op::AddModK, "OP_ADDMODK"),
    (Op::AddSelectLt, "OP_ADDSELECTLT"),
    (Op::Sub, "OP_SUB"),
    (Op::SubK, "OP_SUBK"),
    (Op::Mul, "OP_MUL"),
    (Op::MulK, "OP_MULK"),
    (Op::MulKAddModK, "OP_MULKADDMODK"),
    (Op::Div, "OP_DIV"),
    (Op::DivK, "OP_DIVK"),
    (Op::FloorDiv, "OP_FLOORDIV"),
    (Op::FloorDivK, "OP_FLOORDIVK"),
    (Op::Mod, "OP_MOD"),
    (Op::ModK, "OP_MODK"),
    (Op::Pow, "OP_POW"),
    (Op::PowK, "OP_POWK"),
    (Op::Eq, "OP_EQ"),
    (Op::Lt, "OP_LT"),
    (Op::Le, "OP_LE"),
    (Op::And, "OP_AND"),
    (Op::Or, "OP_OR"),
    (Op::BitAnd, "OP_BITAND"),
    (Op::BitXor, "OP_BITXOR"),
    (Op::BitOr, "OP_BITOR"),
    (Op::Shl, "OP_SHL"),
    (Op::Shr, "OP_SHR"),
    (Op::Concat, "OP_CONCAT"),
    (Op::Not, "OP_NOT"),
    (Op::Neg, "OP_NEG"),
    (Op::Len, "OP_LEN"),
    (Op::BitNot, "OP_BITNOT"),
    (Op::Cell, "OP_CELL"),
    (Op::GetCell, "OP_GETCELL"),
    (Op::SetCell, "OP_SETCELL"),
    (Op::GetUp, "OP_GETUP"),
    (Op::SetUp, "OP_SETUP"),
    (Op::Jmp, "OP_JMP"),
    (Op::JmpFalse, "OP_JMPFALSE"),
    (Op::JmpNotEq, "OP_JMPNOTEQ"),
    (Op::JmpNotLt, "OP_JMPNOTLT"),
    (Op::JmpNotLe, "OP_JMPNOTLE"),
    (Op::ForCheck, "OP_FORCHECK"),
    (Op::ForCheckPos, "OP_FORCHECKPOS"),
    (Op::ForStep, "OP_FORSTEP"),
    (Op::ForStepPos, "OP_FORSTEPPOS"),
    (Op::ForStepAddPos, "OP_FORSTEPADDPOS"),
    (Op::Call, "OP_CALL"),
    (Op::CallGlobal, "OP_CALLGLOBAL"),
    (Op::TailCallGlobal, "OP_TAILCALLGLOBAL"),
    (Op::TailCallGlobalR, "OP_TAILCALLGLOBALR"),
    (Op::TailCallGlobalRR, "OP_TAILCALLGLOBALRR"),
    (Op::TailCallGlobalK, "OP_TAILCALLGLOBALK"),
    (Op::TailCallGlobalKK, "OP_TAILCALLGLOBALKK"),
    (Op::TailCallGlobalKR, "OP_TAILCALLGLOBALKR"),
    (Op::CallN, "OP_CALLN"),
    (Op::Call3, "OP_CALL3"),
    (Op::ReturnCall, "OP_RETURNCALL"),
    (Op::SetTableCall, "OP_SETTABLECALL"),
    (Op::GenericFor, "OP_GENERICFOR"),
    (Op::GenericFor2Jmp, "OP_GENERICFOR2JMP"),
    (Op::SuperBlock, "OP_SUPERBLOCK"),
    (Op::Return, "OP_RETURN"),
    (Op::ReturnVarArg, "OP_RETURNVARARG"),
];

impl Op {
    pub fn token(self) -> &'static str {
        OPCODE_DEFS
            .iter()
            .find_map(|(op, token)| (*op == self).then_some(*token))
            .expect("opcode token registered")
    }
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
