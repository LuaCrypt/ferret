pub(super) fn op_name(op: ferret_ir::Op) -> &'static str {
    op.token()
}
