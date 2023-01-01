use crate::util::Code;
use crate::{
    Cache, CacheKey, FltrPtr, NodeImpl, ParseError, ParsedResult, ProductionError, TokenImpl,
    TokenStream,
};
use std::fmt::Write;
use std::{collections::HashMap, hash::Hash};

impl<TP: Default + Eq + Hash + Ord + Copy, TToken> Cache<TP, TToken> {
    pub fn root() -> Self {
        Self {
            parsed_result_cache: HashMap::new(),
            max_parsed_point: 0,
        }
    }

    #[cfg(debug_assertions)]
    pub fn debug_new(starting_point: usize) -> Self {
        Self {
            parsed_result_cache: HashMap::new(),
            max_parsed_point: starting_point,
        }
    }

    pub fn contains(&self, key: CacheKey, index: usize) -> bool {
        self.parsed_result_cache.contains_key(&(key, index))
    }

    pub fn find(&self, key: CacheKey, index: usize) -> Option<&ParsedResult<TP, TToken>> {
        if index <= self.max_parsed_point {
            self.parsed_result_cache.get(&(key, index))
        } else {
            None
        }
    }

    pub fn insert(
        &mut self,
        key: CacheKey,
        index: usize,
        result: ParsedResult<TP, TToken>,
    ) -> Option<ParsedResult<TP, TToken>> {
        self.max_parsed_point = std::cmp::max(index, self.max_parsed_point);
        self.parsed_result_cache.insert((key, index), result)
    }

    pub fn update_index(&mut self, index: usize) {
        if self.max_parsed_point < index {
            self.max_parsed_point = index;
        }
    }

    pub fn get_index(&self) -> usize {
        self.max_parsed_point
    }
}

impl<TNode: NodeImpl> Cache<FltrPtr, TNode> {
    pub fn create_error<'lex, TL: TokenImpl>(
        &self,
        code: &Code,
        stream: &TokenStream<'lex, TL>,
        err: ProductionError,
    ) -> ParseError {
        let mut error_message = String::new();
        let pointer = match err {
            ProductionError::Unparsed => {
                let failed_index = match stream.filtered_index_at(self.max_parsed_point) {
                    Ok(i) => i + 1,
                    Err(i) => i,
                };

                match stream.get(failed_index) {
                    Some(lex_data) => {
                        if lex_data.token == TL::eof() {
                            writeln!(error_message, "Unexpected end of file.").unwrap();
                        } else {
                            let s = unsafe {
                                std::str::from_utf8_unchecked(
                                    &code.value[lex_data.start..lex_data.end],
                                )
                            };
                            if cfg!(debug_assertions) {
                                writeln!(
                                    error_message,
                                    "Unexpected token {:?}({:?}).",
                                    s, lex_data.token,
                                )
                                .unwrap();
                            } else {
                                writeln!(error_message, "Unexpected {:?}.", s).unwrap();
                            }
                        }
                        lex_data.start
                    }
                    None => {
                        writeln!(error_message, "Unexpected end of file.").unwrap();
                        code.value.len()
                    }
                }
            }
            ProductionError::Validation(pointer, message) => {
                writeln!(error_message, "{}", message).unwrap();
                pointer
            }
        };

        let position = code.obtain_position(pointer);

        writeln!(error_message, "Failed to parse at {}.", position).unwrap();

        ParseError::new(pointer, error_message)
    }
}
impl<TToken> Cache<usize, TToken> {
    pub fn create_error(&self, code: &Code, err: ProductionError) -> ParseError {
        let (pointer, mut error_message) = match err {
            ProductionError::Unparsed => {
                if self.get_index() == code.value.len() {
                    (self.get_index(), format!("Unexpected end of file."))
                } else {
                    let failed_index = self.get_index();
                    (
                        failed_index,
                        format!("Unexpected '{}'.", unsafe {
                            std::str::from_utf8_unchecked(
                                &code.value[failed_index..failed_index + 1],
                            )
                        },),
                    )
                }
            }
            ProductionError::Validation(pointer, message) => (pointer, message),
        };

        let position = code.obtain_position(pointer);

        writeln!(error_message, "\nFailed to parse at {}.", position).unwrap();

        ParseError::new(pointer, error_message)
    }
}
