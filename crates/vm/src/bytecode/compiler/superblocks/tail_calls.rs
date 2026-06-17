use std::collections::BTreeSet;

use ferret_ir::{Chunk, Instr, Op};
use ferret_util::Result;

use super::{branch_targets, remap_branch_targets};

pub(super) fn peephole_tail_calls(chunk: &mut Chunk) -> Result<()> {
    let original = std::mem::take(&mut chunk.instructions);
    let targets = branch_targets(&original);
    let mut old_to_new = vec![0usize; original.len() + 1];
    let mut rewritten = Vec::with_capacity(original.len());
    let mut index = 0usize;
    while index < original.len() {
        if let Some((len, instr)) = tail_call_fusion(&original, index, &targets) {
            let new_index = rewritten.len();
            for offset in 0..len {
                old_to_new[index + offset] = new_index;
            }
            rewritten.push(instr);
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

fn tail_call_fusion(
    instructions: &[Instr],
    start: usize,
    targets: &BTreeSet<usize>,
) -> Option<(usize, Instr)> {
    let max_len = 4usize.min(instructions.len().saturating_sub(start));
    for len in (2..=max_len).rev() {
        if (1..len).any(|offset| targets.contains(&(start + offset))) {
            continue;
        }
        let call_index = start + len - 2;
        let halt_index = start + len - 1;
        if instructions.get(halt_index)?.op != Op::Halt {
            continue;
        }
        let call = instructions.get(call_index)?;
        if call.op != Op::CallGlobal || call.c > 2 {
            continue;
        }
        let args = &instructions[start..call_index];
        if args.len() != usize::from(call.c) {
            continue;
        }
        return tail_call_instr(call, args).map(|instr| (len, instr));
    }
    None
}

fn tail_call_instr(call: &Instr, args: &[Instr]) -> Option<Instr> {
    Some(match args {
        [] => Instr::new(Op::TailCallGlobal, call.a, call.b, call.c),
        [arg] if call.c == 1 && arg.a == call.b => match arg.op {
            Op::Move => Instr::new(Op::TailCallGlobalR, call.a, arg.b, 0),
            Op::LoadK => Instr::new(Op::TailCallGlobalK, call.a, arg.b, 0),
            _ => return None,
        },
        [first, second] if call.c == 2 && first.a == call.b && second.a == call.b + 1 => {
            match (first.op, second.op) {
                (Op::Move, Op::Move) => Instr::new(Op::TailCallGlobalRR, call.a, first.b, second.b),
                (Op::LoadK, Op::LoadK) => {
                    Instr::new(Op::TailCallGlobalKK, call.a, first.b, second.b)
                }
                (Op::LoadK, Op::Move) => {
                    Instr::new(Op::TailCallGlobalKR, call.a, first.b, second.b)
                }
                _ => return None,
            }
        }
        _ => return None,
    })
}
