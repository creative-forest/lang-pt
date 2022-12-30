use std::fmt::{Debug, Display, Formatter};

use crate::{util::Code, Lex};

use super::Log;
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

impl<TL: Display> Log<TL> {
    pub fn log_success<T: Debug>(
        &self,
        index: usize,
        result: Option<Lex<T>>,
        code: &Code,
    ) -> Option<Lex<T>> {
        #[cfg(debug_assertions)]
        match &result {
            Some(data) => {
                if self.order() >= Log::Success(()).order() {
                    println!(
                        "[{}; LexemeSuccess]: token: {:?} at {}",
                        self,
                        data.token,
                        code.obtain_position(data.start)
                    )
                }
            }
            None => {
                if self.order() >= Log::Result(()).order() {
                    println!(
                        "[{}; LexemeError]: at {}",
                        self,
                        code.obtain_position(index)
                    )
                }
            }
        }

        result
    }
    pub fn wrap_lexeme_result<T: Debug>(
        &self,
        index: usize,
        result: Option<Lex<T>>,
        code: &Code,
    ) -> Option<Lex<T>> {
        #[cfg(debug_assertions)]
        match &result {
            Some(data) => {
                if self.order() >= Log::Success(()).order() {
                    println!(
                        "[{}; LexemeSuccess]: token: {:?} at {}",
                        self,
                        data.token,
                        code.obtain_position(data.start)
                    )
                }
            }
            None => {
                if self.order() >= Log::Result(()).order() {
                    println!(
                        "[{}; LexemeError]: at {}",
                        self,
                        code.obtain_position(index)
                    )
                }
            }
        }

        result
    }
}
