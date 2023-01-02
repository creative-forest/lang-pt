use crate::Code;
use crate::{CombinedTokenizer, ILexeme, Log, TokenImpl, Tokenizer};
use crate::{ITokenization, Lex, ParseError};
use once_cell::unsync::OnceCell;
use std::fmt::Debug;
use std::fmt::Write;
use std::rc::Rc;

impl<TToken> Tokenizer<TToken, u8> {
    pub fn new(lexers: Vec<Rc<dyn ILexeme<Token = TToken, State = u8>>>) -> Self {
        Self { lexers }
    }
}

impl<TT, TS: Ord + Eq + Copy> CombinedTokenizer<TT, TS> {
    pub fn new(default_state: TS, lexemes: Vec<Rc<dyn ILexeme<Token = TT, State = TS>>>) -> Self {
        Self {
            analyzers: vec![(default_state, lexemes)],
            default_state,
            debug: OnceCell::new(),
        }
    }

    pub fn add_state(&mut self, state: TS, lexemes: Vec<Rc<dyn ILexeme<Token = TT, State = TS>>>) {
        let index = match self.analyzers.binary_search_by_key(&state, |a| a.0) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        self.analyzers.insert(index, (state, lexemes))
    }

    pub fn set_log(&mut self, log_label: Log<&'static str>) -> Result<(), String> {
        self.debug
            .set(log_label)
            .map_err(|err| format!("Log label {} is already assigned.", err))
    }
}

impl<TToken: TokenImpl, TState: Copy + Debug + Ord + Eq> ITokenization
    for CombinedTokenizer<TToken, TState>
{
    type Token = TToken;
    /// Tokenize the code and return result consisting of vec of tokenize stream.
    fn tokenize(&self, code: &Code) -> Result<Vec<Lex<TToken>>, ParseError> {
        let mut tokenized_stream: Vec<Lex<TToken>> = Vec::new();
        let mut pointer: usize = 0;
        let eof_pointer: usize = code.value.len();

        let mut state_stack = Vec::<TState>::new();
        let mut current_state = self.default_state;
        let mut current_analyzer = match self
            .analyzers
            .binary_search_by_key(&&current_state, |(b, _)| b)
        {
            Ok(index) => &self.analyzers[index],
            Err(_) => panic!("TokenizationState '{:?}' is not implemented", current_state),
        };

        #[cfg(debug_assertions)]
        let debug = self.debug.get().map_or(Log::None, |s| s.clone());

        #[cfg(debug_assertions)]
        if debug.order() >= Log::Verbose(()).order() {
            println!("Begin tokenization for state: {:?}", current_state);
        }

        loop {
            match current_analyzer
                .1
                .iter()
                .find_map(|lexer| lexer.consume(code, pointer, &tokenized_stream, &mut state_stack))
            {
                Some(lex_data) => {
                    debug_assert_eq!(pointer, lex_data.start);
                    pointer = lex_data.end;

                    tokenized_stream.push(lex_data);

                    if pointer == eof_pointer {
                        #[cfg(debug_assertions)]
                        if debug.order() >= Log::Success(()).order() {
                            println!("[{}; Tokenization success]", debug);
                        }
                        let eof_token = TToken::eof();

                        tokenized_stream.push(Lex::new(eof_token, eof_pointer, eof_pointer));
                        break Ok(tokenized_stream);
                    }
                }
                None => {
                    #[cfg(debug_assertions)]
                    if debug.order() >= Log::Default(()).order() {
                        println!(
                            "{}: Tokenization failed in state {:?} at {}",
                            debug,
                            current_state,
                            code.obtain_position(pointer)
                        );
                    }
                    break Err(ParseError::new(
                        pointer,
                        format!(
                            "Failed to tokenize code @ {}",
                            code.obtain_position(pointer)
                        ),
                    ));
                }
            }

            let latest_state = state_stack.last().map_or(self.default_state, |s| s.clone());
            if latest_state != current_state {
                current_analyzer = match self
                    .analyzers
                    .binary_search_by_key(&latest_state, |(b, _)| *b)
                {
                    Ok(index) => &self.analyzers[index],
                    Err(_) => panic!("Tokenize state '{:?}' not implemented", current_state),
                };
                #[cfg(debug_assertions)]
                if debug.order() >= Log::Default(()).order() {
                    println!(
                        "{} : Switching state {:?} -> {:?} at {}",
                        debug,
                        current_state,
                        latest_state,
                        code.obtain_position(pointer)
                    );
                }

                current_state = latest_state;
            }
        }
    }

    fn build_grammar(&self) -> Result<String, std::fmt::Error> {
        let mut writer = String::new();
        for (state, lexers) in &self.analyzers {
            writeln!(writer, "fragment {:?} {{", state)?;

            for fields in lexers.iter().map(|l| l.get_grammar_field()) {
                for (t, s) in &fields {
                    writeln!(writer, "{:>6}{:?} : {} ,", "", t, s)?;
                }
            }
            writeln!(writer, "}}")?;
            writeln!(writer, "")?;
        }
        Ok(writer)
    }
}

impl<TToken: TokenImpl, TState: Copy + Debug + Default + Ord + Eq> ITokenization
    for Tokenizer<TToken, TState>
{
    type Token = TToken;
    /// Tokenize the code and return result consisting of vec of tokenize stream.
    fn tokenize(&self, code: &Code) -> Result<Vec<Lex<TToken>>, ParseError> {
        let mut tokenized_stream: Vec<Lex<TToken>> = Vec::new();
        let mut pointer: usize = 0;
        let eof_pointer: usize = code.value.len();

        let mut state_stack = Vec::new();

        loop {
            match self
                .lexers
                .iter()
                .find_map(|lexer| lexer.consume(code, pointer, &tokenized_stream, &mut state_stack))
            {
                Some(lex_data) => {
                    debug_assert_eq!(pointer, lex_data.start);
                    pointer = lex_data.end;

                    tokenized_stream.push(lex_data);

                    if pointer == eof_pointer {
                        let eof_token = TToken::eof();

                        tokenized_stream.push(Lex::new(eof_token, eof_pointer, eof_pointer));
                        break Ok(tokenized_stream);
                    }
                }
                None => {
                    break Err(ParseError::new(
                        pointer,
                        format!(
                            "Failed to tokenize code @ {}",
                            code.obtain_position(pointer)
                        ),
                    ));
                }
            }
        }
    }

    fn build_grammar(&self) -> Result<String, std::fmt::Error> {
        let mut writer = String::new();
        writeln!(writer, "fragment {{")?;
        for fields in self.lexers.iter().map(|l| l.get_grammar_field()) {
            for (t, s) in &fields {
                writeln!(writer, "{:>6}{:?} : {} ,", "", t, s)?;
            }
        }
        writeln!(writer, "}}")?;
        Ok(writer)
    }
}
