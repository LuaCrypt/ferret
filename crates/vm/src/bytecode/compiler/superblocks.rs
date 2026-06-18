use std::collections::{BTreeMap, BTreeSet};

use ferret_ir::{Chunk, Const, Instr, Op};
use ferret_util::Result;

use crate::bytecode::support::to_u16;

mod tail_calls;
#[cfg(test)]
mod tests;

use tail_calls::peephole_tail_calls;

const SUPERBLOCK_MIN_LEN: usize = 3;
const SUPERBLOCK_MAX_LEN: usize = 16;

pub(super) fn apply(chunk: &mut Chunk) -> Result<()> {
    for constant in &mut chunk.constants {
        if let Const::Function { chunk, .. } = constant {
            apply(chunk)?;
        }
    }
    apply_chunk(chunk)
}

fn apply_chunk(chunk: &mut Chunk) -> Result<()> {
    peephole_tail_calls(chunk)?;
    let spans = superblock_spans(&chunk.instructions);
    if spans.is_empty() {
        return Ok(());
    }

    let original = std::mem::take(&mut chunk.instructions);
    let mut span_starts = BTreeMap::new();
    for (start, len) in spans {
        span_starts.insert(start, len);
    }

    let mut old_to_new = vec![0usize; original.len() + 1];
    let mut rewritten = Vec::with_capacity(original.len() + span_starts.len());
    let mut index = 0usize;
    while index < original.len() {
        if let Some(len) = span_starts.get(&index).copied() {
            let header = rewritten.len();
            old_to_new[index] = header;
            rewritten.push(Instr::new(Op::SuperBlock, to_u16(len)?, 0, 0));
            for offset in 0..len {
                if offset > 0 {
                    old_to_new[index + offset] = rewritten.len();
                }
                rewritten.push(original[index + offset].clone());
            }
            index += len;
        } else {
            old_to_new[index] = rewritten.len();
            rewritten.push(original[index].clone());
            index += 1;
        }
    }
    old_to_new[original.len()] = rewritten.len();
    remap_branch_targets(&mut rewritten, &old_to_new)?;
    chunk.instructions = rewritten;
    Ok(())
}

fn superblock_spans(instructions: &[Instr]) -> Vec<(usize, usize)> {
    let targets = branch_targets(instructions);
    let mut spans = Vec::new();
    let mut index = 0usize;
    while index < instructions.len() {
        if !superblock_body_eligible(&instructions[index])
            && !superblock_terminal(&instructions[index])
        {
            index += 1;
            continue;
        }

        let start = index;
        let mut len = 0usize;
        while start + len < instructions.len() && len < SUPERBLOCK_MAX_LEN {
            if len > 0 && targets.contains(&(start + len)) {
                break;
            }
            if !superblock_body_eligible(&instructions[start + len]) {
                break;
            }
            len += 1;
        }

        if start + len < instructions.len()
            && len < SUPERBLOCK_MAX_LEN
            && (len == 0 || !targets.contains(&(start + len)))
            && superblock_terminal(&instructions[start + len])
        {
            len += 1;
        }

        let has_terminal = instructions[start..start + len]
            .iter()
            .any(superblock_terminal);
        if len >= SUPERBLOCK_MIN_LEN || (has_terminal && len > 1) {
            spans.push((start, len));
            index = start + len;
        } else {
            index += 1;
        }
    }
    spans
}

fn superblock_body_eligible(instr: &Instr) -> bool {
    match instr.op {
        Op::LoadK
        | Op::Move
        | Op::GetGlobal
        | Op::NewTable
        | Op::GetTable
        | Op::SetTable
        | Op::Add
        | Op::AddK
        | Op::AddModK
        | Op::AddSelectLt
        | Op::Sub
        | Op::SubK
        | Op::Mul
        | Op::MulK
        | Op::MulKAddModK
        | Op::Div
        | Op::DivK
        | Op::FloorDiv
        | Op::FloorDivK
        | Op::Mod
        | Op::ModK
        | Op::Pow
        | Op::PowK
        | Op::Eq
        | Op::Lt
        | Op::Le
        | Op::And
        | Op::Or
        | Op::BitAnd
        | Op::BitXor
        | Op::BitOr
        | Op::Shl
        | Op::Shr
        | Op::Concat
        | Op::Not
        | Op::Neg
        | Op::Len
        | Op::BitNot
        | Op::Cell
        | Op::GetCell
        | Op::SetCell
        | Op::GetUp
        | Op::SetUp => true,
        Op::Call => (instr.c & 0xff) <= 4,
        Op::CallGlobal => instr.c <= 4,
        Op::Halt
        | Op::SetGlobal
        | Op::Jmp
        | Op::JmpFalse
        | Op::JmpNotEq
        | Op::JmpNotLt
        | Op::JmpNotLe
        | Op::ForCheck
        | Op::ForCheckPos
        | Op::ForStep
        | Op::ForStepPos
        | Op::ForStepAddPos
        | Op::CallN
        | Op::Call3
        | Op::ReturnCall
        | Op::SetTableCall
        | Op::GenericFor
        | Op::GenericFor2Jmp
        | Op::TailCallGlobal
        | Op::TailCallGlobalR
        | Op::TailCallGlobalRR
        | Op::TailCallGlobalK
        | Op::TailCallGlobalKK
        | Op::TailCallGlobalKR
        | Op::SuperBlock
        | Op::Return
        | Op::ReturnVarArg => false,
    }
}

fn superblock_terminal(instr: &Instr) -> bool {
    matches!(
        instr.op,
        Op::Halt | Op::Return | Op::ReturnVarArg | Op::ForStepPos
    )
}

fn branch_targets(instructions: &[Instr]) -> BTreeSet<usize> {
    let mut targets = BTreeSet::new();
    for instr in instructions {
        if let Some(target) = branch_target(instr) {
            targets.insert(target);
        }
    }
    targets
}

fn branch_target(instr: &Instr) -> Option<usize> {
    match instr.op {
        Op::Jmp => Some(usize::from(instr.a)),
        Op::JmpFalse | Op::ForCheck | Op::ForCheckPos => Some(usize::from(instr.b)),
        Op::JmpNotEq | Op::JmpNotLt | Op::JmpNotLe | Op::ForStepAddPos | Op::GenericFor2Jmp => {
            Some(usize::from(instr.c))
        }
        Op::ForStep | Op::ForStepPos => Some(usize::from(instr.b)),
        _ => None,
    }
}

fn remap_branch_targets(instructions: &mut [Instr], old_to_new: &[usize]) -> Result<()> {
    for instr in instructions {
        match instr.op {
            Op::Jmp => instr.a = remap_target(instr.a, old_to_new)?,
            Op::JmpFalse | Op::ForCheck | Op::ForCheckPos => {
                instr.b = remap_target(instr.b, old_to_new)?;
            }
            Op::JmpNotEq | Op::JmpNotLt | Op::JmpNotLe | Op::ForStepAddPos | Op::GenericFor2Jmp => {
                instr.c = remap_target(instr.c, old_to_new)?;
            }
            Op::ForStep | Op::ForStepPos => instr.b = remap_target(instr.b, old_to_new)?,
            _ => {}
        }
    }
    Ok(())
}

fn remap_target(target: u16, old_to_new: &[usize]) -> Result<u16> {
    let target = usize::from(target);
    to_u16(*old_to_new.get(target).unwrap_or(&target))
}
