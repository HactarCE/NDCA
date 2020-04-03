use inkwell::context::Context;

mod ast;
mod compiler;
mod errors;
mod interpreter;
mod span;
mod types;

pub use errors::{CompleteLangResult, LangResult};
pub use span::{Span, Spanned};

const CELL_STATE_COUNT: usize = 4;

fn main() -> CompleteLangResult<()> {
    let source_code = "
        @transition {
            become #(2 == 2 > 0)
        }
        ";
    let program = make_ast(source_code).map_err(|e| e.with_source(source_code))?;

    // Interpret transition function.
    let result = interpret(program.clone()).map_err(|e| e.with_source(source_code));
    match result {
        Ok(ret) => println!("Interpreted transition function output: {:?}", ret),
        Err(err) => println!("Error while interpreting transition function\n{}", err),
    }
    println!();

    // Compile and execute transition function.
    let result = compile_and_run(program).map_err(|e| e.with_source(source_code));
    match result {
        Ok(ret) => println!("JIT-compiled transition function output: {:?}", ret),
        Err(err) => println!("Error in compiled transition function\n{}", err),
    }

    Ok(())
}

fn interpret(program: ast::Program) -> LangResult<interpreter::Value> {
    let mut interpreter = interpreter::State::new(program.transition_fn);
    loop {
        if let Some(ret) = interpreter.step()?.return_value() {
            return Ok(ret);
        }
    }
}

fn compile_and_run(program: ast::Program) -> LangResult<types::LangCellState> {
    let context = Context::create();
    let mut compiler = compiler::Compiler::new(&context)?;
    let transition_fn = compiler.jit_compile_transition_fn(&program.transition_fn)?;
    transition_fn.call()
}

fn make_ast(source_code: &str) -> LangResult<ast::Program> {
    ast::make_program(source_code)
}
