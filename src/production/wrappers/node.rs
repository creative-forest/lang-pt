use crate::production::{Node, ProductionLogger};
use crate::util::Code;
use crate::{
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, StreamPtr,
    SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Node<TProd> {
    pub fn new(production: &Rc<TProd>, node_value: Option<TProd::Node>) -> Self {
        Self {
            rule_name: OnceCell::new(),
            node_value,
            production: production.clone(),
            debugger: OnceCell::new(),
        }
    }
    #[inline]
    pub fn get_production(&self) -> &TProd {
        &self.production
    }
    pub fn set_rule_name(&self, s: &'static str) -> Result<(), String> {
        self.rule_name
            .set(s)
            .map_err(|err| format!("Rule name {} is already assigned", err))
    }
}

impl<TP: IProduction> Node<TP> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Node<TProd> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for Node<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(&s) => write!(f, "{}", s),
            None => {
                write!(f, "[{}; @{:?}]", self.get_production(), self.node_value)
            }
        }
    }
}
impl<TProd: IProduction> IProduction for Node<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        self.get_production().is_nullable()
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        match self.rule_name.get() {
            Some(&s) => {
                if visited.insert(s.into()) {
                    writeln!(writer, "{}", s)?;
                    writeln!(
                        writer,
                        "{:>6} [{}; @{:?}]",
                        ":",
                        self.get_production(),
                        self.node_value
                    )?;
                    writeln!(writer, "{:>6}", ";")?;
                }
            }
            None => {}
        }

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
        self.is_nullable() && self.node_value.is_none()
    }

    #[inline]
    fn validate<'id>(
        &'id self,
        first_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        // println!("Validating Node:{}", self.get_id());
        self.get_production().validate(first_sets, visited_prod)
    }

    fn eat_fltr_ptr(
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
            .eat_fltr_ptr(code, index, token_stream, cached)
            .map(|parsed_data| match &self.node_value {
                Some(node) => {
                    let tree = ASTNode::new(
                        node.clone(),
                        token_stream.pointer(index),
                        token_stream.pointer(parsed_data.consumed_index),
                        Some((
                            token_stream.get_stream_ptr(index),
                            token_stream.get_stream_ptr(parsed_data.consumed_index),
                        )),
                        parsed_data.children,
                    );

                    SuccessData::tree(parsed_data.consumed_index, tree)
                }
                None => SuccessData::hidden(parsed_data.consumed_index),
            });

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);

        result
    }

    fn eat_token_ptr(
        &self,
        code: &Code,
        index: StreamPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .eat_token_ptr(code, index, token_stream, cache)
            .map(|parsed_data| match &self.node_value {
                Some(node) => {
                    let tree = ASTNode::new(
                        node.clone(),
                        token_stream[index].start,
                        token_stream[parsed_data.consumed_index].start,
                        Some((index, parsed_data.consumed_index)),
                        parsed_data.children,
                    );

                    SuccessData::tree(parsed_data.consumed_index, tree)
                }
                None => SuccessData::hidden(parsed_data.consumed_index),
            });

        #[cfg(debug_assertions)]
        self.log_lex_result(code, index, token_stream, &result);

        result
    }

    fn eat_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .eat_ptr(code, index, cache)
            .map(|parsed_data| match &self.node_value {
                Some(node) => {
                    let tree = ASTNode::new(
                        node.clone(),
                        index,
                        parsed_data.consumed_index,
                        None,
                        parsed_data.children,
                    );

                    SuccessData::tree(parsed_data.consumed_index, tree)
                }
                None => SuccessData::hidden(parsed_data.consumed_index),
            });

        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);

        result
    }
}
