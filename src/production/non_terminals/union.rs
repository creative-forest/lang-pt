use crate::production::NTHelper;
#[cfg(debug_assertions)]
use crate::production::ProductionLogger;
use crate::{
    production::Union, util::Code, ASTNode, Cache, FltrPtr, IProduction, ImplementationError,
    NodeImpl, ParsedResult, ProductionError, SuccessData, TokenImpl, TokenPtr, TokenStream,
};

use once_cell::unsync::OnceCell;
use std::fmt::Display;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

impl<TN: NodeImpl, TL: TokenImpl> Union<TN, TL> {
    /// Create a new [Union] utility without alternative production symbols.
    /// ## Arguments
    /// * `identifier` - An unique identifier.

    pub fn init(identifier: &'static str) -> Self {
        Union {
            symbols: OnceCell::new(),
            nt_helper: NTHelper::new(identifier),
            first_set: OnceCell::new(),
        }
    }

    /// Create a new [Union] utility with alternative production symbols.
    /// ## Arguments
    /// * `identifier` - An unique identifier.
    /// * `symbols` - Alternative production symbols.
    pub fn new(
        identifier: &'static str,
        symbols: Vec<Rc<dyn IProduction<Node = TN, Token = TL>>>,
    ) -> Self {
        let production_cell = OnceCell::new();

        if let Err(_) = production_cell.set(symbols) {
            panic!("Internal error.");
        }
        Self {
            symbols: production_cell,
            nt_helper: NTHelper::new(identifier),
            first_set: OnceCell::new(),
        }
    }

    /// Set a log label to debug the production based on the level of [Log](crate::util::Log).
    pub fn set_log(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.nt_helper.assign_debugger(debugger)
    }

    /// Set alternative symbols for the production.
    /// ### Arguments
    /// * `symbols` - A [Vec] of production symbols.
    pub fn set_symbols(
        &self,
        symbols: Vec<Rc<dyn IProduction<Node = TN, Token = TL>>>,
    ) -> Result<(), String> {
        self.symbols.set(symbols).map_err(|err| {
            format!(
                "Symbols {:?} is already set for {}.",
                err.iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<String>>(),
                self.nt_helper.identifier
            )
        })
    }
    fn get_productions(&self) -> &Vec<Rc<dyn IProduction<Node = TN, Token = TL>>> {
        self.symbols.get_or_init(|| {
            if cfg!(debug_assertions) {
                panic!("Productions is not set")
            }
            Vec::new()
        })
    }

    fn obtain_first_set(&self) -> &(bool, Vec<(TL, Vec<usize>)>) {
        self.first_set.get_or_init(|| {
            let mut children_set: HashMap<TL, Vec<usize>> = HashMap::new();
            for (index, prod) in self.get_productions().iter().enumerate() {
                let mut child_set = HashSet::new();
                prod.impl_first_set(&mut child_set);

                for t in child_set {
                    let indexes = children_set.entry(t).or_insert_with(|| Vec::new());
                    indexes.push(index);
                }
                if prod.is_nullable() {
                    for (_, v) in &mut children_set {
                        if v.last().unwrap() != &index {
                            v.push(index);
                        }
                    }
                    break; // We do not want to visit more production if we find nullable production.
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

            let mut v: Vec<(TL, Vec<usize>)> = children_set.into_iter().collect();
            v.sort_by_key(|(t, _)| *t);
            (v.iter().all(|(t, _)| t.is_structural()), v)
        })
    }
}

impl<TN: NodeImpl, TL: TokenImpl> Display for Union<TN, TL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.nt_helper.identifier)
    }
}
impl<TN: NodeImpl, TL: TokenImpl> IProduction for Union<TN, TL> {
    type Node = TN;

    type Token = TL;

    fn is_nullable(&self) -> bool {
        match self.nt_helper.nullability.get() {
            Some(s) => *s,
            None => self
                .obtain_nullability(HashMap::new())
                .expect("Nullability error should have been caught in validation"),
        }
    }

    fn is_nullable_n_hidden(&self) -> bool {
        *self.nt_helper.null_hidden.get_or_init(|| {
            self.get_productions()
                .iter()
                .find(|prod| prod.is_nullable())
                .map_or(false, |p| p.is_nullable_n_hidden())
        })
    }

