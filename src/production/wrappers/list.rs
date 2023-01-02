use crate::{
    production::{List, ProductionLogger},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, TokenPtr,
    SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> List<TProd> {
    /// Create a [List] production of the provided symbol.
    ///
    /// ### Arguments
    /// * `symbol` - A terminal or non-terminal symbol which will be parse one or multiple time.
    pub fn new(symbol: &Rc<TProd>) -> Self {
        Self {
            symbol: symbol.clone(),
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    /// Get the associated terminal or non-terminal symbol of the production.
    pub fn get_symbol(&self) -> &TProd {
        &self.symbol
    }

    fn consume<
        T: PartialEq + Copy,
        TCache,
        P: Fn(T, &mut Cache<TCache, TProd::Node>) -> ParsedResult<T, TProd::Node>,
    >(
        &self,
        index: T,
        cache: &mut Cache<TCache, TProd::Node>,
        parse_production: P,
    ) -> ParsedResult<T, TProd::Node> {
        let success_data = parse_production(index, cache)?;

        if success_data.consumed_index == index {
            return Ok(success_data);
        }

        let mut children: Vec<ASTNode<TProd::Node>> = success_data.children;
        let mut moved_ptr = success_data.consumed_index;
        loop {
            match parse_production(moved_ptr, cache) {
                Ok(next_success_data) => {
                    children.extend(next_success_data.children);
                    if moved_ptr == next_success_data.consumed_index {
                        break Ok(SuccessData::new(next_success_data.consumed_index, children));
                    }
                    moved_ptr = next_success_data.consumed_index;
                }
                Err(err) => {
                    if err.is_invalid() {
                        return Err(err);
                    } else {
                        break Ok(SuccessData::new(moved_ptr, children));
                    }
                }
            }
        }
    }
}

impl<TP: IProduction> List<TP> {
    pub fn assign_debugger(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for List<TProd> {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}
impl<TProd: IProduction> Display for List<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+", self.get_symbol())
    }
}
impl<TP: IProduction> IProduction for List<TP> {
    type Node = TP::Node;
    type Token = TP::Token;

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        self.get_symbol().impl_grammar(writer, visited)
    }

    fn validate<'id>(
        &'id self,
        first_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        self.get_symbol().validate(first_sets, visited_prod)
    }

    fn advance_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();
        let result = self.consume(index, cache, |moved_pointer, cache| {
            self.get_symbol()
                .advance_fltr_ptr(code, moved_pointer, token_stream, cache)
        });
        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);
        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();
        let result = self.consume(index, cache, |moved_pointer, cache| {
            self.get_symbol()
                .advance_token_ptr(code, moved_pointer, token_stream, cache)
        });
        #[cfg(debug_assertions)]
        self.log_lex_result(code, index, token_stream, &result);
        result
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();
        let result = self.consume(index, cache, |moved_pointer, cache| {
            self.get_symbol().advance_ptr(code, moved_pointer, cache)
        });

        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);

        result
    }

    fn is_nullable(&self) -> bool {
        self.get_symbol().is_nullable()
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.get_symbol().is_nullable_n_hidden()
    }

    fn obtain_nullability<'id>(
        &'id self,
        visited: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        self.get_symbol().obtain_nullability(visited)
    }

    fn impl_first_set(&self, first_set: &mut HashSet<Self::Token>) {
        self.get_symbol().impl_first_set(first_set)
    }
}
