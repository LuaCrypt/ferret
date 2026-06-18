use ferret_ir::Op;

use crate::emit::opcodes::OpcodePlan;
use crate::emit::runtime_handlers as handlers;

#[derive(Debug, Clone, Default)]
pub(super) struct RuntimeAliases {
    pub(super) halt: String,
    pub(super) loadk: String,
    pub(super) move_: String,
    pub(super) getglobal: String,
    pub(super) newtable: String,
    pub(super) gettable: String,
    pub(super) settable: String,
    pub(super) callglobal: String,
    pub(super) return_: String,
}

pub(super) fn runtime_aliases(opcodes: &OpcodePlan) -> RuntimeAliases {
    RuntimeAliases {
        halt: branches(opcodes, Op::Halt, "return"),
        loadk: branches(opcodes, Op::LoadK, &handlers::loadk_body("a", "b")),
        move_: branches(opcodes, Op::Move, &handlers::move_body("a", "b")),
        getglobal: branches(opcodes, Op::GetGlobal, &handlers::getglobal_body("a", "b")),
        newtable: branches(opcodes, Op::NewTable, &handlers::newtable_body("a")),
        gettable: branches(
            opcodes,
            Op::GetTable,
            &handlers::gettable_body("a", "b", "c"),
        ),
        settable: branches(
            opcodes,
            Op::SetTable,
            &handlers::settable_body("a", "b", "c"),
        ),
        callglobal: branches(
            opcodes,
            Op::CallGlobal,
            &handlers::call_global_body("a", "b", "c", true),
        ),
        return_: branches(opcodes, Op::Return, &handlers::return_body("a", "b")),
    }
}

fn branches(opcodes: &OpcodePlan, op: Op, body: &str) -> String {
    let mut out = String::new();
    for alias in opcodes.aliases_for(op) {
        out.push_str(" elseif op==");
        out.push_str(&alias.name);
        out.push_str(" then ");
        out.push_str(body);
        out.push('\n');
    }
    out
}
