use ferret_output::{BytecodeLayout, ConstantLayout, RuntimeTemplateVariant};

use crate::emit::runtime_aliases::RuntimeAliases;
use crate::emit::runtime_shape::apply_runtime_shape;
use crate::emit::symbols::Symbols;

pub(super) struct RuntimeInput<'a> {
    pub(super) seed: u64,
    pub(super) syms: &'a Symbols,
    pub(super) op_text: &'a str,
    pub(super) word_text: &'a str,
    pub(super) constant_text: &'a str,
    pub(super) word_count: usize,
    pub(super) reuse_root_registers: bool,
    pub(super) variant: RuntimeTemplateVariant,
    pub(super) bytecode_layout: BytecodeLayout,
    pub(super) constant_layout: ConstantLayout,
    pub(super) aliases: &'a RuntimeAliases,
}

pub(super) fn runtime(input: RuntimeInput<'_>) -> String {
    let RuntimeInput {
        seed,
        syms,
        op_text,
        word_text,
        constant_text,
        word_count,
        reuse_root_registers,
        variant,
        bytecode_layout,
        constant_layout,
        aliases,
    } = input;
    let mut code = format!(
        r#"local _env=_ENV
local _fc=_env.@CACHE@; if _fc==nil then _fc={{}}; _env.@CACHE@=_fc end
local _entry=_fc[{cache_key}]
if _entry then return _entry() end
{op_text}local @M@=2147483647
local _u,_tc,_ch,_num,_sel=table.unpack,table.concat,string.char,tonumber,select
local function @DWV@(T,seed)
 local s=(seed ~ 1113150533)&@M@
 for i=1,#T do s=(s*1103515245+12345+i*97)&@M@; T[i]=(T[i]~s)&@M@ end
end
local function @PW@(W)
 local O,A,B,C={{}},{{}},{{}},{{}}; local j=1
 for i=1,#W,4 do O[j]=W[i]; A[j]=W[i+1]; B[j]=W[i+2]; C[j]=W[i+3]; j=j+1 end
 return {{[{bo}]=O,[{ba}]=A,[{bb}]=B,[{bc}]=C}}
end
local function @DB@(seed,b)
 local s=(seed ~ 1398035015)&255
 local o={{}}
 for i=1,#b do s=(s*73+41+i*17)&255; o[i]=_ch(b[i]~s) end
 return _tc(o)
end
local function @PK@(C)
 if C[{cs}]==1 then return end
 C[{cs}]=1
 local KC=C[{cc}]
 for ci=1,#C[{cm}] do
  local r=C[{cr}][C[{cm}][ci]]; local t=r[1]; local v
  if KC[ci]==nil then
   if t==1 then KC[ci]=r[2]==1
   elseif t==2 then v=@DB@(r[2],r[3]); KC[ci]=_num(v)
   elseif t==3 then KC[ci]=@DB@(r[2],r[3]) end
  end
 end
end
local @RUN@
local function @K@(C,i,R,U)
 local k=i+1; local cache=C[{cc}]; local v=cache[k]
 if v~=nil then return v end
 local r=C[{cr}][C[{cm}][k]]; local t=r[1]
 if t==0 then return nil elseif t==1 then v=r[2]==1; cache[k]=v; return v end
 if t==4 then
  if #r[6]==0 and r[8]~=nil then return r[8] end
  if r[7]~=1 then @DWV@(r[3],r[2]); r[3]=@PW@(r[3]); r[7]=1 end
  local FW,FC,P,CAP=r[3],r[4],r[5],r[6]
  @PK@(FC)
  local NU={{}}
  for i=1,#CAP do local c=CAP[i]; if c[1]==0 then NU[i]=R[c[2]] else NU[i]=U[c[2]+1] end end
  local FN=function(...)
   return @RUN@(FW,FC,P,NU,_sel('#',...),nil,...)
  end
  if #CAP==0 then r[8]=FN end
  return FN
 end
 v=@DB@(r[2],r[3]); if t==2 then v=_num(v) end; cache[k]=v; return v
end
@RUN@=function(W,C,P,U,N,R,...)
R=R or {{}}
if P==1 then R[1]=...
elseif P==2 then local a1,a2=...; R[1]=a1; R[2]=a2
elseif P==3 then local a1,a2,a3=...; R[1]=a1; R[2]=a2; R[3]=a3
elseif P>0 then local A={{...}}; for i=1,P do R[i]=A[i] end end
local O,WA,WB,WC=W[{bo}],W[{ba}],W[{bb}],W[{bc}]
local pc=1; local KC=C[{cc}]
while true do
 local op,a,b,c=O[pc],WA[pc],WB[pc],WC[pc]; pc=pc+1
 if op==OP_SUPERBLOCK then
  local ep=pc+a
  while pc<ep do
   local mop,ma,mb,mc=O[pc],WA[pc],WB[pc],WC[pc]; pc=pc+1
   if mop==OP_MOVE then R[ma]=R[mb]
   elseif mop==OP_LOADK then R[ma]=@K@(C,mb,R,U)
   elseif mop==OP_ADDMODK then R[ma]=(R[ma]+R[mb])%KC[mc+1]
   elseif mop==OP_MULKADDMODK then local mk=(mc>>8)&255; local dk=mc&255; R[ma]=(R[ma]*KC[mk+1]+R[mb])%KC[dk+1]
   elseif mop==OP_ADDSELECTLT then if R[mb]<R[mc] then R[ma]=R[ma]+R[mb] else R[ma]=R[ma]+R[mc] end
   elseif mop==OP_ADD then R[ma]=R[mb]+R[mc]
   elseif mop==OP_ADDK then R[ma]=R[mb]+KC[mc+1]
   elseif mop==OP_SUB then R[ma]=R[mb]-R[mc]
   elseif mop==OP_SUBK then R[ma]=R[mb]-KC[mc+1]
   elseif mop==OP_MUL then R[ma]=R[mb]*R[mc]
   elseif mop==OP_MULK then R[ma]=R[mb]*KC[mc+1]
   elseif mop==OP_DIV then R[ma]=R[mb]/R[mc]
   elseif mop==OP_DIVK then R[ma]=R[mb]/KC[mc+1]
   elseif mop==OP_FLOORDIV then R[ma]=R[mb]//R[mc]
   elseif mop==OP_FLOORDIVK then R[ma]=R[mb]//KC[mc+1]
   elseif mop==OP_MOD then R[ma]=R[mb]%R[mc]
   elseif mop==OP_MODK then R[ma]=R[mb]%KC[mc+1]
   elseif mop==OP_POW then R[ma]=R[mb]^R[mc]
   elseif mop==OP_POWK then R[ma]=R[mb]^KC[mc+1]
   elseif mop==OP_LT then R[ma]=R[mb]<R[mc]
   elseif mop==OP_LE then R[ma]=R[mb]<=R[mc]
   elseif mop==OP_EQ then R[ma]=R[mb]==R[mc]
   elseif mop==OP_GETGLOBAL then R[ma]=_env[@K@(C,mb,R,U)]
   elseif mop==OP_NEWTABLE then R[ma]={{}}
   elseif mop==OP_GETTABLE then R[ma]=R[mb][R[mc]]
   elseif mop==OP_SETTABLE then R[ma][R[mb]]=R[mc]
   elseif mop==OP_CELL then R[ma]={{R[mb]}}
   elseif mop==OP_GETCELL then R[ma]=R[mb][1]
   elseif mop==OP_SETCELL then R[ma][1]=R[mb]
   elseif mop==OP_GETUP then R[ma]=U[mb+1][1]
   elseif mop==OP_SETUP then U[ma+1][1]=R[mb]
   elseif mop==OP_CALL then local s=(mc>>8)&255; local n=mc&255; if n==0 then R[ma]=R[mb]() elseif n==1 then R[ma]=R[mb](R[s]) elseif n==2 then R[ma]=R[mb](R[s],R[s+1]) elseif n==3 then R[ma]=R[mb](R[s],R[s+1],R[s+2]) elseif n==4 then R[ma]=R[mb](R[s],R[s+1],R[s+2],R[s+3]) else error(0,0) end
   elseif mop==OP_CALLGLOBAL then local f=_env[KC[ma+1]]; local s=mb; if mc==0 then f() elseif mc==1 then f(R[s]) elseif mc==2 then f(R[s],R[s+1]) elseif mc==3 then f(R[s],R[s+1],R[s+2]) elseif mc==4 then f(R[s],R[s+1],R[s+2],R[s+3]) else error(0,0) end
   elseif mop==OP_AND then R[ma]=R[mb] and R[mc]
   elseif mop==OP_OR then R[ma]=R[mb] or R[mc]
   elseif mop==OP_BITAND then R[ma]=R[mb]&R[mc]
   elseif mop==OP_BITXOR then R[ma]=R[mb]~R[mc]
   elseif mop==OP_BITOR then R[ma]=R[mb]|R[mc]
   elseif mop==OP_SHL then R[ma]=R[mb]<<R[mc]
   elseif mop==OP_SHR then R[ma]=R[mb]>>R[mc]
   elseif mop==OP_CONCAT then R[ma]=R[mb]..R[mc]
   elseif mop==OP_NOT then R[ma]=not R[mb]
   elseif mop==OP_NEG then R[ma]=-R[mb]
   elseif mop==OP_LEN then R[ma]=#R[mb]
   elseif mop==OP_BITNOT then R[ma]=~R[mb]
   elseif mop==OP_HALT then return
   elseif mop==OP_RETURN then if mb==0 then return end; return _u(R,ma,ma+mb-1)
   elseif mop==OP_RETURNVARARG then local T={{}}; local n=mb; for i=1,mb do T[i]=R[ma+i-1] end; for i=P+1,N do n=n+1; T[n]=_sel(i,...) end; return _u(T,1,n)
   else error(0,0) end
  end
 elseif op==OP_MOVE then R[a]=R[b]
{alias_move} elseif op==OP_LOADK then R[a]=@K@(C,b,R,U)
{alias_loadk}
 elseif op==OP_FORCHECKPOS then if R[a]>R[a+1] then pc=b+1 end
 elseif op==OP_FORSTEPPOS then R[a]=R[a]+R[a+2]; if R[a]<=R[a+1] then pc=b+1 end
 elseif op==OP_FORSTEPADDPOS then R[a]=R[a]+R[b]; R[b]=R[b]+R[b+2]; if R[b]<=R[b+1] then pc=c+1 end
 elseif op==OP_ADDMODK then R[a]=(R[a]+R[b])%KC[c+1]
 elseif op==OP_MULKADDMODK then local mk=(c>>8)&255; local dk=c&255; R[a]=(R[a]*KC[mk+1]+R[b])%KC[dk+1]
 elseif op==OP_ADDSELECTLT then if R[b]<R[c] then R[a]=R[a]+R[b] else R[a]=R[a]+R[c] end
 elseif op==OP_HALT then return
{alias_halt}
 elseif op==OP_ADD then R[a]=R[b]+R[c]
 elseif op==OP_ADDK then R[a]=R[b]+KC[c+1]
 elseif op==OP_SUB then R[a]=R[b]-R[c]
 elseif op==OP_SUBK then R[a]=R[b]-KC[c+1]
 elseif op==OP_MUL then R[a]=R[b]*R[c]
 elseif op==OP_MULK then R[a]=R[b]*KC[c+1]
 elseif op==OP_DIV then R[a]=R[b]/R[c]
 elseif op==OP_DIVK then R[a]=R[b]/KC[c+1]
 elseif op==OP_FLOORDIV then R[a]=R[b]//R[c]
 elseif op==OP_FLOORDIVK then R[a]=R[b]//KC[c+1]
 elseif op==OP_MOD then R[a]=R[b]%R[c]
 elseif op==OP_MODK then R[a]=R[b]%KC[c+1]
 elseif op==OP_POW then R[a]=R[b]^R[c]
 elseif op==OP_POWK then R[a]=R[b]^KC[c+1]
 elseif op==OP_LT then R[a]=R[b]<R[c]
 elseif op==OP_LE then R[a]=R[b]<=R[c]
 elseif op==OP_EQ then R[a]=R[b]==R[c]
 elseif op==OP_JMPNOTLT then if not (R[a]<R[b]) then pc=c+1 end
 elseif op==OP_JMPNOTLE then if not (R[a]<=R[b]) then pc=c+1 end
 elseif op==OP_JMPNOTEQ then if R[a]~=R[b] then pc=c+1 end
 elseif op==OP_JMPFALSE then if not R[a] then pc=b+1 end
 elseif op==OP_JMP then pc=a+1
 elseif op==OP_FORCHECK then local s=R[a+2]; local v=R[a]; local l=R[a+1]; if not ((s>=0 and v<=l) or (s<0 and v>=l)) then pc=b+1 end
 elseif op==OP_FORSTEP then R[a]=R[a]+R[a+2]; local s=R[a+2]; local v=R[a]; local l=R[a+1]; if (s>=0 and v<=l) or (s<0 and v>=l) then pc=b+1 end
 elseif op==OP_GETGLOBAL then R[a]=_env[@K@(C,b,R,U)]
{alias_getglobal}
 elseif op==OP_SETGLOBAL then _env[@K@(C,a,R,U)]=R[b]
 elseif op==OP_NEWTABLE then R[a]={{}}
{alias_newtable} elseif op==OP_GETTABLE then R[a]=R[b][R[c]]
{alias_gettable} elseif op==OP_SETTABLE then R[a][R[b]]=R[c]
{alias_settable}
 elseif op==OP_GENERICFOR then if c==1 then local v1=R[b](R[b+1],R[b+2]); R[b+2]=v1; R[a]=v1 elseif c==2 then local v1,v2=R[b](R[b+1],R[b+2]); R[b+2]=v1; R[a]=v1; R[a+1]=v2 elseif c==3 then local v1,v2,v3=R[b](R[b+1],R[b+2]); R[b+2]=v1; R[a]=v1; R[a+1]=v2; R[a+2]=v3 else local V={{R[b](R[b+1],R[b+2])}}; R[b+2]=V[1]; for i=1,c do R[a+i-1]=V[i] end end
 elseif op==OP_CELL then R[a]={{R[b]}}
 elseif op==OP_GETCELL then R[a]=R[b][1]
 elseif op==OP_SETCELL then R[a][1]=R[b]
 elseif op==OP_GETUP then R[a]=U[b+1][1]
 elseif op==OP_SETUP then U[a+1][1]=R[b]
 elseif op==OP_RETURN then if b==0 then return end; return _u(R,a,a+b-1)
{alias_return}
 elseif op==OP_RETURNVARARG then local T={{}}; local n=b; for i=1,b do T[i]=R[a+i-1] end; for i=P+1,N do n=n+1; T[n]=_sel(i,...) end; return _u(T,1,n)
 elseif op==OP_CALL then local s=(c>>8)&255; local n=c&255; if n==0 then R[a]=R[b]() elseif n==1 then R[a]=R[b](R[s]) elseif n==2 then R[a]=R[b](R[s],R[s+1]) elseif n==3 then R[a]=R[b](R[s],R[s+1],R[s+2]) elseif n==4 then R[a]=R[b](R[s],R[s+1],R[s+2],R[s+3]) else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a]=R[b](_u(A,1,n)) end
 elseif op==OP_CALLGLOBAL then local f=_env[KC[a+1]]; local s=b; if c==0 then f() elseif c==1 then f(R[s]) elseif c==2 then f(R[s],R[s+1]) elseif c==3 then f(R[s],R[s+1],R[s+2]) elseif c==4 then f(R[s],R[s+1],R[s+2],R[s+3]) else local A={{}}; for i=1,c do A[i]=R[s+i-1] end; f(_u(A,1,c)) end
{alias_callglobal}
 elseif op==OP_TAILCALLGLOBAL then local f=_env[KC[a+1]]; if c==0 then f() elseif c==1 then f(R[b]) elseif c==2 then f(R[b],R[b+1]) else local A={{}}; for i=1,c do A[i]=R[b+i-1] end; f(_u(A,1,c)) end; return
 elseif op==OP_TAILCALLGLOBALR then _env[KC[a+1]](R[b]); return
 elseif op==OP_TAILCALLGLOBALRR then _env[KC[a+1]](R[b],R[c]); return
 elseif op==OP_TAILCALLGLOBALK then _env[KC[a+1]](KC[b+1]); return
 elseif op==OP_TAILCALLGLOBALKK then _env[KC[a+1]](KC[b+1],KC[c+1]); return
 elseif op==OP_TAILCALLGLOBALKR then _env[KC[a+1]](KC[b+1],R[c]); return
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
 elseif op==OP_CALLN then local r=(c>>8)&255; local n=c&255; local s=a+r; if r==1 then if n==0 then R[a]=R[b]() elseif n==1 then R[a]=R[b](R[s]) elseif n==2 then R[a]=R[b](R[s],R[s+1]) elseif n==3 then R[a]=R[b](R[s],R[s+1],R[s+2]) else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a]=R[b](_u(A,1,n)) end elseif r==2 then if n==0 then R[a],R[a+1]=R[b]() elseif n==1 then R[a],R[a+1]=R[b](R[s]) elseif n==2 then R[a],R[a+1]=R[b](R[s],R[s+1]) elseif n==3 then R[a],R[a+1]=R[b](R[s],R[s+1],R[s+2]) else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a],R[a+1]=R[b](_u(A,1,n)) end elseif r==3 then if n==0 then R[a],R[a+1],R[a+2]=R[b]() elseif n==1 then R[a],R[a+1],R[a+2]=R[b](R[s]) elseif n==2 then R[a],R[a+1],R[a+2]=R[b](R[s],R[s+1]) elseif n==3 then R[a],R[a+1],R[a+2]=R[b](R[s],R[s+1],R[s+2]) else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a],R[a+1],R[a+2]=R[b](_u(A,1,n)) end else local V; if n==0 then V={{R[b]()}} elseif n==1 then V={{R[b](R[s])}} elseif n==2 then V={{R[b](R[s],R[s+1])}} elseif n==3 then V={{R[b](R[s],R[s+1],R[s+2])}} else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; V={{R[b](_u(A,1,n))}} end; for i=1,r do R[a+i-1]=V[i] end end
 elseif op==OP_CALL3 then local s=(c>>8)&255; local n=c&255; if n==0 then R[a],R[a+1],R[a+2]=R[b]() elseif n==1 then R[a],R[a+1],R[a+2]=R[b](R[s]) elseif n==2 then R[a],R[a+1],R[a+2]=R[b](R[s],R[s+1]) elseif n==3 then R[a],R[a+1],R[a+2]=R[b](R[s],R[s+1],R[s+2]) else local A={{}}; for i=1,n do A[i]=R[s+i-1] end; R[a],R[a+1],R[a+2]=R[b](_u(A,1,n)) end
 else error(0,0) end
