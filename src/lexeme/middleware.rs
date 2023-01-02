use super::{LexemeLogger, Middleware};
use crate::{
    Code, Log,
    ILexeme, Lex,
};
use once_cell::unsync::OnceCell;

impl<TS: ILexeme, TMiddleware: Fn(&[u8], &Vec<Lex<TS::Token>>) -> bool>
    Middleware<TS, TMiddleware>
{
    /// Create a new [Middleware] utility.
    /// ## Arguments
    /// * 'lexeme' - A lexer utility which implement [ILexeme] trait.
    /// * 'middleware' - A closure [Fn] which receive immutable [Code] and token stream data as arguments and return [bool] value.
    pub fn new(lexeme: TS, middleware: TMiddleware) -> Self {
        Self {
            lexeme,
            middleware,
            log_label: OnceCell::new(),
        }
    }

    /// Set a log label to debug the lexeme.
    /// Based on the level of the [Log], the lexeme will debug the lexeme result.
    pub fn set_log(&self, log: Log<&'static str>) -> Result<(), String> {
        self.log_label
            .set(log)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }
}

impl<TL: ILexeme, TMiddleware: Fn(&[u8], &Vec<Lex<TL::Token>>) -> bool> LexemeLogger
    for Middleware<TL, TMiddleware>
{
    fn log_cell(&self) -> &OnceCell<crate::Log<&'static str>> {
        &self.log_label
    }
}
impl<TL: ILexeme, TMiddleware: Fn(&[u8], &Vec<Lex<TL::Token>>) -> bool> ILexeme
    for Middleware<TL, TMiddleware>
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
        #[cfg(debug_assertions)]
        self.log_enter();
        if (self.middleware)(&code.value, tokenized_stream) {
            let result = self.lexeme.consume(code, pointer, tokenized_stream, info);
            #[cfg(debug_assertions)]
            self.log_result(pointer, code, &result);
            result
        } else {
            None
        }
    }

    fn get_grammar_field(&self) -> Vec<(TL::Token, String)> {
        self.lexeme.get_grammar_field()
    }
}
