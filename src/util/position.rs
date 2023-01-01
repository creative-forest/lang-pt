use std::fmt::{Display, Formatter};

use super::Position;

impl Position {
    /// Create a new Position object based ob the line and column number.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl From<&[u8]> for Position {
    fn from(code: &[u8]) -> Self {
        let mut pointer: usize = 0;
        let mut line: usize = 0;
        for c in code {
            if *c == b'\n' {
                line += 1;
            }
            pointer += 1;
        }
        let s = unsafe { std::str::from_utf8_unchecked(&code[pointer..]) };
        Position::new(line + 1, s.len() + 1)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("")
            .field("line", &self.line)
            .field("column", &self.column)
            .finish()
    }
}
