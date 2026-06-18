use ferret_output::RuntimeTemplateVariant;

pub(super) fn apply_runtime_shape(code: &mut String, variant: RuntimeTemplateVariant) {
    *code = code.replace("while true do", "while(true) do");
    match variant {
        RuntimeTemplateVariant::Compact => {}
        RuntimeTemplateVariant::SwappedFetch => swap_fetches(code),
        RuntimeTemplateVariant::SwappedCompare => swap_comparisons(code),
        RuntimeTemplateVariant::Mixed => {
            swap_fetches(code);
            swap_comparisons(code);
        }
    }
}

fn swap_fetches(code: &mut String) {
    *code = code.replace(
        "local op,a,b,c=O[pc],WA[pc],WB[pc],WC[pc]; pc=pc+1",
        "local a,b,c,op=WA[pc],WB[pc],WC[pc],O[pc]; pc=pc+1",
    );
    *code = code.replace(
        "local mop,ma,mb,mc=O[pc],WA[pc],WB[pc],WC[pc]; pc=pc+1",
        "local ma,mb,mc,mop=WA[pc],WB[pc],WC[pc],O[pc]; pc=pc+1",
    );
}

fn swap_comparisons(code: &mut String) {
    *code = swap_pattern(code, "op==OP_", "op");
    *code = swap_pattern(code, "mop==OP_", "mop");
}

fn swap_pattern(code: &str, pattern: &str, register: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut start = 0;
    while let Some(offset) = code[start..].find(pattern) {
        let found = start + offset;
        if found > 0 && is_ident_next(code[..found].chars().next_back().unwrap()) {
            out.push_str(&code[start..found + 1]);
            start = found + 1;
            continue;
        }
        let token_start = found + register.len() + 2;
        let token_end = token_end(code, token_start);
        out.push_str(&code[start..found]);
        out.push_str(&code[token_start..token_end]);
        out.push_str("==");
        out.push_str(register);
        start = token_end;
    }
    out.push_str(&code[start..]);
    out
}

fn token_end(code: &str, start: usize) -> usize {
    code[start..]
        .char_indices()
        .find_map(|(offset, ch)| (!is_ident_next(ch)).then_some(start + offset))
        .unwrap_or(code.len())
}

fn is_ident_next(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
