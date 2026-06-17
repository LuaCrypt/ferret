use ferret_crypto::{encode_bytes, encode_words};
use ferret_ir::{Capture, Chunk, Const};
use ferret_util::stable_hash;

use crate::bytecode::layout::opcode_layout;
use crate::emit::names::op_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitReport {
    pub code: String,
    pub bytecode_words: usize,
    pub constant_count: usize,
    pub output_hash: u64,
}

pub fn emit_lua(chunk: &Chunk, seed: u64) -> EmitReport {
    let layout = opcode_layout(seed);
    let raw_words = pack(chunk, &layout);
    let enc_words = encode_words(&raw_words, seed);
    let mut code = String::new();
    code.push_str("do\nlocal W=");
    words(&mut code, &enc_words);
    code.push_str("\nlocal C=");
    constants(&mut code, &chunk.constants, seed, &layout);
    code.push('\n');
    op_locals(&mut code, &layout);
    code.push_str(&runtime(seed));
    code.push_str("\nend\n");
    EmitReport {
        output_hash: stable_hash(code.as_bytes()),
        code,
        bytecode_words: enc_words.len(),
        constant_count: chunk.constants.len(),
    }
}

fn pack(chunk: &Chunk, layout: &std::collections::BTreeMap<ferret_ir::Op, u32>) -> Vec<u32> {
    let mut words = Vec::with_capacity(chunk.instructions.len() * 4);
    for instr in &chunk.instructions {
        words.push(layout[&instr.op]);
        words.push(u32::from(instr.a));
        words.push(u32::from(instr.b));
        words.push(u32::from(instr.c));
    }
    words
}

fn constants(
    out: &mut String,
    constants: &[Const],
    seed: u64,
    layout: &std::collections::BTreeMap<ferret_ir::Op, u32>,
) {
    out.push('{');
    for (index, constant) in constants.iter().enumerate() {
        let item_seed = seed ^ ((index as u64 + 1) * 0x9e37);
        match constant {
            Const::Nil => out.push_str("{0},"),
            Const::Bool(value) => {
                out.push_str(if *value { "{1,1}," } else { "{1,0}," });
            }
            Const::Number(value) => protected(out, 2, &number_text(*value), item_seed),
            Const::String(value) => protected(out, 3, value, item_seed),
            Const::Function { chunk, captures } => {
                function_const(out, chunk, captures, item_seed, layout);
            }
        }
    }
    out.push('}');
}

fn function_const(
    out: &mut String,
    chunk: &Chunk,
    captures: &[Capture],
    seed: u64,
    layout: &std::collections::BTreeMap<ferret_ir::Op, u32>,
) {
    out.push_str("{4,");
    out.push_str(&(seed as u32).to_string());
    out.push(',');
    words(out, &encode_words(&pack(chunk, layout), seed));
    out.push(',');
    constants(out, &chunk.constants, seed, layout);
    out.push(',');
    out.push_str(&chunk.params.to_string());
    out.push(',');
    capture_list(out, captures);
    out.push_str("},");
}

fn capture_list(out: &mut String, captures: &[Capture]) {
    out.push('{');
    for capture in captures {
        match capture {
            Capture::Local(reg) => {
                out.push_str("{0,");
                out.push_str(&reg.to_string());
                out.push_str("},");
            }
            Capture::Upvalue(index) => {
                out.push_str("{1,");
                out.push_str(&index.to_string());
                out.push_str("},");
            }
        }
    }
    out.push('}');
}

fn protected(out: &mut String, tag: u8, value: &str, seed: u64) {
    out.push('{');
    out.push_str(&tag.to_string());
    out.push(',');
    out.push_str(&(seed as u32).to_string());
    out.push(',');
    bytes(out, &encode_bytes(value.as_bytes(), seed));
    out.push_str("},");
}

fn op_locals(out: &mut String, layout: &std::collections::BTreeMap<ferret_ir::Op, u32>) {
    for (op, value) in layout {
        out.push_str("local ");
        out.push_str(op_name(*op));
        out.push('=');
        out.push_str(&value.to_string());
        out.push('\n');
    }
}