    fn obtain_nullability<'id>(
        &'id self,
        mut visited: HashMap<&'id str, usize>,
    ) -> Result<bool, crate::ImplementationError> {
        self.nt_helper.validate_circular_dependency(&mut visited)?;

        match self.nt_helper.nullability.get() {
            Some(s) => Ok(*s),
            None => {
                let mut is_nullable = false;
                for prod in self.get_productions() {
                    if prod.obtain_nullability(visited.clone())? {
                        is_nullable = true;
                        break;
                    }
                }
                self.nt_helper.nullability.set(is_nullable).unwrap();
                Ok(is_nullable)
            }
        }
    }

    fn impl_first_set(&self, first_set: &mut HashSet<Self::Token>) {
        for prod in self.get_productions() {
            prod.impl_first_set(first_set)
        }
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        if visited.insert(self.nt_helper.identifier) {
            writeln!(writer, "{}", self.nt_helper.identifier)?;
            for (index, prod) in self.get_productions().iter().enumerate() {
                if index == 0 {
                    writeln!(writer, "{:>6} {}", ":", prod)?;
                } else {
                    writeln!(writer, "{:>6} {}", "|", prod)?;
                }
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
            if self.symbols.get().is_none() {
                return Err(ImplementationError::new(
                    "InitializationError".into(),
                    format!(
                        "Symbols are not assigned for {:?}.",
                        self.nt_helper.identifier
                    ),
                ));
            }
            for prod in self.get_productions() {
                prod.validate(connected_set.clone(), visited_prod)?;
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

        let immediate_lex = &token_stream[fltr_ptr];

        let (is_structural, first_sets) = self.obtain_first_set();

        let productions = self.get_productions();

        let mut production_set_index: Option<usize> = None;

        match first_sets.binary_search_by_key(&immediate_lex.token, |(t, _)| *t) {
            Ok(p_index) => {
                production_set_index = Some(p_index);
            }
            Err(_) => {
                if !is_structural {
                    let last_token_ptr = if fltr_ptr > FltrPtr::default() {
                        token_stream.get_stream_ptr(fltr_ptr - 1)
                    } else {
                        TokenPtr::default()
                    };

                    let current_token_ptr = last_token_ptr + 1;

                    let current_token_lex = &token_stream[current_token_ptr];

                    if !current_token_lex.token.is_structural() {
                        if let Ok(p_i) =
                            first_sets.binary_search_by_key(&current_token_lex.token, |(t, _)| *t)
                        {
                            production_set_index = Some(p_i);
                        }
                    }
                }
            }
        }
        match production_set_index {
            Some(p_index) => {
                for prod in first_sets[p_index].1.iter().map(|j| &productions[*j]) {
                    match prod.advance_fltr_ptr(code, fltr_ptr, token_stream, cache) {
                        Ok(s) => {
                            #[cfg(debug_assertions)]
                            self.nt_helper.log_success(
                                code,
                                token_stream[fltr_ptr].start,
                                token_stream[s.consumed_index].start,
                            );

                            return Ok(s);
                        }
                        Err(err) => {
                            if err.is_invalid() {
                                #[cfg(debug_assertions)]
                                self.nt_helper
                                    .log_error(code, token_stream[fltr_ptr].start, &err);
                                // println!("Returning validation Err:{:?}", err);
                                return Err(err);
                            }
                        }
                    }
                }
            }
            None => {
                if self.is_nullable_n_hidden() {
                    return Ok(SuccessData::hidden(fltr_ptr));
                } else if self.is_nullable() {
                    let tree = ASTNode::null(
                        token_stream[fltr_ptr].start,
                        Some(token_stream.get_stream_ptr(fltr_ptr)),
                    );
                    return Ok(SuccessData::tree(fltr_ptr, tree));
                }
            }
        }

        #[cfg(debug_assertions)]
        self.nt_helper.log_error(
            code,
            token_stream[fltr_ptr].start,
            &ProductionError::Unparsed,
        );

        Err(ProductionError::Unparsed)
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let immediate_lex = &token_stream[index];

        let (_, first_sets) = self.obtain_first_set();

        if let Ok(p_index) = first_sets.binary_search_by_key(&immediate_lex.token, |(t, _)| *t) {
            let productions = self.get_productions();
            for prod in first_sets[p_index].1.iter().map(|j| &productions[*j]) {
                match prod.advance_token_ptr(code, index, token_stream, cache) {
                    Ok(s) => {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_success(
                            code,
                            immediate_lex.start,
                            token_stream[s.consumed_index].start,
                        );

                        return Ok(s);
                    }
                    Err(err) => {
                        if err.is_invalid() {
                            #[cfg(debug_assertions)]
                            self.nt_helper.log_error(code, immediate_lex.start, &err);
                            // println!("Returning validation Err:{:?}", err);
                            return Err(err);
                        }
                    }
                }
            }
        }

        if self.is_nullable_n_hidden() {
            Ok(SuccessData::hidden(index))
        } else if self.is_nullable() {
            let tree = ASTNode::null(token_stream[index].start, Some(index));
            Ok(SuccessData::tree(index, tree))
        } else {
            #[cfg(debug_assertions)]
            self.nt_helper
                .log_error(code, token_stream[index].start, &ProductionError::Unparsed);

            Err(ProductionError::Unparsed)
        }
    }

    fn advance_ptr(
        &self,
        code: &crate::util::Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        for prod in self.get_productions() {
            match prod.advance_ptr(code, index, cache) {
                Ok(s) => return Ok(s),
                Err(err) => {
                    if err.is_invalid() {
                        #[cfg(debug_assertions)]
                        self.nt_helper.log_error(code, index, &err);

                        return Err(err);
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        self.nt_helper
            .log_error(code, index, &ProductionError::Unparsed);

        Err(ProductionError::Unparsed)
    }
}
