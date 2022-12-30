use crate::{NodeImpl, TokenImpl};

impl TokenImpl for i8 {
    fn eof() -> Self {
        Self::MAX
    }

    fn is_structural(&self) -> bool {
        *self >= 0
    }
}
impl TokenImpl for isize {
    fn eof() -> Self {
        Self::MAX
    }
    fn is_structural(&self) -> bool {
        *self >= 0
    }
}
impl TokenImpl for i16 {
    fn eof() -> Self {
        Self::MAX
    }
    fn is_structural(&self) -> bool {
        *self >= 0
    }
}
impl NodeImpl for u8 {
    fn null() -> Self {
        0
    }
}
impl NodeImpl for usize {
    fn null() -> Self {
        0
    }
}
impl NodeImpl for u16 {
    fn null() -> Self {
        0
    }
}
