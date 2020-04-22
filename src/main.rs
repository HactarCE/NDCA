//! A testing ground for a cellular automaton description language for NDCell.
#![allow(dead_code)]
#![warn(missing_docs)]

use inkwell::context::Context;

#[macro_use]
mod macros;

mod ast;
mod compiler;
mod errors;
mod interpreter;
mod span;
mod types;

pub use errors::{CompleteLangResult, LangResult};
pub use span::{Span, Spanned};

const CELL_STATE_COUNT: usize = 100;

fn main() -> Result<(), ()> {
    let source_code = "
            @transition {
                set x = 3
                // if 0 { set some_var = 0 } set some_var += 0 // no-op because variable has been defined
                set y = 2 - 10
                set y -= 3
                // set y = z // use of uninitialized variable
                set z = #(-y / x)
                // set z = 0 // type error
                become z
                // become #(9223372036854775805 + 3)   // overflow
                // become #(-9223372036854775808 / -1) // overflow
                // become #(--9223372036854775808)     // overflow
                // become #(10 % 0)                    // div by zero
                if 3 * 99 % 2 == 1 {
                    become #(10 / 3 * 3)
                } else if 1 + 2 < 2 {
                    become #12
                } else {
                    become #98
                }
                become #2 // unreachable
            }
            ";
    let rule = ast::make_ndca_rule(source_code).map_err(|err| {
        println!(
            "Error while parsing rule and generating AST\n{}",
            err.with_source(source_code)
        );
        ()
    })?;

    println!();
    // Interpret transition function.
    let result = interpret(rule.clone());
    match result {
        Ok(ret) => println!("Interpreted transition function output: {:?}", ret),
        Err(err) => println!(
            "Error while interpreting transition function\n{}",
            err.with_source(source_code)
        ),
    }

    println!();
    // Compile and execute transition function.
    let result = compile_and_run(rule);
    match result {
        Ok(ret) => println!("JIT-compiled transition function output: {:?}", ret),
        Err(err) => println!(
            "Error in compiled transition function\n{}",
            err.with_source(source_code)
        ),
    }

    Ok(())
}

/// Runs the given rule's transition function using the interpreter and returns
/// the result.
fn interpret(rule: ast::Rule) -> LangResult<interpreter::Value> {
    let mut interpreter = interpreter::State::new(rule.transition_fn)?;
    loop {
        if let Some(ret) = interpreter.step()?.return_value() {
            return Ok(ret);
        }
    }
}

/// Runs the given rule's transition function using the compiler and returns the
/// result.
fn compile_and_run(rule: ast::Rule) -> LangResult<types::LangCellState> {
    let context = Context::create();
    let mut compiler = compiler::Compiler::new(&context)?;
    let transition_fn = compiler.jit_compile_fn(&rule.transition_fn)?;
    transition_fn.call()
}

#[cfg(test)]
mod tests;
