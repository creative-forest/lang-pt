use crate::production::ProductionLogger;
use crate::{
    production::SeparatedList, util::Code, Cache, FltrPtr, IProduction, ImplementationError,
    ParsedResult, TokenPtr, SuccessData, TokenStream,
};
use once_cell::unsync::OnceCell;
use std::hash::Hash;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

impl<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> SeparatedList<TP, TS> {
    pub fn new(production: &Rc<TP>, separator: &Rc<TS>, inclusive: bool) -> Self {
        Self {
            rule_name: OnceCell::new(),
            inclusive,
            production: production.clone(),
            separator: separator.clone(),
            debugger: OnceCell::new(),
        }
    }
    #[inline]
    pub fn get_production(&self) -> &TP {
        &self.production
    }

    #[inline]
    pub fn get_separator(&self) -> &TS {
        &self.separator
    }

    pub fn set_rule_name(&self, s: &'static str) -> Result<(), String> {
        self.rule_name
            .set(s)
            .map_err(|err| format!("Rule name {} is already assigned", err))
    }

    fn consume<
        T: Copy,
        TCache: Copy + Default + Eq + Hash + Ord,
        P: Fn(T, &mut Cache<TCache, TP::Node>) -> ParsedResult<T, TP::Node>,
        S: Fn(T, &mut Cache<TCache, TP::Node>) -> ParsedResult<T, TP::Node>,
    >(
        &self,
        index: T,
        cache: &mut Cache<TCache, TP::Node>,
        parse_production: P,
        parse_separator: S,
    ) -> ParsedResult<T, TP::Node> {
        let success_data = parse_production(index, cache)?;

        let mut moved_ptr = success_data.consumed_index;
        let mut children = success_data.children;
        loop {
            match parse_separator(moved_ptr, cache) {
                Ok(separator_success_data) => {
                    match parse_production(separator_success_data.consumed_index, cache) {
                        Ok(next_success_data) => {
                            children.extend(separator_success_data.children);
                            children.extend(next_success_data.children);
                            moved_ptr = next_success_data.consumed_index;
                        }
                        Err(err) => {
                            if err.is_invalid() {
                                return Err(err);
                            } else if self.inclusive {
                                children.extend(separator_success_data.children);
                                break Ok(SuccessData::new(
                                    separator_success_data.consumed_index,
                                    children,
                                ));
                            } else {
                                break Ok(SuccessData::new(moved_ptr, children));
                            }
                        }
                    }
                }
                Err(err) => {
                    if err.is_invalid() {
                        break Err(err);
                    } else {
                        break Ok(SuccessData::new(moved_ptr, children));
                    }
                }
            }
        }
    }
}

impl<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> SeparatedList<TP, TS> {
    pub fn assign_debugger(&self, debugger: crate::util::Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}

impl<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> ProductionLogger
    for SeparatedList<TP, TS>
{
    fn get_debugger(&self) -> Option<&crate::util::Log<&'static str>> {
        self.debugger.get()
    }
}

impl<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> Display
    for SeparatedList<TP, TS>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rule_name.get() {
            Some(&rule_name) => write!(f, "{}", rule_name),
            None => {
                write!(
                    f,
                    "{p} ({s} {p})*",
                    p = self.get_production(),
                    s = self.get_separator()
                )?;
                if !self.inclusive {
                    write!(f, " ({})?", self.get_separator())?;
                }
                Ok(())
            }
        }
    }
}
impl<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> IProduction
    for SeparatedList<TP, TS>
{
    type Node = TP::Node;
    type Token = TP::Token;

    #[inline]
    fn validate<'id>(
        &'id self,
        connected_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError> {
        // vps.push(&self.separator);
        if self.get_production().obtain_nullability(HashMap::new())? {
            self.get_production()
                .validate(connected_sets.clone(), visited_prod)?;
            self.get_separator()
                .validate(connected_sets, visited_prod)?;
        } else {
            self.get_production()
                .validate(connected_sets, visited_prod)?;
            self.get_separator()
                .validate(HashMap::new(), visited_prod)?;
        }

        Ok(())
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
                    write!(
                        writer,
                        "{:>6} {p} ({s} {p})*",
                        ":",
                        p = self.get_production(),
                        s = self.get_separator()
                    )?;

                    if !self.inclusive {
                        writeln!(writer, " ({})?", self.get_separator()).unwrap();
                    } else {
                        writeln!(writer, "")?;
                    }
                    writeln!(writer, ";")?;
                }
            }
            None => {}
        }

        self.production.impl_grammar(writer, visited)?;
        self.separator.impl_grammar(writer, visited)
    }

    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, TP::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();
        let result = self.consume(
            index,
            cache,
            |moved_pointer, c| {
                self.get_production()
                    .advance_token_ptr(code, moved_pointer, token_stream, c)
            },
            |moved_pointer, c| {
                self.get_separator()
                    .advance_token_ptr(code, moved_pointer, token_stream, c)
            },
        );
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
        let result = self.consume(
            index,
            cache,
            |moved_pointer, c| {
                self.get_production()
                    .advance_ptr(code, moved_pointer, c)
            },
            |moved_pointer, c| self.get_separator().advance_ptr(code, moved_pointer, c),
        );
        #[cfg(debug_assertions)]
        self.log_result(code, index, &result);
        result
    }

    fn advance_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node> {
        #[cfg(debug_assertions)]
        self.log_entry();

        let result = self.consume(
            index,
            cache,
            |moved_pointer, c| {
                self.get_production()
                    .advance_fltr_ptr(code, moved_pointer, token_stream, c)
            },
            |moved_pointer, c| {
                self.get_separator()
                    .advance_fltr_ptr(code, moved_pointer, token_stream, c)
            },
        );

        #[cfg(debug_assertions)]
        self.log_filtered_result(code, index, token_stream, &result);

        result
    }

    fn is_nullable(&self) -> bool {
        self.production.is_nullable() && self.separator.is_nullable()
    }

    fn is_nullable_n_hidden(&self) -> bool {
        self.production.is_nullable_n_hidden() && self.separator.is_nullable_n_hidden()
    }

    fn obtain_nullability<'id>(
        &'id self,
        visited: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError> {
        Ok(self.production.obtain_nullability(visited.clone())?
            && self.separator.obtain_nullability(visited)?)
    }

    fn impl_first_set(&self, first_set: &mut HashSet<Self::Token>) {
        self.production.impl_first_set(first_set);
        if self.production.is_nullable() {
            self.separator.impl_first_set(first_set);
        }
    }
}
