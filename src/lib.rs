//! Language parsing tool (lang_pt) is a library to generate a recursive descent top-down parser to parse languages or text into Abstract Syntax Tree ([AST](ASTNode)).
//!
//! # Overview
//! Parsers written for the languages like Javascript are often custom handwritten due to the complexity of the languages.
//! However, writing custom parser code often increases development and maintenance costs for the parser.
//! With an intention to reduce development efforts, the library has been created for building a parser for a high-level language (HLL).
//! The goal for this library is to develop a flexible library to support a wide range of grammar keeping a fair performance in comparison to a custom-written parser.
//!
//!
//! # Design
//!
//! A language parser is usually developed either by writing custom code by hand or using a parser generator tool.
//! While building a parser using a parser generator, grammar for the language is implemented in a Domain Specific Language (DSL) specified by the generator tool.
//! The generator will then compile the grammar and generate a parser code in the target runtime language.
//! However, this parser library uses a set of production utilities to implement grammar in the rust programming language.
//! Therefore, instead of writing grammar in the generator-specified language, one can make use of utilities
//! like [Concat](crate::production::Concat), [Union](crate::production::Union), etc.
//! to implement concatenation and alternative production of symbols.
//!
//! This parsing tool is also equipped with utilities like [Lookahead](crate::production::Lookahead), [Validator](crate::production::Validator),
//! and [NonStructural](crate::production::NonStructural) to support custom validation, precedence-based parsing, etc.
//! This parsing library can be used to parse a wide range of languages which often require custom functionality to be injected into the grammar.
//! Moreover, the library also includes production utilities like [SeparatedList](crate::production::SeparatedList), and [Suffixes](crate::production::Suffixes),
//! to ease writing grammar for a language.
//!
//! # Example
//!
//! Following is the JSON program implementation using lang_pt.
//! ```
//! // # Tokenization
//!
//! use lang_pt::production::ProductionBuilder;
//! use lang_pt::{
//!     lexeme::{Pattern, Punctuations},
//!     production::{Concat, EOFProd, Node, SeparatedList, TokenField, TokenFieldSet, Union},
//!     DefaultParser, NodeImpl, TokenImpl, Tokenizer,
//! };
//! use std::rc::Rc;
//!
//! #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
//! // JSON token
//! pub enum JSONToken {
//!     EOF,
//!     String,
//!     Space,
//!     Colon,
//!     Comma,
//!     Number,
//!     Constant,
//!     OpenBrace,
//!     CloseBrace,
//!     OpenBracket,
//!     CloseBracket,
//! }
//!
//! #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
//! // Node value for AST
//! pub enum JSONNode {
//!     Key,
//!     String,
//!     Number,
//!     Constant,
//!     Array,
//!     Object,
//!     Item,
//!     Main,
//!     NULL,
//! }
//!
//! impl TokenImpl for JSONToken {
//!     fn eof() -> Self { JSONToken::EOF }
//!     fn is_structural(&self) -> bool {
//!         match self {
//!             JSONToken::Space => false,
//!             _ => true,
//!         }
//!     }
//! }
//! impl NodeImpl for JSONNode {
//!     fn null() -> Self { JSONNode::NULL }
//! }
//!
//! let punctuations = Rc::new(
//!     Punctuations::new(vec![
//!         ("{", JSONToken::OpenBrace),
//!         ("}", JSONToken::CloseBrace),
//!         ("[", JSONToken::OpenBracket),
//!         ("]", JSONToken::CloseBracket),
//!         (",", JSONToken::Comma),
//!         (":", JSONToken::Colon),
//!     ])
//!     .unwrap(),
//! );
//!
//! let dq_string = Rc::new(
//!     Pattern::new(
//!         JSONToken::String,
//!         r#"^"([^"\\\r\n]|(\\[^\S\r\n]*[\r\n][^\S\r\n]*)|\\.)*""#, //["\\bfnrtv]
//!     )
//!     .unwrap(),
//! );
//!
//! let lex_space = Rc::new(Pattern::new(JSONToken::Space, r"^\s+").unwrap());
//! let number_literal = Rc::new(
//!     Pattern::new(JSONToken::Number, r"^([0-9]+)(\.[0-9]+)?([eE][+-]?[0-9]+)?").unwrap(),
//! );
//! let const_literal = Rc::new(Pattern::new(JSONToken::Constant, r"^(true|false|null)").unwrap());
//!
//! let tokenizer = Tokenizer::new(vec![
//!     lex_space,
//!     punctuations,
//!     dq_string,
//!     number_literal,
//!     const_literal,
//! ]);
//!
//! // # Parser
//!
//! let eof = Rc::new(EOFProd::new(None));
//!
//! let json_key = Rc::new(TokenField::new(JSONToken::String, Some(JSONNode::Key)));
//!
//! let json_primitive_values = Rc::new(TokenFieldSet::new(vec![
//!     (JSONToken::String, Some(JSONNode::String)),
//!     (JSONToken::Constant, Some(JSONNode::Constant)),
//!     (JSONToken::Number, Some(JSONNode::Number)),
//! ]));
//!
//!
//! let hidden_open_brace = Rc::new(TokenField::new(JSONToken::OpenBrace, None));
//! let hidden_close_brace = Rc::new(TokenField::new(JSONToken::CloseBrace, None));
//! let hidden_open_bracket = Rc::new(TokenField::new(JSONToken::OpenBracket, None));
//! let hidden_close_bracket = Rc::new(TokenField::new(JSONToken::CloseBracket, None));
//! let hidden_comma = Rc::new(TokenField::new(JSONToken::Comma, None));
//! let hidden_colon = Rc::new(TokenField::new(JSONToken::Colon, None));
//! let json_object = Rc::new(Concat::init("json_object"));
//! let json_value_union = Rc::new(Union::init("json_value_union"));
//! let json_object_item = Rc::new(Concat::new(
//!     "json_object_item",
//!     vec![
//!         json_key.clone(),
//!         hidden_colon.clone(),
//!         json_value_union.clone(),
//!     ],
//! ));
//!
//! let json_object_item_node = Rc::new(Node::new(&json_object_item, Some(JSONNode::Item)));
//! let json_object_item_list =
//!     Rc::new(SeparatedList::new(&json_object_item_node, &hidden_comma, true).into_nullable());
//! let json_array_item_list =
//!     Rc::new(SeparatedList::new(&json_value_union, &hidden_comma, true).into_nullable());
//! let json_array_node = Rc::new(
//!     Concat::new(
//!         "json_array",
//!         vec![
//!             hidden_open_bracket.clone(),
//!             json_array_item_list.clone(),
//!             hidden_close_bracket.clone(),
//!         ],
//!     )
//!     .into_node(Some(JSONNode::Array)),
//! );
//!
//! let json_object_node = Rc::new(Node::new(&json_object, Some(JSONNode::Object)));
//!
//! json_value_union
//!     .set_symbols(vec![
//!         json_primitive_values.clone(),
//!         json_object_node.clone(),
//!         json_array_node.clone(),
//!     ])
//!     .unwrap();
//!
//! json_object
//!     .set_symbols(vec![
//!         hidden_open_brace.clone(),
//!         json_object_item_list,
//!         hidden_close_brace.clone(),
//!     ])
//!     .unwrap();
//!
//! let main = Rc::new(Concat::new("root", vec![json_value_union, eof]));
//! let main_node = Rc::new(Node::new(&main, Some(JSONNode::Main)));
//! let parser = DefaultParser::new(Rc::new(tokenizer), main_node).unwrap();
//!
//! ```

