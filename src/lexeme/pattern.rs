use super::{LexemeLogger, Pattern};
use crate::util::{Code, Log};
use crate::{ILexeme, Lex};
use once_cell::unsync::OnceCell;
use regex::bytes::Regex;
use std::fmt::Debug;
use std::marker::PhantomData;

impl<TToken, TState> Pattern<TToken, TState> {
    /// Create a new [Pattern] lexer utility based on regular expression and corresponding token.
    /// ## Arguments
    /// `token` - Token to be return for the lexical data
    /// `pattern` - Associated regular expression pattern to be matched
    ///  
    /// Given regex expression should not parse an empty string.
    ///
    pub fn new(token: TToken, pattern: &str) -> Result<Self, String> {
        let regexp = Regex::new(pattern)
            .map_err(|err| format!("Pattern should be a valid regex expression.{:?}", err))?;

        if !regexp.is_match(b"") {
            Ok(Self {
                regexp,
                token,
                log: OnceCell::new(),
                _state: PhantomData,
            })
        } else {
            Err(format!(
                "Regex expression '{}' should not be nullable.",
                regexp.as_str()
            ))
        }
    }

    /// Set a log label to debug the lexeme.
    /// Based on the level of the [Log], the lexeme will debug the lexeme result.
    pub fn set_log(&self, log: Log<&'static str>) -> Result<(), String> {
        self.log
            .set(log)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }
}

impl<TToken, TState> LexemeLogger for Pattern<TToken, TState> {
    fn log_cell(&self) -> &OnceCell<crate::util::Log<&'static str>> {
        &self.log
    }
}

impl<TToken, TState> ILexeme for Pattern<TToken, TState>
where
    TToken: Copy + Debug + Eq + Ord,
    TState: Copy + Debug + Eq + Ord,
{
    type Token = TToken;
    type State = TState;

    fn consume(
        &self,
        code: &Code,
        pointer: usize,
        _: &Vec<Lex<Self::Token>>,
        _: &mut Vec<Self::State>,
    ) -> Option<Lex<Self::Token>> {
        self.log_enter();
        if let Some(m) = self.regexp.find(&&code.value[pointer..]) {
            debug_assert_eq!(m.start(), 0);
            let consumed_ptr = pointer + m.end();
            if consumed_ptr != pointer {
                let lex = Lex::new(self.token, pointer, consumed_ptr);
                self.log_success(code, &lex);
                return Some(lex);
            }
        }
        self.log_failure(pointer, code);
        None
    }

    fn get_grammar_field(&self) -> Vec<(TToken, String)> {
        vec![(
            self.token,
            format!("/{}/", self.regexp.as_str().replace('/', "\\/")),
        )]
    }
}
