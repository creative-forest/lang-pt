use crate::Lex;
use std::fmt::{Debug, Display, Formatter};

impl<TToken: Debug> Display for Lex<TToken> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.token)
            .field(&self.start)
            .field(&self.end)
            .finish()
    }
}

impl<TToken> Lex<TToken> {
    pub fn new(token: TToken, start: usize, end: usize) -> Self {
        Self { token, start, end }
    }
}
