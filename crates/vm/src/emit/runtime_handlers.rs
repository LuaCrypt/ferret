pub(super) fn move_body(dst: &str, src: &str) -> String {
    format!("R[{dst}]=R[{src}]")
}

pub(super) fn loadk_body(dst: &str, key: &str) -> String {
    format!("R[{dst}]=@K@(C,{key},R,U)")
}

pub(super) fn getglobal_body(dst: &str, key: &str) -> String {
    format!("R[{dst}]=_env[@K@(C,{key},R,U)]")
}

pub(super) fn newtable_body(dst: &str) -> String {
    format!("R[{dst}]={{}}")
}

pub(super) fn gettable_body(dst: &str, table: &str, key: &str) -> String {
    format!("R[{dst}]=R[{table}][R[{key}]]")
}

pub(super) fn settable_body(table: &str, key: &str, value: &str) -> String {
    format!("R[{table}][R[{key}]]=R[{value}]")
}

pub(super) fn return_body(start: &str, count: &str) -> String {
    format!("if {count}==0 then return end; return _u(R,{start},{start}+{count}-1)")
}

pub(super) fn call_body(dst: &str, func: &str, packed: &str, large_pack: bool) -> String {
    let start = "s";
    let count = "n";
    let large = if large_pack {
        pack_call(Some(dst), func, start, count)
    } else {
        "error(0,0)".to_string()
    };
    format!(
        "local s=({packed}>>8)&255; local n={packed}&255; {}",
        call_cases(Some(dst), func, start, count, 4, &large)
    )
}

pub(super) fn call_global_body(key: &str, start: &str, count: &str, large_pack: bool) -> String {
    let large = if large_pack {
        pack_call(None, "f", "s", count)
    } else {
        "error(0,0)".to_string()
    };
    format!(
        "local f=_env[@K@(C,{key},R,U)]; local s={start}; {}",
        call_cases(None, "f", "s", count, 4, &large)
    )
}

pub(super) fn settable_call_body() -> String {
    format!(
        "local n=c&255; local ix=(c>>8)&255; local f=R[b-1]; {}; for i=1,rn do R[a][ix+i-1]=V[i] end",
        packed_results("f", "b", "n")
    )
}

pub(super) fn return_call_body() -> String {
    format!(
        "local n=c&255; local fc=(c>>8)&255; local f=R[b-1]; {}; if fc==0 then return _u(V,1,rn) end; local T={{}}; for i=1,fc do T[i]=R[a+i-1] end; for i=1,rn do T[fc+i]=V[i] end; return _u(T,1,fc+rn)",
        packed_results("f", "b", "n")
    )
}

pub(super) fn calln_body() -> String {
    let r1 = call_cases(
        Some("R[a]"),
        "R[b]",
        "s",
        "n",
        3,
        &pack_call(Some("R[a]"), "R[b]", "s", "n"),
    );
    let r2 = call_cases(
        Some("R[a],R[a+1]"),
        "R[b]",
        "s",
        "n",
        3,
        &pack_call(Some("R[a],R[a+1]"), "R[b]", "s", "n"),
    );
    let r3 = call_cases(
        Some("R[a],R[a+1],R[a+2]"),
        "R[b]",
        "s",
        "n",
        3,
        &pack_call(Some("R[a],R[a+1],R[a+2]"), "R[b]", "s", "n"),
    );
    format!(
        "local r=(c>>8)&255; local n=c&255; local s=a+r; if r==1 then {r1} elseif r==2 then {r2} elseif r==3 then {r3} else {}; for i=1,r do R[a+i-1]=V[i] end end",
        packed_value_table("R[b]", "s", "n")
    )
}

pub(super) fn call3_body() -> String {
    format!(
        "local s=(c>>8)&255; local n=c&255; {}",
        call_cases(
            Some("R[a],R[a+1],R[a+2]"),
            "R[b]",
            "s",
            "n",
            3,
            &pack_call(Some("R[a],R[a+1],R[a+2]"), "R[b]", "s", "n"),
        )
    )
}

fn call_cases(
    assign: Option<&str>,
    func: &str,
    start: &str,
    count: &str,
    max_inline: usize,
    large: &str,
) -> String {
    let mut out = String::new();
    for argc in 0..=max_inline {
        out.push_str(if argc == 0 { "if " } else { " elseif " });
        out.push_str(count);
        out.push_str("==");
        out.push_str(&argc.to_string());
        out.push_str(" then ");
        out.push_str(&invoke(assign, func, start, argc));
    }
    out.push_str(" else ");
    out.push_str(large);
    out.push_str(" end");
    out
}

fn invoke(assign: Option<&str>, func: &str, start: &str, argc: usize) -> String {
    let args = (0..argc)
        .map(|offset| reg(start, offset))
        .collect::<Vec<_>>()
        .join(",");
    match assign {
        Some(assign) => format!("{assign}={func}({args})"),
        None => format!("{func}({args})"),
    }
}

fn pack_call(assign: Option<&str>, func: &str, start: &str, count: &str) -> String {
    let call = format!("{func}(_u(A,1,{count}))");
    let action = match assign {
        Some(assign) => format!("{assign}={call}"),
        None => call,
    };
    format!("local A={{}}; for i=1,{count} do A[i]=R[{start}+i-1] end; {action}")
}

fn packed_results(func: &str, start: &str, count: &str) -> String {
    let large = format!(
        "local A={{}}; for i=1,{count} do A[i]=R[{start}+i-1] end; rn,V=@PR@({func}(_u(A,1,{count})))"
    );
    let mut out = String::from("local rn,V; ");
    for argc in 0..=4 {
        out.push_str(if argc == 0 { "if " } else { " elseif " });
        out.push_str(count);
        out.push_str("==");
        out.push_str(&argc.to_string());
        out.push_str(" then rn,V=@PR@(");
        out.push_str(&invoke(None, func, start, argc));
        out.push(')');
    }
    out.push_str(" else ");
    out.push_str(&large);
    out.push_str(" end");
    out
}

fn packed_value_table(func: &str, start: &str, count: &str) -> String {
    let large = format!(
        "local A={{}}; for i=1,{count} do A[i]=R[{start}+i-1] end; V={{{{{func}(_u(A,1,{count}))}}}}"
    );
    let mut out = String::from("local V; ");
    for argc in 0..=3 {
        out.push_str(if argc == 0 { "if " } else { " elseif " });
        out.push_str(count);
        out.push_str("==");
        out.push_str(&argc.to_string());
        out.push_str(" then V={");
        out.push_str(&invoke(None, func, start, argc));
        out.push('}');
    }
    out.push_str(" else ");
    out.push_str(&large);
    out.push_str(" end");
    out
}

fn reg(start: &str, offset: usize) -> String {
    if offset == 0 {
        format!("R[{start}]")
    } else {
        format!("R[{start}+{offset}]")
    }
}
