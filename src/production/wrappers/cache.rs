use crate::{
    production::{Cacheable, ProductionLogger},
    util::Code,
    Cache, CacheKey, FltrPtr, IProduction, ImplementationError, ParsedResult, TokenPtr,
    TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Cacheable<TProd> {
    pub fn new(cache_key: CacheKey, production: &Rc<TProd>) -> Self {
        Self {
            cache_key,
            production: production.clone(),
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    pub fn get_production(&self) -> &Rc<TProd> {
        &self.production
    }
}

impl<TP: IProduction> Cacheable<TP> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Cacheable<TProd> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for Cacheable<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.get_production())
    }
}
impl<TProd: IProduction> IProduction for Cacheable<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        self.get_production().is_nullable()
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
        true
    }

    #[inline]
    fn validate<'id>(
        &'id self,
        first_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        self.get_production().validate(first_sets, visited_prod)
    }

    fn advance_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        memory_cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let lex_data = &token_stream[index];
        let result = match memory_cache.find(self.cache_key, lex_data.start) {
            Some(result) => result.clone(),
            None => {
                let advance_result = self.get_production().advance_fltr_ptr(
                    code,
                    index,
                    token_stream,
                    memory_cache,
                );
                memory_cache.insert(self.cache_key, lex_data.start, advance_result.clone());
                advance_result
            }
        };

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);

        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        lexical_index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        memory_cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        if cfg!(debug_assertions) {
            panic!("Cacheability is not implemented for Non-structural production. Remove Cacheable wrapper for {} production",self.get_production());
        } else {
            self.get_production()
                .advance_token_ptr(code, lexical_index, token_stream, memory_cache)
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

        let result = match cache.find(self.cache_key, index) {
            Some(result) => result.clone(),
            None => {
                let advance_result = self.get_production().advance_ptr(code, index, cache);
                cache.insert(self.cache_key, index, advance_result.clone());
                advance_result
            }
        };

        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);

        result
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        self.production.impl_grammar(writer, visited)
    }
}
