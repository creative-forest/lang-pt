use crate::production::NTHelper;
#[cfg(debug_assertions)]
use crate::production::ProductionLogger;
use crate::{
    production::{Suffixes, TSuffixMap},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, ProductionError,
    SuccessData, TokenImpl, TokenPtr, TokenStream,
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
            suffix_first_set: OnceCell::new(),
            null_suffix_index: OnceCell::new(),
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
            suffix_first_set: OnceCell::new(),
            null_suffix_index: OnceCell::new(),
        }
    }

    /// Set a log label to debug the production based on the level of [Log](crate::Log).
    pub fn set_log(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
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
        TP::Node,
    )> {
        self.suffixes.get_or_init(|| {
            if cfg!(debug_assertions) {
                panic!("Productions are not set. Validate grammar before parsing.")
            }
            Vec::new()
        })
    }

    fn obtain_first_null_suffix(&self) -> &Option<usize> {
        self.null_suffix_index.get_or_init(|| {
            self.get_suffixes()
                .iter()
                .position(|(prod, _)| prod.is_nullable())
        })
    }

    fn obtain_suffixes_set(&self) -> &(bool, Vec<(TP::Token, Vec<usize>)>) {
        self.suffix_first_set.get_or_init(|| {
            let mut children_set: HashMap<TP::Token, Vec<usize>> = HashMap::new();
            for (index, (prod, _)) in self.get_suffixes().iter().enumerate() {
                let mut child_set = HashSet::new();
                prod.impl_first_set(&mut child_set);
                for t in child_set {
                    let v = children_set.entry(t).or_insert_with(|| Vec::new());
                    v.push(index);
                }
                if prod.is_nullable() {
                    for (_, v) in &mut children_set {
                        if v.last().unwrap() != &index {
                            v.push(index);
                        }
                    }
                    break;
                }
            }

            #[cfg(debug_assertions)]
            for (_, v) in &children_set {
                let mut s = HashSet::new();
                for k in v {
                    if !s.insert(k) {
                        panic!("Bug! Children set is not unique.")
                    }
                }
            }

            let mut v: Vec<(TP::Token, Vec<usize>)> = children_set.into_iter().collect();
            v.sort_by_key(|(t, _)| *t);
            (v.iter().all(|(t, _)| t.is_structural()), v)
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
        *self
            .nt_helper
            .null_hidden
            .get_or_init(|| self.standalone && self.left.is_nullable_n_hidden())
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
                        let mut is_nullable = false;
                        for (prod, _) in self.get_suffixes() {
                            if prod.obtain_nullability(visited.clone())? {
                                is_nullable = true;
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
        self.left.impl_first_set(first_set);
        if self.left.is_nullable() {
            for (prod, _) in self.get_suffixes() {
                prod.impl_first_set(first_set);
            }
        }
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
                if index == 0 {
                    writeln!(writer, "[{} {}; @{:?}]", self.left, prod.0, prod.1)?;
                } else {
                    writeln!(
                        writer,
                        "{:>6} [{} {}; @{:?}]",
                        "|", self.left, prod.0, prod.1
                    )?;
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

    fn advance_fltr_ptr(
        &self,
        code: &Code,
        fltr_ptr: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_success_data =
            self.left
                .advance_fltr_ptr(code, fltr_ptr, token_stream, cache)?;
        // let mut parsed_children: Vec<Rc<AST<TProd::Token>>> = Vec::new();

        let suffixes = self.get_suffixes();

        let (is_structural, suffix_first_set) = self.obtain_suffixes_set();

        let moved_ptr: FltrPtr = left_success_data.consumed_index;
        let immediate_lex = &token_stream[moved_ptr];

        let mut production_set_index: Option<usize> = None;
        match suffix_first_set.binary_search_by_key(&immediate_lex.token, |(t, _)| *t) {
            Ok(p_index) => {
                production_set_index = Some(p_index);
            }
            Err(_) => {
                if !is_structural {
                    let last_token_ptr = if moved_ptr > FltrPtr::default() {
                        token_stream.get_token_ptr(moved_ptr - 1)
                    } else {
                        TokenPtr::default()
                    };

                    let current_token_ptr = last_token_ptr + 1;

                    let current_token_lex = &token_stream[current_token_ptr];

                    if !current_token_lex.token.is_structural() {
                        if let Ok(p_index) = suffix_first_set
                            .binary_search_by_key(&current_token_lex.token, |(t, _)| *t)
                        {
                            production_set_index = Some(p_index);
                        }
                    }
                }
            }
        }

        match production_set_index {
            Some(p_index) => {
                for (prod, node_value) in suffix_first_set[p_index].1.iter().map(|j| &suffixes[*j])
                {
                    match prod.advance_fltr_ptr(
                        code,
                        left_success_data.consumed_index,
                        token_stream,
                        cache,
                    ) {
                        Ok(data) => {
                            let mut children = left_success_data.children;
                            children.extend(data.children);
                            let ast = ASTNode::<TP::Node>::new(
                                node_value.clone(),
                                token_stream.pointer(fltr_ptr),
                                token_stream.pointer(data.consumed_index),
                                Some((
                                    token_stream.get_token_ptr(fltr_ptr),
                                    token_stream.get_token_ptr(data.consumed_index),
                                )),
                                children,
                            );

                            #[cfg(debug_assertions)]
                            self.nt_helper.log_success(
                                code,
                                token_stream[fltr_ptr].start,
                                token_stream[data.consumed_index].end,
                            );

                            return Ok(SuccessData::tree(data.consumed_index, ast));
                        }
                        Err(err) => {
                            if err.is_invalid() {
                                #[cfg(debug_assertions)]
                                self.nt_helper
                                    .log_error(code, token_stream[moved_ptr].start, &err);

                                return Err(err);
                            }
                        }
                    }
                }
            }
            None => {
                if let Some(i) = self.obtain_first_null_suffix() {
                    let left_bound = token_stream.get_token_ptr(fltr_ptr);
                    let right_bound = token_stream.get_token_ptr(left_success_data.consumed_index);

                    if !suffixes[*i].0.is_nullable_n_hidden() {
                        left_success_data.children.push(ASTNode::null(
                            token_stream[left_success_data.consumed_index].start,
                            Some(right_bound),
                        ))
                    }
                    let tree = ASTNode::new(
                        suffixes[*i].1.clone(),
                        token_stream[fltr_ptr].start,
                        token_stream[left_success_data.consumed_index].start,
                        Some((left_bound, right_bound)),
                        left_success_data.children,
                    );
                    return Ok(SuccessData::tree(left_success_data.consumed_index, tree));
                }
            }
        }
        if self.standalone {
            #[cfg(debug_assertions)]
            self.nt_helper.log_success(
                code,
                token_stream[fltr_ptr].start,
                token_stream[left_success_data.consumed_index].start,
            );

            Ok(left_success_data)
        } else {
            #[cfg(debug_assertions)]
            self.nt_helper.log_error(
                code,
                token_stream[fltr_ptr].start,
                &ProductionError::Unparsed,
            );

            Err(ProductionError::Unparsed)
        }
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_success_data = self.left.advance_token_ptr(code, index, stream, cache)?;
        // let mut parsed_children: Vec<Rc<AST<TProd::Token>>> = Vec::new();
        let moved_ptr: TokenPtr = left_success_data.consumed_index;
        let suffixes = self.get_suffixes();

        let (_, suffix_first_set) = self.obtain_suffixes_set();

        let immediate_lex = &stream[moved_ptr];

        if let Ok(i) = suffix_first_set.binary_search_by_key(&immediate_lex.token, |(t, _)| *t) {
            for (prod, node_value) in suffix_first_set[i].1.iter().map(|j| &suffixes[*j]) {
                match prod.advance_token_ptr(code, moved_ptr, stream, cache) {
                    Ok(success_data) => {
                        left_success_data.consumed_index = success_data.consumed_index;
                        left_success_data.children.extend(success_data.children);
                        let ast = ASTNode::<TP::Node>::new(
                            node_value.clone(),
                            stream[index].start,
                            stream[success_data.consumed_index].start,
                            Some((index, success_data.consumed_index)),
                            left_success_data.children,
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
                    Err(err) => {
                        if err.is_invalid() {
                            #[cfg(debug_assertions)]
                            self.nt_helper.log_error(code, stream[index].start, &err);
                            return Err(err);
                        }
                    }
                }
            }
        } else {
            if let Some(i) = self.obtain_first_null_suffix() {
                if !suffixes[*i].0.is_nullable_n_hidden() {
                    left_success_data.children.push(ASTNode::null(
                        stream[left_success_data.consumed_index].start,
                        Some(left_success_data.consumed_index),
                    ))
                }
                let tree = ASTNode::new(
                    suffixes[*i].1.clone(),
                    stream[index].start,
                    stream[left_success_data.consumed_index].start,
                    Some((index, left_success_data.consumed_index)),
                    left_success_data.children,
                );
                return Ok(SuccessData::tree(left_success_data.consumed_index, tree));
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

    fn advance_ptr(
        &self,
        code: &crate::Code,
        index: usize,
        cache: &mut crate::Cache<usize, Self::Node>,
    ) -> crate::ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let mut left_parsed_result = self.left.advance_ptr(code, index, cache)?;
        let moved_ptr: usize = left_parsed_result.consumed_index;

        for (prod, node_value) in self.get_suffixes() {
            match prod.advance_ptr(code, moved_ptr, cache) {
                Ok(success_data) => {
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
