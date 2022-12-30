mod code;
mod logger;
mod position;
use once_cell::unsync::OnceCell;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

pub struct Code<'c> {
    pub value: &'c [u8],
    line_breaks: OnceCell<Vec<usize>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A enum structure to assign multiple level debugging to lexeme and production utilities.
pub enum Log<T> {
    None,
    Default(T),
    Success(T),
    Result(T),
    Verbose(T),
}
