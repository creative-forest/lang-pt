use crate::{
    production::{Nullable, ProductionLogger},
    util::Code,
    ASTNode, Cache, FltrPtr, IProduction, ImplementationError, ParsedResult, StreamPtr,
    SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TProd: IProduction> Nullable<TProd> {
    pub fn new(production: &Rc<TProd>) -> Self {
        Self {
            production: production.clone(),
            debugger: OnceCell::new(),
        }
    }

    #[inline]
    pub fn get_production(&self) -> &TProd {
        &self.production
    }
}

impl<TP: IProduction> Nullable<TP> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TProd: IProduction> ProductionLogger for Nullable<TProd> {
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TProd: IProduction> Display for Nullable<TProd> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})?", self.get_production())
    }
}
impl<TProd: IProduction> IProduction for Nullable<TProd> {
    type Node = TProd::Node;
    type Token = TProd::Token;

    #[inline]
    fn is_nullable(&self) -> bool {
        true
    }

    fn impl_grammar(
        &self,
        writer: &mut dyn std::fmt::Write,
        visited: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error> {
        self.get_production().impl_grammar(writer, visited)
    }

    fn obtain_nullability<'id>(
        &'id self,
        _: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(true)
    }
    fn impl_first_set<'prod>(&'prod self, first_set: &mut HashSet<TProd::Token>) {
        self.get_production().impl_first_set(first_set);
    }

    fn is_nullable_n_hidden(&self) -> bool {
        false
    }

    #[inline]
    fn validate<'id>(
        &'id self,
        first_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        self.get_production().validate(first_sets, visited_prod)
    }

    fn eat_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cached: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        match self
            .get_production()
            .eat_fltr_ptr(code, index, token_stream, cached)
        {
            Ok(success_data) => Ok(success_data),
            Err(err) => {
                if err.is_invalid() {
                    Err(err)
                } else {
                    let pointer_start = token_stream.pointer(index);
                    let tree =
                        ASTNode::null(pointer_start, Some(token_stream.get_stream_ptr(index)));
                    Ok(SuccessData::tree(index, tree))
                }
            }
        }
    }

    fn eat_token_ptr(
        &self,
        code: &Code,
        index: StreamPtr,
        stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<StreamPtr, Self::Node> {
        match self
            .get_production()
            .eat_token_ptr(code, index, stream, cache)
        {
            Ok(success_data) => Ok(success_data),
            Err(err) => {
                if err.is_invalid() {
                    Err(err)
                } else {
                    let pointer_start = stream[index].start;
                    let tree = ASTNode::null(pointer_start, Some(index));
                    Ok(SuccessData::tree(index, tree))
                }
            }
        }
    }

    fn eat_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node> {
        match self.get_production().eat_ptr(code, index, cache) {
            Ok(success_data) => Ok(success_data),
            Err(err) => {
                if err.is_invalid() {
                    Err(err)
                } else {
                    let tree = ASTNode::null(index, None);
                    Ok(SuccessData::tree(index, tree))
                }
            }
        }
    }
}