//! # License
//! [lang_pt](crate) is provided under the MIT license. See [LICENSE](https://github.com/creative-forest/lang-pt/blob/main/LICENSE).
mod ast_node;
mod cache;
mod code;
mod doc;
mod error;
pub mod examples;
mod field_tree;
mod filtered_stream;
mod impl_default;
mod lex;
pub mod lexeme;
mod logger;
mod parsing;
mod position;
pub mod production;
mod success_data;
mod tokenization;
mod wrapper_index;

use once_cell::unsync::OnceCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Write};
use std::hash::Hash;
use std::rc::Rc;

/// A trait implementation to generate default tokens to assign token values to the associated [ASTNode].
///
/// When the tokenizer or the parser encounter null production or end of file production during lexical analysis and parsing phase,
/// self implementation will create and assign corresponding token in the token stream and [ASTNode].  
/// A trait implementation to filter the tokens after lexical analysis.
///
/// The non structural tokens like whitespace, line break, in javascript language do not provide any grammatical meaning.
/// Therefore these tokens can be omitted from the tokes stream to simplify the grammar and optimize the parser performance.
pub trait TokenImpl: Copy + Debug + Eq + Hash + Ord {
    fn eof() -> Self;
    fn is_structural(&self) -> bool;
}

/// A trait implementation to generate default tokens to assign token values to the associated [ASTNode].
///
/// When the tokenizer or the parser encounter null production or end of file production during lexical analysis and parsing phase,
/// self implementation will create and assign corresponding token in the token stream and [ASTNode].  
pub trait NodeImpl: Debug + Clone {
    /// Default token placeholder for null production.
    fn null() -> Self;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// A wrapper to indicate the index of the tokenized data in the [TokenStream].
pub struct TokenPtr(usize);

#[derive(Clone)]
/// Abstract Syntax tree (AST) of the parsed input.
pub struct ASTNode<TNode> {
    pub node: TNode,
    pub bound: Option<(TokenPtr, TokenPtr)>, // Start and end position information of the lexical stream generated from the tokenizer.
    pub start: usize, // Actual starting position of the parsed utf-8 slice. This is different from the starting position of the parsed string.
    pub end: usize, // Actual end point of the parsed utf-8 slice. This is different from the end of the parsed string.
    pub children: Vec<ASTNode<TNode>>, // Children of the abstract syntax tree
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
/// Element of the tokenized data.
pub struct Lex<TToken> {
    pub token: TToken,
    pub start: usize,
    pub end: usize,
}

/// An interface implemented by all lexeme utilities which are primary element of a tokenizer.   
pub trait ILexeme {
    type Token: Copy + Debug + Eq + Ord;
    type State: Copy + Debug + Eq + Ord;

