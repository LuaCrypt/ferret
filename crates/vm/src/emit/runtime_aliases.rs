use ferret_ir::Op;

use crate::emit::opcodes::OpcodePlan;

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
        loadk: branches(opcodes, Op::LoadK, "R[a]=@K@(C,b,R,U)"),
        move_: branches(opcodes, Op::Move, "R[a]=R[b]"),
        getglobal: branches(opcodes, Op::GetGlobal, "R[a]=_env[@K@(C,b,R,U)]"),
        newtable: branches(opcodes, Op::NewTable, "R[a]={}"),
        gettable: branches(opcodes, Op::GetTable, "R[a]=R[b][R[c]]"),
        settable: branches(opcodes, Op::SetTable, "R[a][R[b]]=R[c]"),
        callglobal: branches(
            opcodes,
            Op::CallGlobal,
            "local f=_env[@K@(C,a,R,U)]; local s=b; if c==0 then f() elseif c==1 then f(R[s]) elseif c==2 then f(R[s],R[s+1]) elseif c==3 then f(R[s],R[s+1],R[s+2]) elseif c==4 then f(R[s],R[s+1],R[s+2],R[s+3]) else local A={}; for i=1,c do A[i]=R[s+i-1] end; f(_u(A,1,c)) end",
        ),
        return_: branches(
            opcodes,
            Op::Return,
            "if b==0 then return end; return _u(R,a,a+b-1)",
        ),
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
