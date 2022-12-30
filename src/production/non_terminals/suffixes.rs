use crate::{
    production::{NTHelper, ProductionLogger, Suffixes, TSuffixMap},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, ProductionError,
    StreamPtr, SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TP: IProduction> Suffixes<TP> {
    /// Create a new [Suffixes] utility without suffixes production symbols.
    /// ## Arguments
    /// * `identifier` - An unique identifier.
    /// * `left` - A production utility to be parsed before suffixes.

    pub fn init(identifier: &'static str, left: &Rc<TP>, standalone: bool) -> Self {
        Suffixes {
            left: left.clone(),
            suffixes: OnceCell::new(),
            standalone,
            nt_helper: NTHelper::new(identifier),
        }
    }

    /// Create a new [Suffixes] utility with suffixes production symbols.
    /// ## Arguments
    /// * `identifier` - An unique identifier.
    /// * `left` - A production utility to be parsed before suffixes.
    /// * `standalone` - A [bool] value to indicate if null production suffix should be a valid production.
    /// * `suffixes` - A [Vec] of tuples of suffix production utility and optional node value.
    pub fn new(
        identifier: &'static str,
        left: &Rc<TP>,
        standalone: bool,
        suffixes: Vec<TSuffixMap<TP::Node, TP::Token>>,
    ) -> Self {
        let production_cell = OnceCell::new();
        if let Err(_) = production_cell.set(suffixes) {
            panic!("Report bug. Production should not be set.");
        }
        Self {
            left: left.clone(),
            suffixes: production_cell,
            standalone,
            nt_helper: NTHelper::new(identifier),
        }
    }

    /// Set a log label to debug the production based on the level of [Log](crate::util::Log).
    pub fn set_log(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.nt_helper.assign_debugger(debugger)
    }

    /// Set production symbols for concatenation operation.
    /// ### Arguments
    /// * `symbols` - A [Vec] of production symbol for suffix operation.
    pub fn set_suffixes(
        &self,
        suffixes: Vec<TSuffixMap<TP::Node, TP::Token>>,
    ) -> Result<(), String> {
        self.suffixes.set(suffixes).map_err(|err| {
            format!(
                "Symbols {:?} is already set for {}.",
                err.iter()
                    .map(|c| format!("{}", c.0))
                    .collect::<Vec<String>>(),
                self.nt_helper.identifier
            )
        })
    }
    fn get_suffixes(
        &self,
    ) -> &Vec<(
        Rc<dyn IProduction<Node = TP::Node, Token = TP::Token>>,
        Option<TP::Node>,
    )> {
        self.suffixes.get_or_init(|| {
            if cfg!(debug_assertions) {
                panic!("Productions are not set. Validate grammar before parsing.")
            }
            Vec::new()
        })
    }
}

impl<TP: IProduction> Display for Suffixes<TP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.nt_helper.identifier)
    }
}

impl<TP: IProduction> IProduction for Suffixes<TP> {
    type Node = TP::Node;

    type Token = TP::Token;

    fn is_nullable(&self) -> bool {
        match self.nt_helper.nullability.get() {
            Some(t) => *t,
            None => self
                .obtain_nullability(HashMap::new())
                .expect("LeftRecursion: Validate grammar before parsing."),
        }
    }

    fn is_nullable_n_hidden(&self) -> bool {
        *self.nt_helper.null_hidden.get_or_init(|| {
            self.left.is_nullable_n_hidden()
                && self
                    .get_suffixes()
                    .iter()
                    .find(|(p, _)| p.is_nullable())
                    .map_or(false, |(p, n)| n.is_none() && p.is_nullable_n_hidden())
        })
    }

