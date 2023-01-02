use super::Log;
use std::fmt::{Display, Formatter};
impl<T: Display> Display for Log<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Log::None => Ok(()),
            Log::Default(s) | Log::Success(s) | Log::Result(s) | Log::Verbose(s) => {
                write!(f, "{}", s)
            }
        }
    }
}

impl<T> Log<T> {
    /// Function which return order of the log.
    pub fn order(&self) -> u8 {
        match self {
            Log::None => 0,
            Log::Default(_) => 1,
            Log::Success(_) => 2,
            Log::Result(_) => 3,
            Log::Verbose(_) => 4,
        }
    }
}
