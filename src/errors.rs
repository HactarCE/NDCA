use std::error::Error;
use std::fmt;

use super::span::Span;
use super::types::Type;

pub type CompleteLangResult<T> = Result<T, LangErrorWithSource>;
pub type LangResult<T> = Result<T, LangError>;

#[derive(Debug)]
pub struct LangErrorWithSource {
    pub source_line: Option<String>,
    pub span: Option<(usize, usize)>,
    pub msg: LangErrorMsg,
}
impl fmt::Display for LangErrorWithSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let (Some(line), Some((start, end))) = (&self.source_line, self.span) {
            // Write line of source code.
            writeln!(f, "{}", line)?;
            for _ in 0..(start - 1) {
                write!(f, " ")?;
            }
            // Write arrows pointing to the part with the error.
            for _ in start..end {
                write!(f, "^")?;
            }
            write!(f, "   ")?;
        }
        // Write the error message.
        write!(f, "{}", self.msg)?;
        Ok(())
    }
}
impl Error for LangErrorWithSource {}

#[derive(Debug)]
pub struct LangError {
    pub span: Option<Span>,
    pub msg: LangErrorMsg,
}
impl LangError {
    pub fn with_source(self, src: &str) -> LangErrorWithSource {
        if let Some(span) = self.span {
            let (start_tp, end_tp) = span.textpoints(src);
            let start = start_tp.column();
            let mut end = start;
            if start_tp.line() == end_tp.line() && end_tp.column() > start_tp.column() {
                end = end_tp.column();
            }
            LangErrorWithSource {
                source_line: src
                    .lines()
                    .skip(start_tp.line() - 1)
                    .next()
                    .map(str::to_owned),
                span: Some((start, end)),
                msg: self.msg,
            }
        } else {
            LangErrorWithSource {
                source_line: None,
                span: None,
                msg: self.msg,
            }
        }
    }
}

#[derive(Debug)]
pub enum LangErrorMsg {
    // Miscellaneous errors
    Unimplemented,
    InternalError(&'static str),
    BoxedInternalError(Box<dyn std::error::Error>),

    // Compile errors
    UnknownSymbol,
    Unterminated(&'static str),
    Unmatched(char, char),
    Expected(&'static str),
    TopLevelNonDirective,
    InvalidDirectiveName,
    MissingTransitionFunction,
    MultipleTransitionFunctions,

    // Compile errors for JIT; runtime errors for interpreter
    TypeError { expected: Type, got: Type },
    UseOfUninitializedVariable,

    // Runtime errors
    IntegerOverflowDuringNegation,
    IntegerOverflowDuringAddition,
    IntegerOverflowDuringSubtraction,
    IntegerOverflowDuringMultiplication,
    CellStateOutOfRange,
}
impl<T: 'static + std::error::Error> From<T> for LangErrorMsg {
    fn from(error: T) -> Self {
        Self::BoxedInternalError(Box::new(error))
    }
}
impl fmt::Display for LangErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unimplemented => {
                write!(f, "This feature is unimplemented")?;
            }
            Self::InternalError(s) => {
                write!(f, "Internal error: {}\nThis is a bug in NDCell, not your code. Please report this to the developer!", s)?;
            }
            Self::BoxedInternalError(e) => {
                write!(f, "Internal error: {}\nThis is a bug in NDCell, not your code. Please report this to the developer!", e)?;
            }

            Self::UnknownSymbol => {
                write!(f, "Unknown symbol")?;
            }
            Self::Unterminated(s) => {
                write!(f, "This {} never ends", s)?;
            }
            Self::Unmatched(char1, char2) => {
                write!(f, "This '{}' has no matching '{}'", char1, char2)?;
            }
            Self::Expected(s) => {
                write!(f, "Expected {}", s)?;
            }
            Self::TopLevelNonDirective => {
                write!(f, "Only directives may appear at the top level of a file")?;
            }
            Self::InvalidDirectiveName => {
                write!(f, "Invalid directive name")?;
            }
            Self::MissingTransitionFunction => {
                write!(f, "Missing transition function")?;
            }
            Self::MultipleTransitionFunctions => {
                write!(f, "Multiple transition functions")?;
            }

            Self::TypeError { expected, got } => {
                write!(f, "Type error: expected {} but got {}", expected, got)?;
            }
            Self::UseOfUninitializedVariable => {
                write!(f, "This variable might not be initialized before use")?;
            }

            Self::IntegerOverflowDuringNegation => {
                write!(f, "Integer overflow during negation")?;
            }
            Self::IntegerOverflowDuringAddition => {
                write!(f, "Integer overflow during addition")?;
            }
            Self::IntegerOverflowDuringSubtraction => {
                write!(f, "Integer overflow during subtraction")?;
            }
            Self::IntegerOverflowDuringMultiplication => {
                write!(f, "Integer overflow during multiplication")?;
            }
            Self::CellStateOutOfRange => {
                write!(f, "Cell state out of range")?;
            }
        }
        Ok(())
    }
}
impl LangErrorMsg {
    pub fn with_span(self, span: impl Into<Span>) -> LangError {
        LangError {
            span: Some(span.into()),
            msg: self,
        }
    }
    pub fn without_span(self) -> LangError {
        LangError {
            span: None,
            msg: self,
        }
    }
}

impl<T: Into<LangErrorMsg>> From<T> for LangError {
    fn from(msg: T) -> Self {
        msg.into().without_span()
    }
}