    fn obtain_nullability<'id>(
        &'id self,
        mut visited: HashMap<&'id str, usize>,
    ) -> Result<bool, crate::ImplementationError> {
        self.nt_helper.validate_circular_dependency(&mut visited)?;
        match self.nt_helper.nullability.get() {
            Some(t) => Ok(*t),
            None => {
                let is_nullable = self.left.is_nullable() && {
                    let standalone_or_nullable = self.standalone || {
                        let mut is_nullable = true;
                        for (prod, _) in self.get_suffixes() {
                            if !prod.obtain_nullability(visited.clone())? {
                                is_nullable = false;
                                break;
                            }
                        }
                        is_nullable
                    };
                    standalone_or_nullable
                };

                self.nt_helper.nullability.set(is_nullable).unwrap();
                Ok(is_nullable)
            }
        }
    }

    fn impl_first_set(&self, first_set: &mut HashSet<Self::Token>) {
        let children_set = self.nt_helper.init_first_set(|| {
            let mut children_set = HashSet::new();
            self.left.impl_first_set(&mut children_set);
            if self.left.is_nullable() {
                for (prod, _) in self.get_suffixes() {
                    prod.impl_first_set(&mut children_set);
                    if !prod.is_nullable() {
                        break;
                    }
                }
            }
            children_set
        });
        first_set.extend(children_set);
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        if visited.insert(self.nt_helper.identifier) {
            writeln!(writer, "{}", self.nt_helper.identifier)?;
            write!(writer, "{:>6}", ":")?;
            for (index, prod) in self.get_suffixes().iter().enumerate() {
                match &prod.1 {
                    Some(node_value) => {
                        if index == 0 {
                            writeln!(writer, "[{} {}; @{:?}]", self.left, prod.0, node_value)?;
                        } else {
                            writeln!(
                                writer,
                                "{:>6} [{} {}; @{:?}]",
                                "|", self.left, prod.0, node_value
                            )?;
                        }
                    }
                    None => {
                        if index == 0 {
                            writeln!(writer, "{} {}", self.left, prod.0,)?;
                        } else {
                            writeln!(writer, "{:>6} {} {}", "|", self.left, prod.0,)?;
                        }
                    }
                }
            }
            if self.standalone {
                writeln!(writer, "{:>6} {}", "|", self.left)?;
            }
        }
        Ok(())
    }

    fn validate<'id>(
        &'id self,
        mut connected_set: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        if !self
            .nt_helper
            .has_visited(&mut connected_set, visited_prod)?
        {
            if self.suffixes.get().is_none() {
                return Err(ImplementationError::new(
                    "InitializationError".into(),
                    format!(
                        "Suffixes symbols are not assigned for {:?}.",
                        self.nt_helper.identifier
                    ),
                ));
            }

            self.left.validate(connected_set.clone(), visited_prod)?;
            let mut is_nullable: bool = self.left.obtain_nullability(HashMap::new())?;
            for (prod, _) in self.get_suffixes() {
                if is_nullable {
                    prod.validate(connected_set.clone(), visited_prod)?;
                    is_nullable = prod.obtain_nullability(HashMap::new())?;
                } else {
                    prod.validate(HashMap::new(), visited_prod)?;
                }
            }
        }
        Ok(())
    }

    fn eat_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_success_data = self.left.eat_fltr_ptr(code, index, stream, cache)?;
        // let mut parsed_children: Vec<Rc<AST<TProd::Token>>> = Vec::new();
        let moved_ptr: FltrPtr = left_success_data.consumed_index;

        for (prod, optional_node) in self.get_suffixes() {
            match prod.eat_fltr_ptr(code, moved_ptr, stream, cache) {
                Ok(success_data) => match optional_node {
                    Some(node_value) => {
                        left_success_data.consumed_index = success_data.consumed_index;
                        left_success_data.children.extend(success_data.children);
                        let ast = ASTNode::<TP::Node>::new(
                            node_value.clone(),
                            stream.pointer(index),
                            stream.pointer(success_data.consumed_index),
                            Some((
                                stream.get_stream_ptr(index),
                                stream.get_stream_ptr(success_data.consumed_index),
                            )),
                            left_success_data.children,
                        );
                        let data = SuccessData::tree(success_data.consumed_index, ast);

                        #[cfg(debug_assertions)]
                        self.nt_helper.log_success(
                            code,
                            stream[index].start,
                            stream[data.consumed_index].end,
                        );

                        return Ok(data);
                    }
                    None => {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_success(
                            code,
                            stream[index].start,
                            stream[success_data.consumed_index].start,
                        );

                        return Ok(success_data);
                    }
                },
                Err(err) => {
                    if err.is_invalid() {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_error(code, stream[index].start, &err);

                        return Err(err);
                    }
                }
            }
        }

        if self.standalone {
            #[cfg(debug_assertions)]
            self.nt_helper.log_success(
                code,
                stream[index].start,
                stream[left_success_data.consumed_index].start,
            );

            Ok(left_success_data)
        } else {
            #[cfg(debug_assertions)]
            self.nt_helper
                .log_error(code, stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn eat_token_ptr(
        &self,
        code: &Code,
        index: StreamPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_parsed_result = self.left.eat_token_ptr(code, index, stream, cache)?;
        // let mut parsed_children: Vec<Rc<AST<TProd::Token>>> = Vec::new();
        let moved_ptr: StreamPtr = left_parsed_result.consumed_index;

        for (prod, optional_node) in self.get_suffixes() {
            match prod.eat_token_ptr(code, moved_ptr, stream, cache) {
                Ok(success_data) => match optional_node {
                    Some(node_value) => {
                        left_parsed_result.consumed_index = success_data.consumed_index;
                        left_parsed_result.children.extend(success_data.children);
                        let ast = ASTNode::<TP::Node>::new(
                            node_value.clone(),
                            stream[index].start,
                            stream[success_data.consumed_index].start,
                            Some((index, success_data.consumed_index)),
                            left_parsed_result.children,
                        );

                        let data = SuccessData::tree(success_data.consumed_index, ast);
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_success(
                            code,
                            stream[index].start,
                            stream[data.consumed_index].start,
                        );
                        return Ok(data);
                    }
                    None => {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_success(
                            code,
                            stream[index].start,
                            stream[success_data.consumed_index].start,
                        );
                        return Ok(success_data);
                    }
                },
                Err(err) => {
                    if err.is_invalid() {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_error(code, stream[index].start, &err);
                        return Err(err);
                    }
                }
            }
        }

        if self.standalone {
            #[cfg(debug_assertions)]
            self.nt_helper.log_success(
                code,
                stream[index].start,
                stream[left_parsed_result.consumed_index].start,
            );
            Ok(left_parsed_result)
        } else {
            #[cfg(debug_assertions)]
            self.nt_helper
                .log_error(code, stream[index].start, &ProductionError::Unparsed);
            Err(ProductionError::Unparsed)
        }
    }

    fn eat_ptr(
        &self,
        code: &crate::util::Code,
        index: usize,
        cache: &mut crate::Cache<usize, Self::Node>,
    ) -> crate::ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_parsed_result = self.left.eat_ptr(code, index, cache)?;
        let moved_ptr: usize = left_parsed_result.consumed_index;

        for (prod, optional_node) in self.get_suffixes() {
            match prod.eat_ptr(code, moved_ptr, cache) {
                Ok(success_data) => match optional_node {
                    Some(node_value) => {
                        left_parsed_result.consumed_index = success_data.consumed_index;
                        left_parsed_result.children.extend(success_data.children);
                        let ast = ASTNode::<TP::Node>::new(
                            node_value.clone(),
                            index,
                            success_data.consumed_index,
                            None,
                            left_parsed_result.children,
                        );
                        return Ok(SuccessData::tree(success_data.consumed_index, ast));
                    }
                    None => return Ok(success_data),
                },
                Err(err) => {
                    if err.is_invalid() {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_error(code, index, &err);

                        return Err(err);
                    }
                }
            }
        }

        if self.standalone {
            Ok(left_parsed_result)
        } else {
            #[cfg(debug_assertions)]
            self.nt_helper
                .log_error(code, index, &ProductionError::Unparsed);
            Err(ProductionError::Unparsed)
        }
    }
}