    /// Primary tokenization method implemented by each lexeme utility.
    /// The analyzer will call this method for all the lexeme at the incremental locations of the input to create tokens.
    fn consume(
        &self,
        code: &Code,
        pointer: usize,
        tokenized_stream: &Vec<Lex<Self::Token>>,
        state_stack: &mut Vec<Self::State>,
    ) -> Option<Lex<Self::Token>>;

    fn get_grammar_field(&self) -> Vec<(Self::Token, String)>;
}

/// A trait consists of [tokenize](ITokenization::tokenize) method which takes input utf-8 string bytes and produces a tokens stream.
///
/// This interface implemented by [Tokenizer] and [CombinedTokenizer].
pub trait ITokenization {
    type Token;
    fn tokenize(&self, code: &Code) -> Result<Vec<Lex<Self::Token>>, ParseError>;
    fn build_grammar(&self) -> Result<String, std::fmt::Error>;
}

/// Base tokenization structure for lexical analysis.
///
/// The [Tokenizer] implements [ITokenization] where the [tokenize](ITokenization::tokenize) method
/// from this trait will split the input string into a token stream and return the result.
/// The [Tokenizer] object consists of lexeme utilities.
/// During tokenization, each lexeme utility will be called sequentially to get split tokens input.
///
pub struct Tokenizer<TToken = i8, TState = u8> {
    lexers: Vec<Rc<dyn ILexeme<Token = TToken, State = TState>>>,
}

/// A state-based tokenizer for lexical analysis.
///
/// A [CombinedTokenizer] consist of multiple set of lexeme utilities.
/// During tokenization lexeme utilities corresponding to the state will be called sequentially to get split tokens input.
/// A [StateMixin][crate::lexeme::StateMixin] or [ThunkStateMixin][crate::lexeme::ThunkStateMixin] can be used with to change the state stack during tokenization.
///  
/// Tokenizing a complex language syntax like template literal in javascript,
/// required implementing a separate state to tokenize template the literal part of the input.
/// Thus, a [CombinedTokenizer] allows us to define a multiple states-based lexer required to tokenize relatively complex language syntax.  
/// Similar to the [Tokenizer] a [CombinedTokenizer] also implements [ITokenization]
/// where the [tokenize](ITokenization::tokenize) method will split the input string into a stream of tokens.
///
pub struct CombinedTokenizer<TT = i8, TS = u8> {
    analyzers: Vec<(TS, Vec<Rc<dyn ILexeme<Token = TT, State = TS>>>)>,
    default_state: TS,
    debug: OnceCell<Log<&'static str>>,
}

#[derive(Debug)]
/// An error returned due to failed validation of production utilities and grammar.
pub struct ImplementationError {
    message: String,
    what: String,
}

#[derive(Debug, Clone)]
/// An error to indicate failure while consuming input into [AST](crate::ASTNode).
///
/// When production failed to parse inputs, the parser will try to implement alternative production or backtracking.
/// However, [Validation](crate::ProductionError::Validation) error will simple terminate the parsing and return [Err] result.   
pub enum ProductionError {
    Unparsed,
    Validation(usize, String),
}

#[derive(Debug)]
/// An error returned when the parser failed to parse the input because of the language syntax error.
pub struct ParseError {
    pub pointer: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
/// A wrapper implementation of the tokenized data.
pub struct TokenStream<'lex, TNode> {
    filtered_stream: Vec<TokenPtr>,
    original_stream: &'lex Vec<Lex<TNode>>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// A wrapper implementation to indicate the indices of structural tokens of the [TokenStream].
pub struct FltrPtr(usize);

#[derive(Debug, Clone)]
/// A [Ok] result value returned from the [production](IProduction) utility
/// when it successfully consume production [derivation](IProduction::advance_token_ptr()).
pub struct SuccessData<I, TNode> {
    pub consumed_index: I,
    pub children: Vec<ASTNode<TNode>>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
///  A unique key to save and retrieve parsed results for the Packrat parsing technique.
pub struct CacheKey(usize);

/// A result returned from [Production](IProduction) when it try to [consume][IProduction::advance_token_ptr] inputs.
pub type ParsedResult<I, TToken> = Result<SuccessData<I, TToken>, ProductionError>;

/// An object structure to store maximum successful parse position and parsed result for Packrat parsing technique.   
pub struct Cache<TP, TToken> {
    parsed_result_cache: HashMap<(CacheKey, usize), ParsedResult<TP, TToken>>,
    max_parsed_point: usize,
}

/// A trait implemented by production utilities which are used to write the various production rule for writing the grammar.
pub trait IProduction: Display {
    type Node: NodeImpl;
    type Token: TokenImpl;
    /// Whether the production is nullable.
    fn is_nullable(&self) -> bool;

