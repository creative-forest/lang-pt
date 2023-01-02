use super::{LexemeLogger, Punctuations};
use crate::{Code, FieldTree, ILexeme, Lex, Log};
use once_cell::unsync::OnceCell;
use std::{fmt::Debug, marker::PhantomData};

impl<TToken: Debug + Copy, TState> Punctuations<TToken, TState> {
    /// Create a new [Punctuations] lexer utility for a set of constant string values
    /// #Argument
    /// `fields` - A [Vec] of tuples of punctuation string values, and their associated token.
    ///
    pub fn new(mut fields: Vec<(&str, TToken)>) -> Result<Self, String> {
        fields.sort_by_key(|s| s.0.len());
        let mut lexer = Self {
            field_tree: FieldTree::new(),
            punctuations: fields.iter().map(|(s, t)| (s.to_string(), *t)).collect(),
            log: OnceCell::new(),
            _state: PhantomData,
        };
        lexer.add(fields)?;

        Ok(lexer)
    }

    fn add(&mut self, fields: Vec<(&str, TToken)>) -> Result<(), String> {
        for (key, token) in fields {
            self.field_tree
                .insert(key.as_bytes(), token)
                .map_err(|err| {
                    format!(
                        "Punctuation '{}' is already added with token {:?}",
                        key, err
                    )
                })?;
        }

        Ok(())
    }

    /// Set a log label to debug the lexeme.
    /// Based on the level of the [Log], the lexeme will debug the lexeme result.

    pub fn set_log(&self, log: Log<&'static str>) -> Result<(), String> {
        self.log
            .set(log)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }
}

impl<TToken, TState> LexemeLogger for Punctuations<TToken, TState> {
    fn log_cell(&self) -> &OnceCell<crate::Log<&'static str>> {
        &self.log
    }
}

impl<TToken, TState> ILexeme for Punctuations<TToken, TState>
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
        match self.field_tree.find(&code.value[pointer..]) {
            Some((token, index)) => {
                let lex = Lex::new(token, pointer, pointer + index);
                self.log_success(code, &lex);
                Some(lex)
            }
            None => {
                self.log_failure(pointer, code);
                None
            }
        }
    }

    fn get_grammar_field(&self) -> Vec<(TToken, String)> {
        self.punctuations
            .iter()
            .map(|(s, t)| (*t, format!("{:?}", s)))
            .collect()
    }
}
