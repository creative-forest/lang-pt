use crate::{
    production::{ConstantField, ConstantFieldSet, ProductionLogger},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, NodeImpl, ParsedResult, ProductionError, StreamPtr,
    SuccessData, TokenImpl, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    marker::PhantomData,
};

impl<TN: NodeImpl> ConstantField<TN, i8> {
    pub fn new(value: &str, node_value: Option<TN>) -> Self {
        assert!(
            value.len() > 0,
            "StringField value should not be {:?}. Use 'NullProd' instead.",
            value
        );
        Self {
            value: value.bytes().collect(),
            node_value,
            _phantom_data: PhantomData,
            debugger: OnceCell::new(),
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> ConstantField<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for ConstantField<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for ConstantField<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = unsafe { std::str::from_utf8_unchecked(&self.value) };
        match &self.node_value {
            Some(n) => {
                write!(f, "[{:?}; {:?}]", value, n)
            }
            None => {
                write!(f, "[{:?}; ]", value)
            }
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for ConstantField<TN, TL> {
    type Node = TN;
    type Token = TL;

    fn is_nullable(&self) -> bool {
        self.value.len() == 0
    }

    fn impl_first_set<'prod>(&'prod self, _: &mut HashSet<Self::Token>) {
        panic!("StringField terminal is not expected with Token implementations");
    }

    fn eat_fltr_ptr(
        &self,
        _: &Code,
        _: FltrPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        panic!("Bug! ConstTerminal should not used with tokenized parsing.")
    }

    fn eat_token_ptr(
        &self,
        _: &Code,
        _: StreamPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        panic!("Bug! ConstTerminal should not used with tokenized parsing.")
    }

    fn eat_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        if code.value[index..].starts_with(&self.value) {
            // let s = &code[pointer..consumed_ptr];
            let consumed_ptr = index + self.value.len();
            cache.update_index(consumed_ptr);

            #[cfg(debug_assertions)]
            self.log_success(code, index, consumed_ptr);

            match &self.node_value {
                Some(n) => {
                    let cached_tree: ASTNode<Self::Node> =
                        ASTNode::leaf(n.clone(), index, consumed_ptr, None);
                    return Ok(SuccessData::tree(consumed_ptr, cached_tree));
                }
                None => return Ok(SuccessData::hidden(consumed_ptr)),
            }
        } else {
            #[cfg(debug_assertions)]
            self.log_error(code, index, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.value.len() == 0 && self.node_value.is_none()
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, crate::ImplementationError> {
        Ok(self.is_nullable())
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
    ) -> Result<(), crate::ImplementationError> {
        Ok(())
    }
}

impl<TN: NodeImpl> ConstantFieldSet<TN, i8> {
    pub fn new(mut values: Vec<(&str, Option<TN>)>) -> Self {
        values.sort_by_key(|b| b.0.len());

        let fields = values
            .into_iter()
            .map(|(s, t)| (s.bytes().collect(), t))
            .collect();

        Self {
            fields,
            rule_name: OnceCell::new(),
            debugger: OnceCell::new(),
            _token: PhantomData,
        }
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ConstantFieldSet<TN, TL> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ProductionLogger for ConstantFieldSet<TN, TL> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> ConstantFieldSet<TN, TL> {
    fn semantics(&self) -> Vec<String> {
        self.fields
            .iter()
            .rev()
            .map(|(v, node_value)| {
                let s = unsafe { std::str::from_utf8_unchecked(v) };
                match node_value {
                    Some(node) => format!("[{:?}; {:?}]", s, node),
                    None => format!("[{:?}; ]", s),
                }
            })
            .collect()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for ConstantFieldSet<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(s) => write!(f, "{}", s),
            None => {
                write!(f, "({})", self.semantics().join("|"))
            }
        }
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for ConstantFieldSet<TN, TL> {
    type Node = TN;
    type Token = TL;

    fn is_nullable(&self) -> bool {
        self.fields.first().map_or(false, |(v, _)| v.len() == 0)
    }

    fn impl_first_set<'prod>(&'prod self, _: &mut HashSet<Self::Token>) {
        panic!("First  set implementation is not expected to be called.");
    }

    fn eat_fltr_ptr(
        &self,
        _: &Code,
        _: FltrPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        panic!("Bug! ConstListTerminal should not used for tokenized parsing.")
    }

    fn eat_token_ptr(
        &self,
        _: &Code,
        _: StreamPtr,
        _: &TokenStream<Self::Token>,
        _: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        panic!("Bug! ConstListTerminal should not used for tokenized parsing.")
    }

    fn eat_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        for (key, node_value) in self.fields.iter().rev() {
            if code.value[index..].starts_with(key) {
                let consumed_ptr = index + key.len();
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
        }

        #[cfg(debug_assertions)]
        self.log_error(code, index, &ProductionError::Unparsed);

        Err(ProductionError::Unparsed)
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.fields
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
        added_rules: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        if let Some(rule_name) = self.rule_name.get() {
            if added_rules.insert(rule_name) {
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
        Ok(())
    }

    fn validate<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
        _: &mut HashSet<&'id str>,
    ) -> Result<(), crate::ImplementationError> {
        todo!()
    }
}
