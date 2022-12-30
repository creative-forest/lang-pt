use crate::{StreamPtr, FltrPtr};
use std::{
    fmt::Display,
    ops::{Add, Sub},
};

impl Default for StreamPtr {
    fn default() -> Self {
        Self(0)
    }
}

impl Display for StreamPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StreamPtr {
    fn new(index: usize) -> Self {
        Self(index)
    }

    #[cfg(debug_assertions)]
    pub fn debug_new(index: usize) -> Self {
        Self(index)
    }
    pub fn origin() -> Self {
        Self(0)
    }
    pub fn is_origin(&self) -> bool {
        self.0 == 0
    }
}

impl From<usize> for StreamPtr {
    fn from(us: usize) -> Self {
        StreamPtr(us)
    }
}

impl Add<usize> for StreamPtr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}
impl Sub<usize> for StreamPtr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        StreamPtr::new(self.0 - rhs)
    }
}

impl Default for FltrPtr {
    fn default() -> Self {
        Self(0)
    }
}

impl Display for FltrPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FltrPtr {
    fn new(index: usize) -> Self {
        Self(index)
    }

    #[cfg(debug_assertions)]
    pub fn debug_new(index: usize) -> Self {
        Self(index)
    }
}

impl Add<usize> for FltrPtr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        if rhs == 0 {
            panic!("Addition of 0 is not valid with lex index");
        }
        Self::new(self.0 + rhs)
    }
}
impl Sub<usize> for FltrPtr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        FltrPtr::new(self.0 - rhs)
    }
}
