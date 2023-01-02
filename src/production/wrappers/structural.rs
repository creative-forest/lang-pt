use crate::production::ProductionLogger;
use crate::{production::Structural, Cache, Code, FltrPtr, IProduction, TokenStream};
use crate::{ImplementationError, ParsedResult, SuccessData, TokenPtr};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Structural<TProd> {
    /// Create a new [Structural] production utility.
    /// ### Arguments
    /// * `symbol` - A terminal or non terminal symbol.
    pub fn new(symbol: &Rc<TProd>) -> Self {
        Self {
            production: symbol.clone(),
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    pub fn get_symbol(&self) -> &TProd {
        &self.production
    }
}

impl<TP: IProduction> Structural<TP> {
    /// Set a [Log](crate::Log) to debug the production.
    pub fn set_log(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Structural<TProd> {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for Structural<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}%", self.get_symbol())
    }
}
impl<TProd: IProduction> IProduction for Structural<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        self.get_symbol().is_nullable()
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        self.production.impl_grammar(writer, visited)
    }

    fn obtain_nullability<'id>(
        &'id self,
        visited: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        self.production.obtain_nullability(visited)
    }
    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<TProd::Token>) {
        self.production.impl_first_set(first_set)
    }
    fn is_nullable_n_hidden(&self) -> bool {
        self.production.is_nullable()
    }

    #[inline]
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
        fltr_ptr: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        let result = self
            .get_symbol()
            .advance_fltr_ptr(code, fltr_ptr, token_stream, cache);

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, fltr_ptr, token_stream, &result);

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

        let next_fltr_ptr = match token_stream.find_filter_ptr(index) {
            Ok(fp) => fp + 1,
            Err(fp) => fp,
        };

        let parsed_data =
            self.get_symbol()
                .advance_fltr_ptr(code, next_fltr_ptr, token_stream, cache)?;

        #[cfg(debug_assertions)]
        self.log_success(
            code,
            token_stream[index].start,
            token_stream[parsed_data.consumed_index].start,
        );

        let last_token_ptr = if parsed_data.consumed_index > FltrPtr::default() {
            token_stream.get_token_ptr(parsed_data.consumed_index - 1)
        } else {
            TokenPtr::default()
        };

        Ok(SuccessData::new(last_token_ptr + 1, parsed_data.children))
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        self.get_symbol().advance_ptr(code, index, cache)
    }
}
