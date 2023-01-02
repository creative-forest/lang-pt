use crate::{Code, ILexeme, Lex, Log};
use once_cell::unsync::OnceCell;
use std::fmt::Debug;

use super::{Action, LexemeLogger, StateMixin, ThunkStateMixin};

fn perform_state_action<TToken, TState: PartialEq + Debug>(
    lexical_data: Lex<TToken>,
    action: Action<TState>,
    state_stack: &mut Vec<TState>,
    pointer: usize,
    code: &Code,
) -> Option<Lex<TToken>> {
    let shall_discard = match action {
        Action::Pop { discard } => match state_stack.pop() {
            Some(_) => discard,
            None => {
                panic!(
                    "Failed to remove a state from empty state stack at {} ({}).",
                    pointer,
                    code.obtain_position(pointer)
                )
            }
        },
        Action::Append { state, discard } => {
            state_stack.push(state);
            discard
        }
        Action::Switch { state, discard } => {
            match state_stack.last_mut() {
                Some(last_state) => {
                    *last_state = state;
                }
                None => state_stack.push(state),
            }
            discard
        }
        Action::None { discard } => discard,
    };
    if shall_discard {
        None
    } else {
        Some(lexical_data)
    }
}

impl<TL: ILexeme> StateMixin<TL> {
    /// Create a new [StateMixin] utility.
    /// ## Arguments
    /// * 'lexeme' - A lexer utility which implement [ILexeme].
    /// * 'middleware' - A [Vec] of tuples of token and corresponding change state stack [Action].

    pub fn new(lexeme: TL, mut actions: Vec<(TL::Token, Action<TL::State>)>) -> Self {
        actions.sort_by_key(|(t, _)| *t);

        Self {
            lexeme,
            actions,
            log: OnceCell::new(),
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
impl<TL: ILexeme> LexemeLogger for StateMixin<TL> {
    fn log_cell(&self) -> &OnceCell<crate::Log<&'static str>> {
        &self.log
    }
}
impl<TL: ILexeme> ILexeme for StateMixin<TL> {
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
        match result {
            Some(lexical_data) => {
                match self
                    .actions
                    .binary_search_by_key(&lexical_data.token, |(t, _)| *t)
                {
                    Ok(index) => perform_state_action(
                        lexical_data,
                        self.actions[index].1,
                        info,
                        pointer,
                        code,
                    ),
                    Err(_) => Some(lexical_data),
                }
            }
            None => None,
        }
    }

    fn get_grammar_field(&self) -> Vec<(TL::Token, String)> {
        self.lexeme.get_grammar_field()
    }
}

impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Action<TL::State>>
    ThunkStateMixin<TL, TF>
{
    /// Create a new [ThunkStateMixin] utility.
    /// ## Arguments
    /// * 'lexeme' - A lexer utility which implement [ILexeme].
    /// * 'thunk_action' - A closure [Fn] which takes tokenize [Lex] from the lexeme utility,
    /// [Code], and tokenize stream data.

    pub fn new(lexeme: TL, thunk_action: TF) -> Self {
        Self {
            lexeme,
            log: OnceCell::new(),
            thunk_action,
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

impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Action<TL::State>>
    LexemeLogger for ThunkStateMixin<TL, TF>
{
    fn log_cell(&self) -> &OnceCell<crate::Log<&'static str>> {
        &self.log
    }
}
impl<TL: ILexeme, TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Action<TL::State>> ILexeme
    for ThunkStateMixin<TL, TF>
{
    type Token = TL::Token;
    type State = TL::State;

    fn consume(
        &self,
        code: &Code,
        pointer: usize,
        tokenized_stream: &Vec<Lex<Self::Token>>,
        state_stack: &mut Vec<Self::State>,
    ) -> Option<Lex<Self::Token>> {
        // console_log!("Regex pointer {}", pointer);
        let result = self
            .lexeme
            .consume(code, pointer, tokenized_stream, state_stack);
        self.log_result(pointer, code, &result);
        match result {
            Some(lexical_data) => {
                let action = (self.thunk_action)(&lexical_data, &code.value, tokenized_stream);

                perform_state_action(lexical_data, action, state_stack, pointer, code)
            }
            None => None,
        }
    }

    fn get_grammar_field(&self) -> Vec<(TL::Token, String)> {
        self.lexeme.get_grammar_field()
    }
}
