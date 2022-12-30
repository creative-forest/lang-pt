use super::{LexemeLogger, Mapper, ThunkMapper};
use crate::{
    util::{Code, Log},
    ILexeme, Lex,
};
use once_cell::unsync::OnceCell;
use std::collections::HashMap;

impl<TS: ILexeme> Mapper<TS> {
    /// Create a new [Mapper] utility.
    /// ## Arguments
    /// * 'lexeme' - Another lexer utility which implement [ILexeme] trait.
    /// * 'fields' - A [Vec] of tuples of mappable string values, and their token values.
    pub fn new(lexeme: TS, fields: Vec<(&str, TS::Token)>) -> Result<Self, String> {
        let mut s = Self {
            lexeme,
            fields: HashMap::new(),
            log: OnceCell::new(),
        };

        s.extend_fields(fields).map(|_| s)
    }

    /// Set a log label to debug the lexeme.
    /// Based on the level of the [Log], the lexeme will debug the lexeme result.
    pub fn set_log(&self, log: Log<&'static str>) -> Result<(), String> {
        self.log
            .set(log)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }

    pub fn extend_fields(&mut self, fields: Vec<(&str, TS::Token)>) -> Result<(), String> {
        for (keyword, token) in fields {
            if let Some(token) = self
                .fields
                .insert(keyword.bytes().collect::<Vec<u8>>(), token)
            {
                return Err(format!(
                    "{:?} is already been used with token {:?}",
                    keyword, token
                ));
            };
        }
        Ok(())
    }
}

impl<TLexer: ILexeme> LexemeLogger for Mapper<TLexer> {
    fn log_cell(&self) -> &OnceCell<crate::util::Log<&'static str>> {
        &self.log
    }
}
impl<TLexer: ILexeme> ILexeme for Mapper<TLexer> {
    type Token = TLexer::Token;
    type State = TLexer::State;

    fn consume(
        &self,
        code: &Code,
        pointer: usize,
        tokenized_stream: &Vec<Lex<Self::Token>>,
        info: &mut Vec<Self::State>,
    ) -> Option<Lex<Self::Token>> {
        let result = self.lexeme.consume(code, pointer, tokenized_stream, info);
        self.log_result(pointer, code, &result);

        result.map(|mut lex_data| {
            let code_part = &code.value[lex_data.start..lex_data.end];
            if let Some(token) = self.fields.get(code_part) {
                lex_data.token = *token;
            }
            lex_data
        })
    }
    fn get_grammar_field(&self) -> Vec<(<TLexer as ILexeme>::Token, String)> {
        let mut v: Vec<(TLexer::Token, String)> = self
            .fields
            .iter()
            .map(|(s, t)| {
                let s = unsafe { std::str::from_utf8_unchecked(s) };

                (*t, format!("{:?}", s))
            })
            .collect();

        v.extend(self.lexeme.get_grammar_field().into_iter());
        v
    }
}

impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Option<TL::Token>>
    ThunkMapper<TL, TF>
{
    pub fn new(lexeme: TL, thunk: TF) -> Self {
        Self {
            lexeme,
            thunk,
            log: OnceCell::new(),
        }
    }
    pub fn set_log(&self, log: Log<&'static str>) -> Result<(), String> {
        self.log
            .set(log)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }
}

impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Option<TL::Token>>
    LexemeLogger for ThunkMapper<TL, TF>
{
    fn log_cell(&self) -> &OnceCell<crate::util::Log<&'static str>> {
        &self.log
    }
}
impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Option<TL::Token>> ILexeme
    for ThunkMapper<TL, TF>
{
    type Token = TL::Token;
    type State = TL::State;

    fn consume(
        &self,
        code: &Code,
        pointer: usize,
        tokenized_stream: &Vec<Lex<Self::Token>>,
        info: &mut Vec<Self::State>,
    ) -> Option<Lex<Self::Token>> {
        let result = self.lexeme.consume(code, pointer, tokenized_stream, info);
        self.log_result(pointer, code, &result);
        result.map(|mut lex| {
            match (self.thunk)(&lex, &code.value, tokenized_stream) {
                Some(token) => {
                    lex.token = token;
                }
                None => {}
            };
            lex
        })
    }

    fn get_grammar_field(&self) -> Vec<(TL::Token, String)> {
        self.lexeme.get_grammar_field()
    }
}
