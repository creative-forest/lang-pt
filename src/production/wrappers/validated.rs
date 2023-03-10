use crate::{
    production::{ProductionLogger, Validator},
    Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, ProductionError,
    TokenPtr, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<
        TProd: IProduction,
        TF: Fn(&Vec<ASTNode<TProd::Node>>, &[u8]) -> Result<(), ProductionError>,
    > Validator<TProd, TF>
{
    pub fn new(production: &Rc<TProd>, validation_fn: TF) -> Self {
        Self {
            validation_fn,
            production: production.clone(),
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    pub fn get_production(&self) -> &TProd {
        &self.production
    }
}

impl<
        TProd: IProduction,
        TF: Fn(&Vec<ASTNode<TProd::Node>>, &[u8]) -> Result<(), ProductionError>,
    > Validator<TProd, TF>
{
    pub fn assign_debugger(&self, debugger: crate::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<
        TProd: IProduction,
        TF: Fn(&Vec<ASTNode<TProd::Node>>, &[u8]) -> Result<(), ProductionError>,
    > ProductionLogger for Validator<TProd, TF>
{
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}
impl<
        TProd: IProduction,
        TF: Fn(&Vec<ASTNode<TProd::Node>>, &[u8]) -> Result<(), ProductionError>,
    > Display for Validator<TProd, TF>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", self.get_production())
    }
}
impl<
        TProd: IProduction,
        TF: Fn(&Vec<ASTNode<TProd::Node>>, &[u8]) -> Result<(), ProductionError>,
    > IProduction for Validator<TProd, TF>
{
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
        self.production.is_nullable_n_hidden()
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
        cached: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .advance_fltr_ptr(code, index, token_stream, cached)
            .and_then(|parsed_data| {
                (self.validation_fn)(&parsed_data.children, code.value)?;
                Ok(parsed_data)
            });

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);

        result
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .advance_token_ptr(code, index, token_stream, cache)
            .and_then(|parsed_data| {
                (self.validation_fn)(&parsed_data.children, code.value)?;
                Ok(parsed_data)
            });

        #[cfg(debug_assertions)]
        self.log_lex_result(code, index, token_stream, &result);

        result
    }

    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self
            .get_production()
            .advance_ptr(code, index, cache)
            .and_then(|parsed_data| {
                (self.validation_fn)(&parsed_data.children, code.value)?;

                Ok(parsed_data)
            });

        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);

        result
    }
}
