pub mod bytecode;
pub mod emit;

pub use bytecode::{compile, CompileReport};
pub use emit::{emit_lua, emit_lua_with_options, EmitOptions, EmitReport, OutputProfile};
