use std::fmt::{Display, Formatter};

use super::Position;

impl Position {
    /// Create a new Position object based ob the line and column number.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
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
