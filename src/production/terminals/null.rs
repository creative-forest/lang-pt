use crate::{
    production::{NullProd, ProductionLogger},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, NodeImpl, ParsedResult, SuccessData,
    TokenImpl, TokenPtr, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

impl<TN: NodeImpl, TL: TokenImpl> NullProd<TN, TL> {
    /// Create a new NullProd.
    ///
    /// The null derivation create a tree node with [null][NodeImpl::null] value in the [ASTNode].
    pub fn new() -> Self {
        NullProd {
            debugger: OnceCell::new(),
            node_value: Some(TN::null()),
            _token: PhantomData,
        }
    }

    /// Create a new NullProd.
    ///
    /// The null derivation does not create any tree node in the [ASTNode].
    pub fn hidden() -> Self {
        NullProd {
            debugger: OnceCell::new(),
            node_value: None,
            _token: PhantomData,
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> NullProd<TN, TL> {
    pub fn set_log(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for NullProd<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for NullProd<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "''")
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for NullProd<TN, TL> {
    type Node = TN;
    type Token = TL;

    fn is_nullable(&self) -> bool {
        true
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(true)
    }

    fn impl_first_set<'prod>(&'prod self, _: &mut HashSet<Self::Token>) {}
    fn is_nullable_n_hidden(&self) -> bool {
        false
    }

    fn validate(
        &self,
        _: HashMap<&str, usize>,
        _: &mut HashSet<&str>,
    ) -> Result<(), ImplementationError> {
        Result::Ok(())
    }

    fn advance_fltr_ptr(
        &self,
        _: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        match &self.node_value {
            Some(node_value) => {
                let pointer = token_stream.pointer(index);
                let lex_bound = token_stream.get_token_ptr(index);
                Ok(SuccessData::tree(
                    index,
                    ASTNode::leaf(
                        node_value.clone(),
                        pointer,
                        pointer,
                        Some((lex_bound, lex_bound)),
                    ),
                ))
            }
            None => Ok(SuccessData::hidden(index)),
        }
    }

    fn advance_token_ptr(
        &self,
        _: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        match &self.node_value {
            Some(node_value) => {
                let pointer = token_stream[index].start;
                Ok(SuccessData::tree(
                    index,
                    ASTNode::leaf(node_value.clone(), pointer, pointer, Some((index, index))),
                ))
            }
            None => Ok(SuccessData::hidden(index)),
        }
    }

    fn advance_ptr(
        &self,
        _: &Code,
        index: usize,
        _: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        match &self.node_value {
            Some(node_value) => Ok(SuccessData::tree(
                index,
                ASTNode::leaf(node_value.clone(), index, index, None),
            )),
            None => Ok(SuccessData::hidden(index)),
        }
    }

    fn impl_grammar(
        &self,
        _: &mut dyn std::fmt::Write,
        _: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