end
end
local @W@,@C@
@W@={word_text}; @C@={constant_text}; @DWV@(@W@,{seed}); @W@=@PW@(@W@)
@PK@(@C@)
{entry_fn}
_fc[{cache_key}]=_entry_fn
return _entry_fn()"#,
        seed = seed as u32,
        cache_key = (seed as u32) ^ ((word_count as u32) << 1),
        bo = bytecode_layout.opcode,
        ba = bytecode_layout.a,
        bb = bytecode_layout.b,
        bc = bytecode_layout.c,
        cr = constant_layout.rows,
        cm = constant_layout.map,
        cc = constant_layout.cache,
        cs = constant_layout.state,
        op_text = op_text,
        word_text = word_text,
        constant_text = constant_text,
        alias_move = aliases.move_,
        alias_loadk = aliases.loadk,
        alias_halt = aliases.halt,
        alias_getglobal = aliases.getglobal,
        alias_newtable = aliases.newtable,
        alias_gettable = aliases.gettable,
        alias_settable = aliases.settable,
        alias_return = aliases.return_,
        alias_callglobal = aliases.callglobal,
        entry_fn = if reuse_root_registers {
            "local _root_r={}\nlocal function _entry_fn() return @RUN@(@W@,@C@,0,nil,0,_root_r) end"
        } else {
            "local function _entry_fn() return @RUN@(@W@,@C@,0,nil,0,nil) end"
        }
    );
    apply_runtime_shape(&mut code, variant);
    syms.apply(&mut code);
    code
}