fn runtime(seed: u64) -> String {
    format!(
        r#"local M=2147483647
local function dwv(T,seed)
 local s=(seed ~ 1113150533)&M
 for i=1,#T do s=(s*1103515245+12345+i*97)&M; T[i]=(T[i]~s)&M end
end
local function db(seed,b)
 local s=(seed ~ 1398035015)&255
 local o={{}}
 for i=1,#b do s=(s*73+41+i*17)&255; o[i]=string.char(b[i]~s) end
 return table.concat(o)
end
local run
local function K(C,i,R,U)
 local r=C[i+1]; local t=r[1]
 if t==0 then return nil elseif t==1 then return r[2]==1 end
 if t==4 then
  if r[7]~=1 then dwv(r[3],r[2]); r[7]=1 end
  local FW,FC,P,CAP=r[3],r[4],r[5],r[6]
  return function(...)
   local NU={{}}
   for i=1,#CAP do local c=CAP[i]; if c[1]==0 then NU[i]=R[c[2]] else NU[i]=U[c[2]+1] end end
   return run(FW,FC,P,{{...}},NU,select('#',...))
  end
 end
 local v=db(r[2],r[3]); if t==2 then return tonumber(v) end; return v
end
run=function(W,C,P,A,U,N)
local R={{}}; for i=1,P do R[i-1]=A[i] end; local pc=1
while true do
 local op,a,b,c=W[pc],W[pc+1],W[pc+2],W[pc+3]; pc=pc+4
 if op==OP_HALT then return
 elseif op==OP_LOADK then R[a]=K(C,b,R,U)
 elseif op==OP_MOVE then R[a]=R[b]
 elseif op==OP_GETGLOBAL then R[a]=_ENV[K(C,b)]
 elseif op==OP_SETGLOBAL then _ENV[K(C,a)]=R[b]
 elseif op==OP_NEWTABLE then R[a]={{}}
 elseif op==OP_GETTABLE then R[a]=R[b][R[c]]
 elseif op==OP_SETTABLE then R[a][R[b]]=R[c]
 elseif op==OP_ADD then R[a]=R[b]+R[c]
 elseif op==OP_SUB then R[a]=R[b]-R[c]
 elseif op==OP_MUL then R[a]=R[b]*R[c]
 elseif op==OP_DIV then R[a]=R[b]/R[c]
 elseif op==OP_FLOORDIV then R[a]=R[b]//R[c]
 elseif op==OP_MOD then R[a]=R[b]%R[c]
 elseif op==OP_POW then R[a]=R[b]^R[c]
 elseif op==OP_EQ then R[a]=R[b]==R[c]
 elseif op==OP_LT then R[a]=R[b]<R[c]
 elseif op==OP_LE then R[a]=R[b]<=R[c]
 elseif op==OP_AND then R[a]=R[b] and R[c]
 elseif op==OP_OR then R[a]=R[b] or R[c]
 elseif op==OP_BITAND then R[a]=R[b]&R[c]
 elseif op==OP_BITXOR then R[a]=R[b]~R[c]
 elseif op==OP_BITOR then R[a]=R[b]|R[c]
 elseif op==OP_SHL then R[a]=R[b]<<R[c]
 elseif op==OP_SHR then R[a]=R[b]>>R[c]
 elseif op==OP_CONCAT then R[a]=R[b]..R[c]
 elseif op==OP_NOT then R[a]=not R[b]
 elseif op==OP_NEG then R[a]=-R[b]
 elseif op==OP_LEN then R[a]=#R[b]
 elseif op==OP_BITNOT then R[a]=~R[b]
 elseif op==OP_CELL then R[a]={{R[b]}}
 elseif op==OP_GETCELL then R[a]=R[b][1]
 elseif op==OP_SETCELL then R[a][1]=R[b]
 elseif op==OP_GETUP then R[a]=U[b+1][1]
 elseif op==OP_SETUP then U[a+1][1]=R[b]
 elseif op==OP_JMP then pc=a*4+1
 elseif op==OP_JMPFALSE then if not R[a] then pc=b*4+1 end
 elseif op==OP_CALL then local s=(c>>8)&255; local n=c&255; local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a]=R[b](table.unpack(A,1,n))
 elseif op==OP_CALLN then local r=(c>>8)&255; local n=c&255; local s=a+r; local A={{}}; for i=1,n do A[i]=R[s+i-1] end; local V={{R[b](table.unpack(A,1,n))}}; for i=1,r do R[a+i-1]=V[i] end
 elseif op==OP_CALL3 then local s=(c>>8)&255; local n=c&255; local A={{}}; for i=1,n do A[i]=R[s+i-1] end; local V={{R[b](table.unpack(A,1,n))}}; R[a]=V[1]; R[a+1]=V[2]; R[a+2]=V[3]
 elseif op==OP_GENERICFOR then local V={{R[b](R[b+1],R[b+2])}}; R[b+2]=V[1]; for i=1,c do R[a+i-1]=V[i] end
 elseif op==OP_RETURN then if b==0 then return end; return table.unpack(R,a,a+b-1)
 elseif op==OP_RETURNVARARG then local T={{}}; local n=b; for i=1,b do T[i]=R[a+i-1] end; for i=P+1,N do n=n+1; T[n]=A[i] end; return table.unpack(T,1,n)
 else error('ferret vm fault',0) end
end
end
dwv(W,{seed})
return run(W,C,0,{{}},{{}},0)"#,
        seed = seed as u32
    )
}

fn words(out: &mut String, values: &[u32]) {
    out.push('{');
    for value in values {
        out.push_str(&value.to_string());
        out.push(',');
    }
    out.push('}');
}

fn bytes(out: &mut String, values: &[u8]) {
    out.push('{');
    for value in values {
        out.push_str(&value.to_string());
        out.push(',');
    }
    out.push('}');
}

fn number_text(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
