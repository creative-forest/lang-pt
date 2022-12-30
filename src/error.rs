use crate::{ImplementationError, ParseError, ProductionError};
use std::fmt::{Display, Formatter};

impl ImplementationError {
    pub fn new(what: String, message: String) -> Self {
        Self { message, what }
    }
}

impl Display for ImplementationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImplementationError: {}-{}", self.what, self.message)
    }
}

impl ProductionError {
    pub fn is_unparsed(&self) -> bool {
        match self {
            ProductionError::Unparsed => true,
            ProductionError::Validation(_, _) => false,
        }
    }
    pub fn is_invalid(&self) -> bool {
        match self {
            ProductionError::Unparsed => false,
            ProductionError::Validation(_, _) => true,
        }
    }
}

impl ParseError {
    pub fn new(pointer: usize, message: String) -> Self {
        Self { pointer, message }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SyntaxError: {}", self.message)
    }
}
