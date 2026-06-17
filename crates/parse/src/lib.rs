pub mod lexer;
pub mod parser;

use ferret_ir::Program;
use ferret_util::Result;

pub fn parse(source: &str) -> Result<Program> {
    let tokens = lexer::lex(source)?;
    parser::Parser::new(tokens).parse_program()
}
