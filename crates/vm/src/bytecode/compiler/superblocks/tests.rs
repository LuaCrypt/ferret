use ferret_ir::Capture;

use super::*;

#[test]
fn superblock_wraps_terminal_global_call_tail() {
    let mut chunk = Chunk {
        constants: Vec::new(),
        instructions: vec![
            Instr::new(Op::Move, 1, 2, 0),
            Instr::new(Op::Add, 1, 1, 2),
            Instr::new(Op::Halt, 0, 0, 0),
        ],
        registers: 2,
        params: 0,
    };

    apply(&mut chunk).unwrap();

    assert_eq!(chunk.instructions[0], Instr::new(Op::SuperBlock, 3, 0, 0));
    assert_eq!(chunk.instructions[1].op, Op::Move);
    assert_eq!(chunk.instructions[2].op, Op::Add);
    assert_eq!(chunk.instructions[3].op, Op::Halt);
}

#[test]
fn superblock_remaps_jump_targets_after_inserted_headers() {
    let mut chunk = Chunk {
        constants: Vec::new(),
        instructions: vec![
            Instr::new(Op::LoadK, 1, 0, 0),
            Instr::new(Op::Move, 2, 1, 0),
            Instr::new(Op::Move, 3, 2, 0),
            Instr::new(Op::Jmp, 4, 0, 0),
            Instr::new(Op::LoadK, 4, 1, 0),
            Instr::new(Op::Move, 5, 4, 0),
            Instr::new(Op::Move, 6, 5, 0),
            Instr::new(Op::Halt, 0, 0, 0),
        ],
        registers: 6,
        params: 0,
    };

    apply(&mut chunk).unwrap();

    assert_eq!(chunk.instructions[0], Instr::new(Op::SuperBlock, 3, 0, 0));
    assert_eq!(chunk.instructions[4], Instr::new(Op::Jmp, 5, 0, 0));
    assert_eq!(chunk.instructions[5], Instr::new(Op::SuperBlock, 4, 0, 0));
}

#[test]
fn superblock_refuses_spans_with_internal_branch_targets() {
    let mut chunk = Chunk {
        constants: Vec::new(),
        instructions: vec![
            Instr::new(Op::LoadK, 1, 0, 0),
            Instr::new(Op::LoadK, 2, 1, 0),
            Instr::new(Op::Move, 3, 2, 0),
            Instr::new(Op::Jmp, 1, 0, 0),
            Instr::new(Op::Halt, 0, 0, 0),
        ],
        registers: 3,
        params: 0,
    };

    apply(&mut chunk).unwrap();

    let blocks = chunk
        .instructions
        .iter()
        .filter(|instr| instr.op == Op::SuperBlock)
        .collect::<Vec<_>>();
    assert!(blocks.is_empty());
}

#[test]
fn superblock_recurses_into_function_constants() {
    let child = Chunk {
        constants: Vec::new(),
        instructions: vec![
            Instr::new(Op::Move, 1, 2, 0),
            Instr::new(Op::Add, 1, 1, 2),
            Instr::new(Op::Halt, 0, 0, 0),
        ],
        registers: 2,
        params: 0,
    };
    let mut chunk = Chunk {
        constants: vec![Const::Function {
            chunk: Box::new(child),
            captures: Vec::<Capture>::new(),
        }],
        instructions: vec![Instr::new(Op::Halt, 0, 0, 0)],
        registers: 0,
        params: 0,
    };

    apply(&mut chunk).unwrap();

    let Const::Function {
        chunk: child_chunk, ..
    } = &chunk.constants[0]
    else {
        panic!("expected function constant");
    };
    assert_eq!(child_chunk.instructions[0].op, Op::SuperBlock);
}

#[test]
fn tail_call_global_fuses_constant_and_register_args() {
    let mut chunk = Chunk {
        constants: Vec::new(),
        instructions: vec![
            Instr::new(Op::LoadK, 4, 1, 0),
            Instr::new(Op::Move, 5, 2, 0),
            Instr::new(Op::CallGlobal, 0, 4, 2),
            Instr::new(Op::Halt, 0, 0, 0),
        ],
        registers: 5,
        params: 0,
    };

    apply(&mut chunk).unwrap();

    assert_eq!(
        chunk.instructions,
        vec![Instr::new(Op::TailCallGlobalKR, 0, 1, 2)]
    );
}
