use std::{error, fmt::Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    position: (usize, usize),
    error: ErrorType,
}

impl Error {
    pub fn new(position: (usize, usize), error: ErrorType) -> Self {
        Self { position, error }
    }
}
impl From<pest::error::Error<crate::parser::Rule>> for Error {
    fn from(e: pest::error::Error<crate::parser::Rule>) -> Self {
        let position = match e.line_col {
            pest::error::LineColLocation::Pos(x) => x,
            pest::error::LineColLocation::Span(start, _) => start,
        };
        Self {
            position,
            error: ErrorType::PestError,
        }
    }
}

impl error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.position.0,
            self.position.1,
            match self.error {
                ErrorType::VariableRedefinition => "Variable redefinition",
                ErrorType::IndexNotInt => "Index of array should be integer",
                ErrorType::VariableNotDefined => "Variable has not been defined",
                ErrorType::FunctionRedefinition => "Function redefinition",
                ErrorType::MismatchedType => "Mismatched type",
                ErrorType::MismatchedTypeFunction => "Mismatched type of Function's return type",
                ErrorType::FunctionNotDefined => "Function has not been defined",
                ErrorType::ExpressionVoidType => "Expression has void type",
                ErrorType::PestError => "",
            }
        )
    }
}
#[derive(Debug)]
pub enum ErrorType {
    VariableRedefinition,
    IndexNotInt,
    VariableNotDefined,
    FunctionRedefinition,
    MismatchedType,
    MismatchedTypeFunction,
    FunctionNotDefined,
    ExpressionVoidType,
    PestError,
}
