use crate::{
    production::{ProductionLogger, PunctuationsField},
    util::Code,
    ASTNode, Cache, FieldTree, FltrPtr, IProduction, NodeImpl, ParsedResult, ProductionError,
    StreamPtr, SuccessData, TokenImpl, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

impl<TN: NodeImpl> PunctuationsField<TN, i8> {
    pub fn new(values: Vec<(&str, Option<TN>)>) -> Result<Self, String> {
        if values.len() == 0 {
            return Err(format!("Punctuation field set should not be empty."));
        }
        let mut field_tree = FieldTree::new();

        for (value, token) in &values {
            field_tree
                .insert(value.as_bytes(), token.clone())
                .map_err(|_| format!("Field {} has been used multiple times.", value))?;
        }

        let mut values: Vec<(String, Option<TN>)> = values
            .into_iter()
            .map(|(s, t)| (s.to_string(), t))
            .collect();
        values.sort_by_key(|(s, _)| s.len());

        Ok(Self {
            tree: field_tree,
            values,
            debugger: OnceCell::new(),
            rule_name: OnceCell::new(),
            _phantom_data: PhantomData,
        })
    }
}

impl<TN: NodeImpl, TL: TokenImpl> PunctuationsField<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> PunctuationsField<TN, TL> {
    fn semantics(&self) -> Vec<String> {
        self.values
            .iter()
            .rev()
            .map(|(v, node_value)| match node_value {
                Some(node) => format!("[{:?}; {:?}]", v, node),
                None => format!("[{:?}; ]", v),
            })
            .collect()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for PunctuationsField<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for PunctuationsField<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(s) => write!(f, "{}", s),
            None => Ok({
                write!(f, "({})", self.semantics().join("|"))?;
            }),
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for PunctuationsField<TN, TL> {
    type Node = TN;
    type Token = TL;

    fn is_nullable(&self) -> bool {
        self.values.first().map_or(false, |(v, _)| v.len() == 0)
    }

    fn impl_first_set<'prod>(&'prod self, _: &mut HashSet<Self::Token>) {
        panic!("First  set implementation is not expected to be called.");
    }

    fn advance_fltr_ptr(
        &self,
        _: &Code,
        _: FltrPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        panic!("Bug! ConstListTerminal should not used for tokenized parsing.")
    }

    fn advance_token_ptr(
        &self,
        _: &Code,
        _: StreamPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        panic!("Bug! ConstListTerminal should not used for tokenized parsing.")
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        match self.tree.find(&code.value[index..]) {
            Some((node_value, shift)) => {
                let consumed_ptr = index + shift;
                cache.update_index(consumed_ptr);

                #[cfg(debug_assertions)]
                self.log_success(code, index, consumed_ptr);

                match node_value {
                    Some(n) => {
                        let cached_tree: ASTNode<Self::Node> =
                            ASTNode::leaf(n.clone(), index, consumed_ptr, None);
                        return Ok(SuccessData::tree(consumed_ptr, cached_tree));
                    }
                    None => return Ok(SuccessData::hidden(consumed_ptr)),
                }
            }
            None => {
                #[cfg(debug_assertions)]
                self.log_error(code, index, &ProductionError::Unparsed);

                Err(ProductionError::Unparsed)
            }
        }
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.values
            .iter()
            .take_while(|(v, _)| v.len() == 0)
            .any(|(_, n)| n.is_none())
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, crate::ImplementationError> {
        Ok(self.is_nullable())
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
    ) -> Result<(), crate::ImplementationError> {
        Ok(())
    }
}
