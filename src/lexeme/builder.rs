use super::{Action, LexemeBuilder, Mapper, Middleware, StateMixin, ThunkMapper, ThunkStateMixin};
use crate::{ILexeme, Lex};

impl<T: ILexeme> LexemeBuilder for T {
    fn mapping(self, fields: Vec<(&str, Self::Token)>) -> Result<Mapper<Self>, String>
    where
        Self: Sized,
    {
        Mapper::new(self, fields)
    }

    fn state_mixin(self, actions: Vec<(Self::Token, Action<Self::State>)>) -> StateMixin<Self>
    where
        Self: Sized,
    {
        StateMixin::new(self, actions)
    }

    fn middleware<TM: Fn(&[u8], &Vec<Lex<Self::Token>>) -> bool>(
        self,
        middleware: TM,
    ) -> Middleware<Self, TM>
    where
        Self: Sized,
    {
        Middleware::new(self, middleware)
    }

    fn thunk_mixin<
        TM: Fn(&Lex<Self::Token>, &[u8], &Vec<Lex<Self::Token>>) -> Action<Self::State>,
    >(
        self,
        thunk: TM,
    ) -> ThunkStateMixin<Self, TM>
    where
        Self: Sized,
    {
        ThunkStateMixin::new(self, thunk)
    }

    fn thunk_mapping<
        TF: Fn(&Lex<Self::Token>, &[u8], &Vec<Lex<Self::Token>>) -> Option<Self::Token>,
    >(
        self,
        thunk: TF,
    ) -> super::ThunkMapper<Self, TF>
    where
        Self: Sized,
    {
        ThunkMapper::new(self, thunk)
    }
}
