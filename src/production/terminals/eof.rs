use crate::{
    production::{EOFProd, ProductionLogger},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, NodeImpl, ParsedResult,
    ProductionError, StreamPtr, SuccessData, TokenImpl, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

impl<'field, TN: NodeImpl, TL: TokenImpl> EOFProd<TN, TL> {
    /// Create a new EOFProd.
    pub fn new(node: Option<TN>) -> Self {
        Self {
            node_value: node,
            debugger: OnceCell::new(),
            _token: PhantomData,
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> EOFProd<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for EOFProd<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for EOFProd<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EOF")
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for EOFProd<TN, TL> {
    type Node = TN;
    type Token = TL;

    fn is_nullable(&self) -> bool {
        false
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(false)
    }

    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<TL>) {
        first_set.insert(TL::eof());
    }
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
        code: &Code,
        index: FltrPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        if stream.is_eos(index) {
            let eof_pointer = stream.eos_pointer();
            cache.update_index(eof_pointer);

            #[cfg(debug_assertions)]
            self.log_success(code, eof_pointer, eof_pointer);
            match &self.node_value {
                Some(node_value) => {
                    let lex_index = stream.get_stream_ptr(index);
                    let tree = ASTNode::leaf(
                        node_value.clone(),
                        eof_pointer,
                        code.value.len(),
                        Some((lex_index, lex_index)),
                    );
                    Ok(SuccessData::tree(index, tree))
                }
                None => Ok(SuccessData::hidden(index)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(code, stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: StreamPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        if stream.is_eos_segment(index) {
            let eof_pointer = stream.eos_pointer();
            cache.update_index(eof_pointer);

            #[cfg(debug_assertions)]
            self.log_success(code, eof_pointer, eof_pointer);

            match &self.node_value {
                Some(node_value) => {
                    let tree = ASTNode::leaf(
                        node_value.clone(),
                        eof_pointer,
                        code.value.len(),
                        Some((index, index)),
                    );
                    Ok(SuccessData::tree(index, tree))
                }
                None => Ok(SuccessData::hidden(index)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(code, stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        if code.value.len() == index {
            cache.update_index(index);

            #[cfg(debug_assertions)]
            self.log_success(code, index, index);

            match &self.node_value {
                Some(node_value) => {
                    let tree = ASTNode::leaf(node_value.clone(), index, index, None);

                    Ok(SuccessData::tree(index, tree))
                }
                None => Ok(SuccessData::hidden(index)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(code, index, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
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
