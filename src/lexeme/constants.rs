use super::{Constants, LexemeLogger};
use crate::{
    util::{Code, Log},
    ILexeme, Lex,
};
use once_cell::unsync::OnceCell;
use std::{fmt::Debug, marker::PhantomData};

impl<TToken: Debug + Copy, TState> Constants<TToken, TState> {
    /// Create a new [Constants] lexeme utility with given set of string values
    /// #Argument
    /// `fields` - A Vec of tuples containing constant string value, associated token.
    pub fn new(mut fields: Vec<(&str, TToken)>) -> Self {
        fields.sort_by_key(|s| s.0.len());

        Self {
            values: fields.iter().map(|(s, t)| (s.to_string(), *t)).collect(),
            log: OnceCell::new(),
            _state: PhantomData,
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

impl<TToken, TState> LexemeLogger for Constants<TToken, TState> {
    fn log_cell(&self) -> &OnceCell<crate::util::Log<&'static str>> {
        &self.log
    }
}

impl<TToken, TState> ILexeme for Constants<TToken, TState>
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
        let result = self.values.iter().rev().find_map(|(value, token)| {
            let lex = Lex::new(token.clone(), pointer, pointer + value.len());
            Some(lex)
        });
        self.log_result(pointer, code, &result);
        result
    }

    fn get_grammar_field(&self) -> Vec<(TToken, String)> {
        self.values
            .iter()
            .map(|(s, t)| (*t, format!("{:?}", s)))
            .collect()
    }
}
