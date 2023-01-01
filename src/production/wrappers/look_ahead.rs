use crate::{
    production::{Lookahead, ProductionLogger},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, StreamPtr,
    SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Lookahead<TProd> {
    /// Create a [Lookahead] production of the provided symbol.
    ///
    /// ### Arguments
    /// * `symbol` - A terminal or non-terminal symbol which will be parse without consuming input.
    /// * `node_value`-
    pub fn new(symbol: &Rc<TProd>, node_value: Option<TProd::Node>) -> Self {
        Self {
            // hidden,
            node_value,
            production: symbol.clone(),
            debugger: OnceCell::new(),
        }
    }
    #[inline]
    pub fn get_production(&self) -> &TProd {
        &self.production
    }
}

impl<TP: IProduction> Lookahead<TP> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Lookahead<TProd> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}
impl<TProd: IProduction> Display for Lookahead<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?={}", self.get_production())
    }
}
impl<TProd: IProduction> IProduction for Lookahead<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        false
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
        true
    }

    #[inline]
    fn validate<'id>(
        &'id self,
        first_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        self.get_production().validate(first_sets, visited_prod)
    }

    fn advance_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cached: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .advance_fltr_ptr(code, index, token_stream, cached)
            .map(|_| match &self.node_value {
                Some(node) => {
                    let pointer = token_stream[index].start;
                    let segment_index = token_stream.get_stream_ptr(index);
                    SuccessData::new(
                        index,
                        vec![ASTNode::new(
                            node.clone(),
                            pointer,
                            pointer,
                            Some((segment_index, segment_index)),
                            Vec::with_capacity(0),
                        )],
                    )
                }
                None => SuccessData::hidden(index),
            });

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);

        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: StreamPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        let result = self
            .get_production()
            .advance_token_ptr(code, index, token_stream, cache)
            .map(|_| match &self.node_value {
                Some(node) => {
                    let pointer = token_stream[index].start;
                    SuccessData::new(
                        index,
                        vec![ASTNode::new(
                            node.clone(),
                            pointer,
                            pointer,
                            Some((index, index)),
                            Vec::with_capacity(0),
                        )],
                    )
                }
                None => SuccessData::hidden(index),
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
        let result =
            self.get_production()
                .advance_ptr(code, index, cache)
                .map(|_| match &self.node_value {
                    Some(node) => SuccessData::new(
                        index,
                        vec![ASTNode::new(
                            node.clone(),
                            index,
                            index,
                            None,
                            Vec::with_capacity(0),
                        )],
                    ),
                    None => SuccessData::hidden(index),
                });

        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);

        result
    }
}
