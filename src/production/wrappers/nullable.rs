use crate::NodeImpl;
use crate::{
    production::{Nullable, ProductionLogger},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, SuccessData, TokenPtr,
    TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Nullable<TProd> {
    /// Create a nullable production i.e. add a null production as alternative production.
    ///
    /// The null derivation create a tree with [null](NodeImpl::null) node value in the [ASTNode].
    /// ## Arguments
    /// * 'symbol' - A terminal or non terminal symbol.
    pub fn new(symbol: &Rc<TProd>) -> Self {
        Self {
            symbol: symbol.clone(),
            debugger: OnceCell::new(),
            node_value: Some(TProd::Node::null()),
        }
    }
    /// Create a nullable production.
    ///
    /// The null derivation does not create any tree node in the [ASTNode].
    /// ## Arguments
    /// * 'symbol' - A terminal or non terminal symbol.
    pub fn hidden(production: &Rc<TProd>) -> Self {
        Self {
            symbol: production.clone(),
            debugger: OnceCell::new(),
            node_value: None,
        }
    }

    #[inline]
    pub fn get_production(&self) -> &TProd {
        &self.symbol
    }
}

impl<TP: IProduction> Nullable<TP> {
    pub fn set_log(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Nullable<TProd> {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for Nullable<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})?", self.get_production())
    }
}
impl<TProd: IProduction> IProduction for Nullable<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        true
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        self.get_production().impl_grammar(writer, visited)
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(true)
    }
    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<TProd::Token>) {
        self.get_production().impl_first_set(first_set);
    }

    fn is_nullable_n_hidden(&self) -> bool {
        false
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
        let result = self
            .get_production()
            .advance_fltr_ptr(code, index, token_stream, cached)
            .or_else(|err| {
                if err.is_invalid() {
                    Err(err)
                } else {
                    match &self.node_value {
                        Some(node_value) => {
                            let pointer_start = token_stream.pointer(index);
                            let bound = token_stream.get_token_ptr(index);
                            let tree = ASTNode::leaf(
                                node_value.clone(),
                                pointer_start,
                                pointer_start,
                                Some((bound, bound)),
                            );
                            Ok(SuccessData::tree(index, tree))
                        }
                        None => Ok(SuccessData::hidden(index)),
                    }
                }
            });

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);
        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        token_ptr: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        let result = self
            .get_production()
            .advance_token_ptr(code, token_ptr, token_stream, cache)
            .or_else(|err| {
                if err.is_invalid() {
                    Err(err)
                } else {
                    match &self.node_value {
                        Some(node_value) => {
                            let pointer_start = token_stream[token_ptr].start;
                            let tree = ASTNode::leaf(
                                node_value.clone(),
                                pointer_start,
                                pointer_start,
                                Some((token_ptr, token_ptr)),
                            );
                            Ok(SuccessData::tree(token_ptr, tree))
                        }
                        None => Ok(SuccessData::hidden(token_ptr)),
                    }
                }
            });

        #[cfg(debug_assertions)]
        self.log_lex_result(code, token_ptr, token_stream, &result);
        result
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        let result = self
            .get_production()
            .advance_ptr(code, index, cache)
            .or_else(|err| {
                if err.is_invalid() {
                    Err(err)
                } else {
                    match &self.node_value {
                        Some(node_value) => {
                            let tree = ASTNode::leaf(node_value.clone(), index, index, None);
                            Ok(SuccessData::tree(index, tree))
                        }
                        None => Ok(SuccessData::hidden(index)),
                    }
                }
            });
        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);
        result
    }
}
