use std::collections::BTreeMap;

use ferret_crypto::Prng;
use ferret_ir::Op;

pub fn opcode_layout(seed: u64) -> BTreeMap<Op, u32> {
    let ops = [
        Op::Halt,
        Op::LoadK,
        Op::Move,
        Op::GetGlobal,
        Op::SetGlobal,
        Op::NewTable,
        Op::GetTable,
        Op::SetTable,
        Op::Add,
        Op::AddK,
        Op::AddModK,
        Op::AddSelectLt,
        Op::Sub,
        Op::SubK,
        Op::Mul,
        Op::MulK,
        Op::MulKAddModK,
        Op::Div,
        Op::DivK,
        Op::FloorDiv,
        Op::FloorDivK,
        Op::Mod,
        Op::ModK,
        Op::Pow,
        Op::PowK,
        Op::Eq,
        Op::Lt,
        Op::Le,
        Op::And,
        Op::Or,
        Op::BitAnd,
        Op::BitXor,
        Op::BitOr,
        Op::Shl,
        Op::Shr,
        Op::Concat,
        Op::Not,
        Op::Neg,
        Op::Len,
        Op::BitNot,
        Op::Cell,
        Op::GetCell,
        Op::SetCell,
        Op::GetUp,
        Op::SetUp,
        Op::Jmp,
        Op::JmpFalse,
        Op::JmpNotEq,
        Op::JmpNotLt,
        Op::JmpNotLe,
        Op::ForCheck,
        Op::ForCheckPos,
        Op::ForStep,
        Op::ForStepPos,
        Op::ForStepAddPos,
        Op::Call,
        Op::CallGlobal,
        Op::TailCallGlobal,
        Op::TailCallGlobalR,
        Op::TailCallGlobalRR,
        Op::TailCallGlobalK,
        Op::TailCallGlobalKK,
        Op::TailCallGlobalKR,
        Op::CallN,
        Op::Call3,
        Op::GenericFor,
        Op::GenericFor2Jmp,
        Op::SuperBlock,
        Op::Return,
        Op::ReturnVarArg,
    ];
    let mut rng = Prng::new(seed ^ 0x0f0f_b17e);
    let mut used = Vec::new();
    let mut map = BTreeMap::new();
    for op in ops {
        let code = unique_code(&mut rng, &used);
        used.push(code);
        map.insert(op, code);
    }
    map
}

fn unique_code(rng: &mut Prng, used: &[u32]) -> u32 {
    loop {
        let value = (rng.next_u32() & 0x7fff_ffff) | 1;
        if !used.contains(&value) {
            return value;
        }
    }
}
