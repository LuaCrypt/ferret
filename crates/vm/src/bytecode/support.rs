use ferret_ir::{BinOp, Op};
use ferret_util::{FerretError, Result};

pub(super) fn bin_op(op: BinOp) -> Result<Op> {
    Ok(match op {
        BinOp::Add => Op::Add,
        BinOp::Sub => Op::Sub,
        BinOp::Mul => Op::Mul,
        BinOp::Div => Op::Div,
        BinOp::FloorDiv => Op::FloorDiv,
        BinOp::Mod => Op::Mod,
        BinOp::Pow => Op::Pow,
        BinOp::Eq => Op::Eq,
        BinOp::Lt => Op::Lt,
        BinOp::Le => Op::Le,
        BinOp::And => Op::And,
        BinOp::Or => Op::Or,
        BinOp::BitAnd => Op::BitAnd,
        BinOp::BitXor => Op::BitXor,
        BinOp::BitOr => Op::BitOr,
        BinOp::Shl => Op::Shl,
        BinOp::Shr => Op::Shr,
        BinOp::Concat => Op::Concat,
        BinOp::Ne | BinOp::Gt | BinOp::Ge => {
            return Err(FerretError::Compile(
                "internal comparison lowering error".to_string(),
            ))
        }
    })
}

pub(super) fn to_u16(value: usize) -> Result<u16> {
    u16::try_from(value).map_err(|_| FerretError::Compile("program exceeds VM limits".to_string()))
}
