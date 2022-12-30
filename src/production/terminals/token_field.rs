use crate::{
    production::{ProductionLogger, TokenField, TokenFieldSet},
    util::{Code, Log},
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, NodeImpl, ParsedResult,
    ProductionError, StreamPtr, SuccessData, TokenImpl, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

/// A terminal symbol which matches a particular token.

impl<TN: NodeImpl, TL: TokenImpl> TokenField<TN, TL> {
    /// Create a new [TokenField].
    /// ## Arguments
    /// * `field` - Token field will be matched.
    /// * `node_value` - Optional node value of [ASTNode](crate::ASTNode).
    /// [Some] node_value will create a node in [AST](crate::ASTNode) while [None] value will hide the tree from [AST](crate::ASTNode).      

    pub fn new(token: TL, node_value: Option<TN>) -> Self {
        Self {
            token,
            node_value,
            debugger: OnceCell::new(),
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> TokenField<TN, TL> {
    pub fn set_log(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for TokenField<TN, TL> {
    fn get_debugger(&self) -> Option<&Log<&'static str>> {
        self.debugger.get()
    }
}
impl<TN: NodeImpl, TL: TokenImpl> Display for TokenField<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node_value {
            Some(node) => {
                write!(f, "[&{:?}; {:?}]", self.token, node)
            }
            None => {
                write!(f, "[&{:?}; ]", self.token,)
            }
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for TokenField<TN, TL> {
    type Node = TN;
    type Token = TL;
    fn is_nullable(&self) -> bool {
        false
    }

    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<Self::Token>) {
        first_set.insert(self.token);
    }

    fn eat_fltr_ptr(
        &self,
        _code: &Code,
        index: FltrPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        if self.token == stream[index].token {
            cache.update_index(stream[index].end);

            #[cfg(debug_assertions)]
            self.log_success(_code, stream[index].start, stream[index].end);

            match &self.node_value {
                Some(node) => {
                    let bound_start = stream.get_stream_ptr(index);

                    Ok(SuccessData::tree(
                        index + 1,
                        ASTNode::leaf(
                            node.clone(),
                            stream[index].start,
                            stream[index].end,
                            Some((bound_start, bound_start + 1)),
                        ),
                    ))
                }
                None => Ok(SuccessData::hidden(index + 1)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(_code, stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn eat_token_ptr(
        &self,
        _code: &Code,
        index: StreamPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        if self.token == stream[index].token {
            cache.update_index(stream[index].end);

            #[cfg(debug_assertions)]
            self.log_success(_code, stream[index].start, stream[index].end);

            match &self.node_value {
                Some(node) => Ok(SuccessData::tree(
                    index + 1,
                    ASTNode::leaf(
                        node.clone(),
                        stream[index].start,
                        stream[index].end,
                        Some((index, index + 1)),
                    ),
                )),
                None => Ok(SuccessData::hidden(index + 1)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(_code, stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn eat_ptr(
        &self,
        _: &Code,
        _: usize,
        _: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        panic!("Bug! TokenTerminal should not be called with lexer-less parsing.")
    }

    fn is_nullable_n_hidden(&self) -> bool {
        false
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(false)
    }

    fn impl_grammar(
        &self,
        _: &mut dyn std::fmt::Write,
        _: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        Ok(())
    }

    fn validate<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
        _: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        Ok(())
    }
}

impl<TN: NodeImpl, TL: TokenImpl> TokenFieldSet<TN, TL> {
    /// Create a new TokenListTerminal.
    /// ## Arguments
    /// * `token_set` - A set of tuples of token which will be matched with the input tokens and optional node value.
    /// Provided [Some] node value will create a node in [AST](crate::ASTNode) while [None] value will hide the tree from [AST](crate::ASTNode).   

    pub fn new(mut token_set: Vec<(TL, Option<TN>)>) -> Self {
        token_set.sort_by(|t1, t2| t1.0.cmp(&t2.0));

        Self {
            token_set,
            debugger: OnceCell::new(),
            rule_name: OnceCell::new(),
        }
    }

    fn semantics(&self) -> Vec<String> {
        self.token_set
            .iter()
            .map(|(token, node_value)| match node_value {
                Some(n) => format!("[&{:?}; {:?}]", token, n),
                None => format!("[&{:?}; ]", token),
            })
            .collect()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> TokenFieldSet<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for TokenFieldSet<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(rule_name) => {
                write!(f, "{}", rule_name)
            }
            None => {
                write!(f, "({})", self.semantics().join("|"))
            }
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for TokenFieldSet<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for TokenFieldSet<TN, TL> {
    type Token = TL;
    type Node = TN;
    fn is_nullable(&self) -> bool {
        false
    }

    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<TL>) {
        first_set.extend(self.token_set.iter().map(|(t, _)| t));
    }

    fn eat_fltr_ptr(
        &self,
        _code: &Code,
        index: FltrPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();
        match self
            .token_set
            .binary_search_by_key(&stream[index].token, |(t, _)| *t)
        {
            Ok(i) => {
                cache.update_index(stream[index].end);

                #[cfg(debug_assertions)]
                self.log_success(_code, stream[index].start, stream[index].end);

                match &self.token_set[i].1 {
                    Some(node) => {
                        let bound_start = stream.get_stream_ptr(index);

                        Ok(SuccessData::tree(
                            index + 1,
                            ASTNode::leaf(
                                node.clone(),
                                stream[index].start,
                                stream[index].end,
                                Some((bound_start, bound_start + 1)),
                            ),
                        ))
                    }
                    None => Ok(SuccessData::hidden(index + 1)),
                }
            }
            Err(_) => {
                #[cfg(debug_assertions)]
                self.log_error(_code, stream[index].start, &ProductionError::Unparsed);

                Err(ProductionError::Unparsed)
            }
        }
    }

    fn eat_token_ptr(
        &self,
        _code: &Code,
        index: StreamPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        match self
            .token_set
            .binary_search_by_key(&stream[index].token, |(t, _)| *t)
        {
            Ok(i) => {
                cache.update_index(stream[index].end);

                #[cfg(debug_assertions)]
                self.log_success(_code, stream[index].start, stream[index].end);

                match &self.token_set[i].1 {
                    Some(node) => Ok(SuccessData::tree(
                        index + 1,
                        ASTNode::leaf(
                            node.clone(),
                            stream[index].start,
                            stream[index].end,
                            Some((index, index + 1)),
                        ),
                    )),
                    None => todo!(),
                }
            }
            Err(_) => {
                #[cfg(debug_assertions)]
                self.log_error(_code, stream[index].start, &ProductionError::Unparsed);
                Err(ProductionError::Unparsed)
            }
        }
    }

    fn eat_ptr(
        &self,
        _: &Code,
        _: usize,
        _: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        panic!("Bug! TokenListTerminal should not be called with lexer-less parsing.")
    }

    fn is_nullable_n_hidden(&self) -> bool {
        false
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(false)
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        match self.rule_name.get() {
            Some(rule_name) => {
                if visited.insert(rule_name) {
                    writeln!(writer, "{}", rule_name)?;
                    writeln!(
                        writer,
                        "{:>6} {}",
                        ":",
                        self.semantics().join(&format!("\n{:>6}", "|"))
                    )?;
                    writeln!(writer, "{:>6}", ";")?;
                }
            }
            None => {}
        }
        Ok(())
    }

    fn validate<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
        _: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        Ok(())
    }
}
