use crate::production::ProductionLogger;
use crate::{production::NonStructural, util::Code, Cache, FltrPtr, IProduction, TokenStream};
use crate::{ImplementationError, ParsedResult, ProductionError, TokenPtr, SuccessData};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> NonStructural<TProd> {

    /// Create a new [NonStructural] production utility.
    /// ### Arguments 
    /// * `symbol` - A terminal or non terminal symbol.
    /// * `shall_fill_range` - A [bool] value to indicate whether it is required to consume all non structural tokens.
    pub fn new(symbol: &Rc<TProd>, shall_fill_range: bool) -> Self {
        Self {
            production: symbol.clone(),
            fill_range: shall_fill_range,
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    pub fn get_symbol(&self) -> &TProd {
        &self.production
    }
}

impl<TP: IProduction> NonStructural<TP> {
    /// Set a [Log](crate::util::Log) to debug the production.
    pub fn set_log(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for NonStructural<TProd> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for NonStructural<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}%", self.get_symbol())
    }
}
impl<TProd: IProduction> IProduction for NonStructural<TProd> {
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
        #[cfg(debug_assertions)]
        self.log_entry();

        let start_segment = if fltr_ptr > FltrPtr::default() {
            token_stream.get_stream_ptr(fltr_ptr - 1)
        } else {
            TokenPtr::default()
        };

        let parsed_data =
            self.get_symbol()
                .advance_token_ptr(code, start_segment + 1, token_stream, cache)?;

        let result = if self.fill_range {
            let end_segment = token_stream.get_stream_ptr(fltr_ptr);
            if end_segment == parsed_data.consumed_index {
                Ok(SuccessData::new(fltr_ptr, parsed_data.children))
            } else {
                Err(ProductionError::Unparsed)
            }
        } else {
            Ok(SuccessData::new(fltr_ptr, parsed_data.children))
        };

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, fltr_ptr, token_stream, &result);

        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        self.get_symbol()
            .advance_token_ptr(code, index, stream, cache)
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