    /// Whether the production is nullable and the parsed tree should be hidden from the [ASTNode].
    fn is_nullable_n_hidden(&self) -> bool;

    /// Validate if any first set child production is left recursive and return id production is nullable.
    fn obtain_nullability<'id>(
        &'id self,
        visited: HashMap<&'id str, usize>,
    ) -> Result<bool, ImplementationError>;
    // fn obtain_hidden_stat<'id>(&'id self, visited: &mut HashSet<&'id str>) -> bool;
    fn impl_first_set(&self, first_set: &mut HashSet<Self::Token>);
    // fn has_first_set(&self, lex_index: LexIndex, stream: &LexStream<Self::Token>) -> bool;

    /// Write grammar for the production.
    fn impl_grammar(
        &self,
        writer: &mut dyn Write,
        added_rules: &mut HashSet<&'static str>,
    ) -> Result<(), std::fmt::Error>;

    /// Validate this and all children production for left recursion.
    fn validate<'id>(
        &'id self,
        connected_sets: HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<(), ImplementationError>;

    /// Consume input in filtered token stream.
    fn advance_fltr_ptr(
        &self,
        code: &Code,
        index: FltrPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<FltrPtr, Self::Node>;

    /// Consume tokenized input data.
    fn advance_token_ptr(
        &self,
        code: &Code,
        index: TokenPtr,
        token_stream: &TokenStream<Self::Token>,
        cache: &mut Cache<FltrPtr, Self::Node>,
    ) -> ParsedResult<TokenPtr, Self::Node>;

    /// Consume a utf-8 byte string input.
    fn advance_ptr(
        &self,
        code: &Code,
        index: usize,
        cache: &mut Cache<usize, Self::Node>,
    ) -> ParsedResult<usize, Self::Node>;

    fn build_grammar(&self) -> Result<String, std::fmt::Error> {
        let mut writer = String::new();
        writeln!(writer, "{}", self)?;
        self.impl_grammar(&mut writer, &mut HashSet::new())?;
        Ok(writer)
    }
}

/// A parser structure to construct a tokenized based parsing program.
pub struct DefaultParser<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    tokenizer: Rc<dyn ITokenization<Token = TL>>,
    root: Rc<dyn IProduction<Node = TN, Token = TL>>,
    #[cfg(debug_assertions)]
    debug_production_map: HashMap<&'static str, Rc<dyn IProduction<Node = TN, Token = TL>>>,
}

/// A parser structure for parsing input without a tokenizer.
pub struct LexerlessParser<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    root: Rc<dyn IProduction<Node = TN, Token = TL>>,
    #[cfg(debug_assertions)]
    debug_production_map: HashMap<&'static str, Rc<dyn IProduction<Node = TN, Token = TL>>>,
}

#[derive(Clone, Debug)]
struct FieldTree<T> {
    token: Option<T>,
    children: Vec<(u8, FieldTree<T>)>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
/// The line and column information at code point.
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// A wrapper for the input language to be parsed with lines information.
pub struct Code<'c> {
    pub value: &'c [u8],
    line_breaks: OnceCell<Vec<usize>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A enum structure to assign multiple level debugging to lexeme and production utilities.
pub enum Log<T> {
    None,
    Default(T),
    Success(T),
    Result(T),
    Verbose(T),
}
