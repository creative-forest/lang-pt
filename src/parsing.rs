use super::{Cache, DefaultParser, IProduction, ImplementationError, LexerlessParser, ParseError};
use crate::{Code, ASTNode, FltrPtr, ITokenization, Lex, NodeImpl, TokenImpl, TokenStream};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

impl<TN: NodeImpl, TL: TokenImpl> DefaultParser<TN, TL> {
    pub fn new(
        tokenizer: Rc<dyn ITokenization<Token = TL>>,
        root: Rc<dyn IProduction<Node = TN, Token = TL>>,
    ) -> Result<Self, ImplementationError> {
        let parser = Self {
            tokenizer,
            root,
            #[cfg(debug_assertions)]
            debug_production_map: HashMap::new(),
        };
        parser.validate()?;
        Ok(parser)
    }

    pub fn grammar(&self) -> Result<String, std::fmt::Error> {
        let mut g = self.root.build_grammar()?;
        g.push_str(&self.tokenizer.build_grammar()?);
        Ok(g)
    }
}

impl<TN: NodeImpl, TL: TokenImpl> DefaultParser<TN, TL> {
    pub fn tokenize(&self, code: &Code) -> Result<Vec<Lex<TL>>, ParseError> {
        self.tokenizer.tokenize(code)
    }
    pub fn parse_stream<'lex>(
        &self,
        code: &Code,
        filtered_stream: TokenStream<'lex, TL>,
    ) -> Result<Vec<ASTNode<TN>>, ParseError> {
        let mut cached_data: Cache<FltrPtr, TN> = Cache::root();

        let index = FltrPtr::default();
        match self
            .root
            .advance_fltr_ptr(code, index, &filtered_stream, &mut cached_data)
        {
            Ok(sd) => Ok(sd.children),
            Err(err) => Err(cached_data.create_error(code, &filtered_stream, err)),
        }
    }

    pub fn validate(&self) -> Result<(), ImplementationError> {
        self.root.validate(HashMap::new(), &mut HashSet::new())
    }

    pub fn tokenize_n_parse(
        &self,
        text: &[u8],
    ) -> Result<(Vec<Lex<TL>>, Vec<ASTNode<TN>>), ParseError> {
        let code = Code::new(text);
        let lexical_stream = self.tokenize(&code)?;
        let filtered_stream = TokenStream::from(&lexical_stream);
        let tree_list = self.parse_stream(&code, filtered_stream)?;
        Ok((lexical_stream, tree_list))
    }
    pub fn parse<'lex>(&self, text: &[u8]) -> Result<Vec<ASTNode<TN>>, ParseError> {
        let code = Code::new(text);
        let lexical_stream = self.tokenize(&code)?;
        let filtered_stream = TokenStream::from(&lexical_stream);
        self.parse_stream(&code, filtered_stream)
    }

    pub fn add_debug_production<T: IProduction<Node = TN, Token = TL> + 'static>(
        &mut self,
        _id: &'static str,
        _production: &Rc<T>,
    ) {
        #[cfg(debug_assertions)]
        self.debug_production_map.insert(_id, _production.clone());
    }
}

#[cfg(debug_assertions)]
impl<TN: NodeImpl, TL: TokenImpl> DefaultParser<TN, TL> {
    pub fn get_production(&self, id: &str) -> Option<&Rc<dyn IProduction<Node = TN, Token = TL>>> {
        self.debug_production_map.get(id)
    }
}
#[cfg(debug_assertions)]
impl<TN: NodeImpl, TL: TokenImpl> DefaultParser<TN, TL> {
    pub fn debug_production_at(
        &self,
        id: &str,
        text: &[u8],
        pointer: usize,
    ) -> Result<Vec<ASTNode<TN>>, ParseError> {
        let production = match self.get_production(id) {
            Some(p) => p.clone(),
            None => {
                return Err(ParseError::new(
                    0,
                    format!("Production {} is not added for debugging.", id),
                ));
            }
        };
        let code = Code::new(text);

        let tokens = self.tokenize(&code)?;

        let stream = TokenStream::from(&tokens);

        let index = match stream.filtered_index_at(pointer) {
            Ok(index) | Err(index) => index,
        };

        let mut cached_data: Cache<FltrPtr, TN> = Cache::debug_new(pointer);

        cached_data.update_index(pointer);

        let success_data = production
            .advance_fltr_ptr(&code, index, &stream, &mut cached_data)
            .map_err(|err| cached_data.create_error(&code, &stream, err))?;
        Ok(success_data.children)
    }
}

impl<TN: NodeImpl, TL: TokenImpl> LexerlessParser<TN, TL> {
    pub fn new(
        root: Rc<dyn IProduction<Node = TN, Token = TL>>,
    ) -> Result<Self, ImplementationError> {
        let parser = Self {
            root,
            #[cfg(debug_assertions)]
            debug_production_map: HashMap::new(),
        };
        println!("Validating parser");
        parser.validate()?;
        println!("Parser validated");
        Ok(parser)
    }
    pub fn grammar(&self) -> Result<String, std::fmt::Error> {
        self.root.build_grammar()
    }
}

impl<TN: NodeImpl, TL: TokenImpl> LexerlessParser<TN, TL> {
    pub fn parse(&self, text: &[u8]) -> Result<Vec<ASTNode<TN>>, ParseError> {
        let code = Code::new(text);
        let mut cached_data: Cache<usize, TN> = Cache::root();

        let index = usize::default();
        match self.root.advance_ptr(&code, index, &mut cached_data) {
            Ok(sd) => Ok(sd.children),
            Err(err) => Err(cached_data.create_error(&code, err)),
        }
    }

    pub fn validate(&self) -> Result<(), ImplementationError> {
        self.root.validate(HashMap::new(), &mut HashSet::new())
    }

    pub fn add_debug_production<T: IProduction<Node = TN, Token = TL> + 'static>(
        &mut self,
        _id: &'static str,
        _production: &Rc<T>,
    ) {
        #[cfg(debug_assertions)]
        self.debug_production_map.insert(_id, _production.clone());
    }
}

#[cfg(debug_assertions)]
impl<TN: NodeImpl, TL: TokenImpl> LexerlessParser<TN, TL> {
    pub fn get_production(&self, id: &str) -> Option<&Rc<dyn IProduction<Node = TN, Token = TL>>> {
        self.debug_production_map.get(id)
    }

    #[cfg(debug_assertions)]
    pub fn debug_parser_at(
        &self,
        id: &str,
        text: &[u8],
        pointer: usize,
    ) -> Result<Vec<ASTNode<TN>>, ParseError> {
        let code = Code::new(text);

        let production = match self.get_production(id) {
            Some(p) => p.clone(),
            None => {
                return Err(ParseError::new(
                    0,
                    format!("Production {} is not added for debugging.", id),
                ));
            }
        };

        let mut cached_data: Cache<usize, TN> = Cache::debug_new(pointer);

        cached_data.update_index(pointer);

        let success_data = production
            .advance_ptr(&code, pointer, &mut cached_data)
            .map_err(|err| cached_data.create_error(&code, err))?;
        Ok(success_data.children)
    }
}
