use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

use once_cell::unsync::OnceCell;
use regex::bytes::Regex;

use crate::{
    production::{ProductionLogger, RegexField},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, NodeImpl, ParsedResult,
    ProductionError, TokenPtr, SuccessData, TokenImpl, TokenStream,
};

impl<TN: NodeImpl> RegexField<TN, i8> {
    pub fn new(regex_str: &str, node_value: Option<TN>) -> Result<Self, String> {
        match Regex::new(regex_str) {
            Ok(regexp) => Ok(Self {
                regexp,
                node_value,
                debugger: OnceCell::new(),
                _token: PhantomData,
                rule_name: OnceCell::new(),
            }),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> RegexField<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for RegexField<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for RegexField<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(s) => {
                write!(f, "{}", s)
            }
            None => {
                write!(f, "/{}/", self.regexp.as_str().replace('/', "\\/"))
            }
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for RegexField<TN, TL> {
    type Token = TL;
    type Node = TN;

    fn is_nullable(&self) -> bool {
        self.regexp.is_match(b"")
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.is_nullable() && self.node_value.is_none()
    }

    fn impl_first_set<'prod>(&'prod self, _: &mut HashSet<Self::Token>) {
        panic!("Bug! RegexField terminal is not expected with Token implementations");
    }

    fn advance_fltr_ptr(
        &self,
        _: &Code,
        _: FltrPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<crate::FltrPtr, Self::Node> {
        panic!("Bug! RegexTerminal should not used for tokenized parsing.")
    }

    fn advance_token_ptr(
        &self,
        _: &Code,
        _: TokenPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        panic!("Bug! RegexTerminal should not used for tokenized parsing.")
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        if let Some(m) = self.regexp.find(&code.value[index..]) {
            debug_assert!(
                m.start() == 0,
                "Regex expression should be match from beginning."
            );
            // let s = &code[pointer..consumed_ptr];
            let consumed_ptr = index + m.end();
            cache.update_index(consumed_ptr);

            #[cfg(debug_assertions)]
            self.log_success(code, index, consumed_ptr);

            match &self.node_value {
                Some(node_value) => {
                    let cached_tree: ASTNode<Self::Node> =
                        ASTNode::leaf(node_value.clone(), index, consumed_ptr, None);
                    Ok(SuccessData::tree(consumed_ptr, cached_tree))
                }
                None => Ok(SuccessData::hidden(consumed_ptr)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(code, index, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(self.regexp.is_match(b""))
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        match self.rule_name.get() {
            Some(s) => {
                if visited.insert(s) {
                    writeln!(writer, "{}", s)?;
                    let re_exp = self.regexp.as_str().replace('/', "\\/");
                    match &self.node_value {
                        Some(node) => {
                            writeln!(writer, "{:>6} [/{}/; {:?}]", ":", re_exp, node)?;
                        }
                        None => {
                            writeln!(writer, "{:>6} [/{}/; ]", ":", re_exp)?;
                        }
                    }
                }
            }
            None => {}
        }
        Ok(())
    }

    fn validate<'id>(
        &'id self,
        _: std::collections::HashMap<&'id str, usize>,
        _: &mut std::collections::HashSet<&'id str>,
    ) -> Result<(), crate::ImplementationError> {
        Ok(())
    }
}
