use crate::{
    production::{NTHelper, ProductionLogger, Union},
    Cache, IProduction, ImplementationError, NodeImpl, ParsedResult, ProductionError, TokenImpl,
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

    fn consume<
        T,
        TCache,
        P: Fn(
            &Rc<dyn IProduction<Node = TN, Token = TL>>,
            &mut Cache<TCache, TN>,
        ) -> ParsedResult<T, TN>,
    >(
        &self,
        cache: &mut Cache<TCache, TN>,
        parse_production: P,
    ) -> ParsedResult<T, TN> {
        for prod in self.get_productions() {
            match parse_production(prod, cache) {
                Ok(s) => return Ok(s),
                Err(err) => {
                    if err.is_invalid() {
                        // println!("Returning validation Err:{:?}", err);
                        return Err(err);
                    }
                }
            }
        }

        Err(ProductionError::Unparsed)
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
        let children_set = self.nt_helper.init_first_set(|| {
            let mut children_set = HashSet::new();
            for prod in self.get_productions() {
                prod.impl_first_set(&mut children_set)
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

    fn eat_fltr_ptr(
        &self,
        code: &crate::util::Code,
        index: crate::FltrPtr,
        stream: &crate::TokenStream<Self::Token>,
        cache: &mut Cache<crate::FltrPtr, Self::Node>,
    ) -> ParsedResult<crate::FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let result = self.consume(cache, |prod, cache| {
            prod.eat_fltr_ptr(code, index, stream, cache)
        });

        #[cfg(debug_assertions)]
        self.nt_helper
            .log_filtered_result(code, index, stream, &result);

        result
    }

    fn eat_token_ptr(
        &self,
        code: &crate::util::Code,
        index: crate::StreamPtr,
        stream: &crate::TokenStream<Self::Token>,
        cache: &mut Cache<crate::FltrPtr, Self::Node>,
    ) -> ParsedResult<crate::StreamPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let result = self.consume(cache, |prod, cache| {
            prod.eat_token_ptr(code, index, stream, cache)
        });

        #[cfg(debug_assertions)]
        self.nt_helper.log_lex_result(code, index, stream, &result);

        result
    }

    fn eat_ptr(
        &self,
        code: &crate::util::Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.nt_helper.log_entry();

        let result = self.consume(cache, |prod, cache| prod.eat_ptr(code, index, cache));

        #[cfg(debug_assertions)]
        self.nt_helper.log_result(code, index, &result);

        result
    }
}
