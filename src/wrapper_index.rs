use crate::{TokenPtr, FltrPtr};
use std::{
    fmt::Display,
    ops::{Add, Sub},
};

impl Default for TokenPtr {
    fn default() -> Self {
        Self(0)
    }
}

impl Display for TokenPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TokenPtr {
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

impl From<usize> for TokenPtr {
    fn from(us: usize) -> Self {
        TokenPtr(us)
    }
}

impl Add<usize> for TokenPtr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}
impl Sub<usize> for TokenPtr {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        TokenPtr::new(self.0 - rhs)
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
