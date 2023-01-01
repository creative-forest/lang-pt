use crate::{
    production::{NullProd, ProductionLogger},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, NodeImpl, ParsedResult, StreamPtr,
    SuccessData, TokenImpl, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

impl<TN: NodeImpl, TL: TokenImpl> NullProd<TN, TL> {
    /// Create a new NullProd.
    pub fn new() -> Self {
        NullProd {
            debugger: OnceCell::new(),
            _node: PhantomData,
            _token: PhantomData,
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> NullProd<TN, TL> {
    pub fn set_log(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for NullProd<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
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
        let pointer = token_stream.pointer(index);
        Ok(SuccessData::tree(
            index,
            ASTNode::null(pointer, Some(token_stream.get_stream_ptr(index))),
        ))
    }

    fn advance_token_ptr(
        &self,
        _: &Code,
        index: StreamPtr,
        token_stream: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        let pointer = token_stream[index].start;
        Ok(SuccessData::tree(
            index,
            ASTNode::null(pointer, Some(index)),
        ))
    }

    fn advance_ptr(
        &self,
        _: &Code,
        index: usize,
        _: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        Ok(SuccessData::tree(index, ASTNode::null(index, None)))
    }

    fn impl_grammar(
        &self,
        _: &mut dyn std::fmt::Write,
        _: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
