//! A module consists of lexeme utilities
//! which analyze string slices at incremental positions of the input and create tokens.
//!
//! A tokenizer is essential for processing input string or language before feeding into a parser.
//! A parser written using a parser generator, implement a tokenizer which is usually defined based on a regular language.
//! And the parser generator will compile the grammar in the target runtime language.
//! However, this tokenizer implementation is based on lexeme utilities which are responsible to use a regular expression to process and tokenize input string.
//! Moreover, this library is equipped with advanced lexeme utilities that are customizable according to the requirement of language syntax.
//!
//! # Example
//!
//! In this section, we will be implementing a tokenizer to tokenize JSON input.
//!
//! We need to create token types which will be returned alongside the tokenized data.
//! The token type should implement [TokenImpl](crate::TokenImpl) to be used by the [Tokenizer](crate::Tokenizer).
//! Custom implementation for [TokenImpl](crate::TokenImpl) trait has been added to primitive types [i8], [i16], and [isize].
//!
//! However, we will be implementing custom types to return as a stream of tokens.
//!
//! ```
//! use lang_pt::Code;
//! use lang_pt::{
//!     lexeme::{Pattern, Punctuations},
//!     TokenImpl, Tokenizer,
//! };
//! use lang_pt::{ITokenization, Lex};
//! use std::rc::Rc;
//!
//! #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
//! enum JSONToken {
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
//! impl TokenImpl for JSONToken {
//!     fn eof() -> Self { JSONToken::EOF }
//!     fn is_structural(&self) -> bool {
//!         match self {
//!             JSONToken::Space => false,
//!             _ => true,
//!         }
//!     }
//! }
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
//! let tokens1 = tokenizer
//!     .tokenize(&Code::from(r#"{"a":34,"b":null}"#))
//!     .unwrap();
//!
//! assert_eq!(
//!     tokens1,
//!     vec![
//!         Lex { token: JSONToken::OpenBrace, start: 0, end: 1 },
//!         Lex { token: JSONToken::String, start: 1, end: 4 },
//!         Lex { token: JSONToken::Colon, start: 4, end: 5 },
//!         Lex { token: JSONToken::Number, start: 5, end: 7 },
//!         Lex { token: JSONToken::Comma, start: 7, end: 8 },
//!         Lex { token: JSONToken::String, start: 8, end: 11 },
//!         Lex { token: JSONToken::Colon, start: 11, end: 12 },
//!         Lex { token: JSONToken::Constant, start: 12, end: 16 },
//!         Lex { token: JSONToken::CloseBrace, start: 16, end: 17 },
//!         Lex { token: JSONToken::EOF, start: 17, end: 17 }
//!     ]
//! );
//!
//! ```
//!  

mod action;
mod builder;
mod constants;
mod mapper;
mod middleware;
mod mixin;
mod pattern;
mod punctuation;
use crate::{Code, FieldTree, ILexeme, Lex, Log};
use once_cell::unsync::OnceCell;
use regex::bytes::Regex;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

trait LexemeLogger {
    fn log_cell(&self) -> &OnceCell<Log<&'static str>>;
    fn log_enter(&self) {
        #[cfg(debug_assertions)]
        if let Some(l) = self.log_cell().get() {
            println!("Entering {}", l)
        }
    }

    fn log_result<T: Debug>(&self, _pointer: usize, _code: &Code, _result: &Option<Lex<T>>) {
        #[cfg(debug_assertions)]
        match _result {
            Some(lex) => self.log_success(_code, lex),
            None => self.log_failure(_pointer, _code),
        }
    }
    fn log_success<T: Debug>(&self, _code: &Code, _lex: &Lex<T>) {
        #[cfg(debug_assertions)]
        if let Some(log_label) = self.log_cell().get() {
            if log_label.order() >= Log::Success(()).order() {
                println!(
                    "Lexeme Success for {} : token: {:?} from {} to {}.",
                    log_label,
                    _lex.token,
                    _code.obtain_position(_lex.start),
                    _code.obtain_position(_lex.end)
                )
            }
        }
    }
    fn log_failure(&self, _pointer: usize, _code: &Code) {
        #[cfg(debug_assertions)]
        if let Some(log_label) = self.log_cell().get() {
            if log_label.order() >= Log::Result(()).order() {
                println!(
                    "Lexeme error for {} : at {}",
                    log_label,
                    _code.obtain_position(_pointer)
                )
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An enum variants to represent tokenization state action.
///
/// [Action] is used by lexeme utilities [StateMixin] and [ThunkStateMixin] to change the stack state
/// so that [CombinedTokenizer](super::CombinedTokenizer) switch to different set of lexeme utilities to tokenize part of the input string.
pub enum Action<T> {
    Pop { discard: bool },
    Append { state: T, discard: bool },
    Switch { state: T, discard: bool },
    None { discard: bool },
}

/// A regular expression based lexeme utility.
///
/// Provided regex expression will be matched at incremental position of the input utf-8 bytes string and return tokenized result.
/// The provided regular expression should be enforce to match string from the beginning i.e. expression should implement start of string (^) match.
///
/// # Example
/// ```
/// use lang_pt::{lexeme::Pattern, Code, ITokenization, Lex, TokenImpl, Tokenizer};
/// use std::rc::Rc;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// enum Token {
///     ID,
///     Space,
///     EOF,
/// }
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { self != &Self::Space }
/// }
/// let identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
/// let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();
///
/// let tokenizer = Tokenizer::new(vec![Rc::new(identifier), Rc::new(space)]);
/// let lex_stream = tokenizer.tokenize(&Code::from("abc xy")).unwrap();
/// assert_eq!(
///     lex_stream,
///     vec![
///         Lex { token: Token::ID, start: 0, end: 3 },
///         Lex { token: Token::Space, start: 3, end: 4 },
///         Lex { token: Token::ID, start: 4, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 },
///     ]
/// );
/// ```
pub struct Pattern<TToken, TState = u8> {
    token: TToken,
    regexp: Regex,
    log: OnceCell<Log<&'static str>>,
    _state: PhantomData<TState>,
}

/// A lexer utility to match a set of constant values like punctuations, operators etc.  
///
/// Match punctuation values at the incremental position of the input and return tokenized result.
/// This lexeme utility will create a tree structure from utf-8 values of the provided punctuation.
/// The input utf-8 values will be match with each node of the tree return associated token value if complete match is found.  
///
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{Pattern, Punctuations},
///     Code,
///     ITokenization, Lex, TokenImpl, Tokenizer,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     ID,
///     Space,
///     Add,
///     Subtract,
///     PlusPlus,
///     MinusMinus,
///     EOF,
/// }
///
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
///
/// let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();
/// let identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
/// let punctuations: Punctuations<Token> = Punctuations::new(vec![
///     ("+", Token::Add),
///     ("++", Token::PlusPlus),
///     ("--", Token::MinusMinus),
///     ("-", Token::Subtract),
/// ])
/// .unwrap();
///
/// let tokenizer = Tokenizer::new(vec![
///     Rc::new(punctuations),
///     Rc::new(space),
///     Rc::new(identifier),
/// ]);
/// let lex = tokenizer.tokenize(&Code::from("a+++b")).unwrap();
/// assert_eq!(
///     lex,
///     vec![
///         Lex { token: Token::ID, start: 0, end: 1 },
///         Lex { token: Token::PlusPlus, start: 1, end: 3 },
///         Lex { token: Token::Add, start: 3, end: 4 },
///         Lex { token: Token::ID, start: 4, end: 5 },
///         Lex { token: Token::EOF, start: 5, end: 5 }
///     ]
/// );
/// let lex = tokenizer.tokenize(&Code::from("a+ ++b")).unwrap();
/// assert_eq!(
///     lex,
///     vec![
///         Lex { token: Token::ID, start: 0, end: 1 },
///         Lex { token: Token::Add, start: 1, end: 2 },
///         Lex { token: Token::Space, start: 2, end: 3 },
///         Lex { token: Token::PlusPlus, start: 3, end: 5 },
///         Lex { token: Token::ID, start: 5, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 }
///     ]
/// );
/// ```
pub struct Punctuations<TToken, TState = u8> {
    field_tree: FieldTree<TToken>,
    punctuations: Vec<(String, TToken)>,
    log: OnceCell<Log<&'static str>>,
    _state: PhantomData<TState>,
}

/// A lexer utility to match a set of string values like keywords, and constant values.  
///
/// All the provided string values will be matched sequentially with the input string at the incremental positions
/// and the corresponding token value will be returned as token data.
pub struct Constants<TToken, TState = u8> {
    values: Vec<(String, TToken)>,
    log: OnceCell<Log<&'static str>>,
    _state: PhantomData<TState>,
}

/// A lexical utility that transforms tokenized data based on the mapped string fields.
///
/// The associated lexeme utility will first be matched with the input string.
/// Once the associated lexeme utility successfully obtains token data,
/// it will then look for the appropriate token value for the tokenized string part of the input.
/// If no match is found for the corresponding tokenized string part, the original tokenized data will be returned.
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{Mapper, Pattern},
///     Code,
///     ITokenization, Lex, TokenImpl, Tokenizer,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     ID,
///     IF,
///     ELSE,
///     FOR,
///     Space,
///     True,
///     False,
///     EOF,
/// }
///
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
/// let id_lexer: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
/// let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();
///
/// let mapped_lexer = Mapper::new(
///     id_lexer,
///     vec![
///         ("if", Token::IF),
///         ("else", Token::ELSE),
///         ("for", Token::FOR),
///         ("true", Token::True),
///         ("false", Token::False),
///     ],
/// )
/// .unwrap();
///
/// let tokenizer = Tokenizer::new(vec![Rc::new(mapped_lexer), Rc::new(space)]);
/// let lex_stream = tokenizer.tokenize(&Code::from("abc xy")).unwrap();
///
/// assert_eq!(
///     lex_stream,
///     vec![
///         Lex { token: Token::ID, start: 0, end: 3 },
///         Lex { token: Token::Space, start: 3, end: 4 },
///         Lex { token: Token::ID, start: 4, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 }
///     ]
/// );
/// let lex = tokenizer.tokenize(&Code::from("if true")).unwrap();
/// assert_eq!(
///     lex,
///     vec![
///         Lex { token: Token::IF, start: 0, end: 2 },
///         Lex { token: Token::Space, start: 2, end: 3 },
///         Lex { token: Token::True, start: 3, end: 7 },
///         Lex { token: Token::EOF, start: 7, end: 7 }
///     ]
/// );
///
/// ```
pub struct Mapper<TLexer: ILexeme> {
    lexeme: TLexer,
    log: OnceCell<Log<&'static str>>,
    fields: HashMap<Vec<u8>, TLexer::Token>,
}

/// A lexical utility which transform tokenized data based on the provided closure function.
///
/// It is similar to [Mapper] however, optional transformed token will be received by executing the associated closure function,
/// The tokenizer will received original token if [None] value returned from the closure.
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{Pattern, ThunkMapper},
///     Code,
///     ITokenization, Lex, TokenImpl, Tokenizer,
/// };
/// use std::{io::BufRead, rc::Rc};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     InlineComment,
///     MultilineComment,
///     EOF,
/// }
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
/// let comment: Pattern<Token> = Pattern::new(Token::InlineComment, r#"^/\*(.|\n)*?\*/"#).unwrap();
///
/// let comment_variants = ThunkMapper::new(comment, |data, code, _| {
///     if code[data.start..data.end].lines().count() > 1 {
///         Some(Token::MultilineComment)
///     } else {
///         None
///     }
/// });
///
/// let tokenizer = Tokenizer::new(vec![Rc::new(comment_variants)]);
/// let inline_comment = "/*This is inline comment*/";
/// let inline_comment_tokens = tokenizer.tokenize(&Code::from(inline_comment)).unwrap();
/// assert_eq!(
///     inline_comment_tokens,
///     vec![
///         Lex { token: Token::InlineComment, start: 0, end: inline_comment.len() },
///         Lex { token: Token::EOF, start: inline_comment.len(), end: inline_comment.len() }
///     ]
/// );
/// let multiline_comment = "/*This is first line\n.Another line comment*/";
/// let multiline_comment_tokens = tokenizer.tokenize(&Code::from(multiline_comment)).unwrap();
/// assert_eq!(
///     multiline_comment_tokens,
///     vec![
///         Lex { token: Token::MultilineComment, start: 0, end: multiline_comment.len() },
///         Lex { token: Token::EOF, start: multiline_comment.len(), end: multiline_comment.len() }
///     ]
/// );
///
/// ```
pub struct ThunkMapper<
    TL: ILexeme,
    TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Option<TL::Token>,
> {
    lexeme: TL,
    log: OnceCell<Log<&'static str>>,
    thunk: TF,
}

/// A lexeme utility which will try to tokenize the input once associated middleware function returns truthy.
///
/// The closure function will be executed before creating token by the associated lexeme utility.
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{Middleware, Pattern, Punctuations},
///     Code,
///     ITokenization, Lex, TokenImpl, Tokenizer,
/// };
/// use std::rc::Rc;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     RegexLiteral,
///     ID,
///     Number,
///     Add,
///     Mul,
///     Div,
///     Assign,
///     Subtract,
///     EOF,
/// }
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
/// let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
/// let number_literal =
///     Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());
///
/// let punctuations = Rc::new(
///     Punctuations::new(vec![
///         ("+", Token::Add),
///         ("*", Token::Mul),
///         ("/", Token::Div),
///         ("=", Token::Assign),
///         ("-", Token::Subtract),
///     ])
///     .unwrap(),
/// );
///
/// let regex_literal =
///     Pattern::new(Token::RegexLiteral, r"^/([^\\/\r\n\[]|\\.|\[[^]]+\])+/").unwrap();
///
/// let validated_regex_literal = Rc::new(Middleware::new(regex_literal, |_, lex_stream| {
///     lex_stream.last().map_or(false, |d| match d.token {
///         Token::ID | Token::Number => false,
///         _ => true,
///     })
/// }));
///
/// let tokenizer = Tokenizer::new(vec![
///     identifier,
///     number_literal,
///     validated_regex_literal, // Should appear before punctuation so that regex literal is validated before div '/'.
///     punctuations,
/// ]);
///
/// let lex = tokenizer.tokenize(&Code::from("2/xy/6")).unwrap();
/// assert_eq!(
///     lex,
///     [
///         Lex { token: Token::Number, start: 0, end: 1 },
///         Lex { token: Token::Div, start: 1, end: 2 },
///         Lex { token: Token::ID, start: 2, end: 4 },
///         Lex { token: Token::Div, start: 4, end: 5 },
///         Lex { token: Token::Number, start: 5, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 },
///     ]
/// );
/// let regex_lex = tokenizer.tokenize(&&Code::from("a=/xy/")).unwrap();
/// assert_eq!(
///     regex_lex,
///     [
///         Lex { token: Token::ID, start: 0, end: 1 },
///         Lex { token: Token::Assign, start: 1, end: 2 },
///         Lex { token: Token::RegexLiteral, start: 2, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 },
///     ]
/// );
/// ```
pub struct Middleware<TLexeme: ILexeme, TMiddleware: Fn(&[u8], &Vec<Lex<TLexeme::Token>>) -> bool> {
    lexeme: TLexeme,
    log_label: OnceCell<Log<&'static str>>,
    middleware: TMiddleware,
}

/// A lexeme utility to modify state stack base on the provided [Action] corresponding to the tokens.
///
/// Once the associated lexeme utility create a token,
/// the utility will change the state stack based on [Action] for the state based [tokenizer](crate::CombinedTokenizer).
///
/// # Example
///
/// ```
/// use lang_pt::{
///     lexeme::{Action, Pattern, Punctuations, StateMixin},
///     Code,
///     CombinedTokenizer, ITokenization, Lex, TokenImpl,
/// };
/// use std::rc::Rc;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     ID,
///     Number,
///     Add,
///     Assign,
///     Subtract,
///     EOF,
///     TemplateTick,
///     TemplateExprStart,
///     TemplateString,
///     OpenBrace,
///     CloseBrace,
/// }
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
/// const MAIN: u8 = 0;
/// const TEMPLATE: u8 = 1;
/// let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
/// let number_literal =
///     Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());
///
/// let expression_punctuation = Punctuations::new(vec![
///     ("+", Token::Add),
///     ("-", Token::Subtract),
///     ("=", Token::Assign),
///     ("{", Token::OpenBrace),
///     ("}", Token::CloseBrace),
///     ("`", Token::TemplateTick),
/// ])
/// .unwrap();
///
/// let expr_punctuation_mixin = Rc::new(StateMixin::new(
///     expression_punctuation,
///     vec![
///         (Token::TemplateTick, Action::append(TEMPLATE, false)), // Encountering a TemplateTick (`) indicates beginning of template literal.
///         // While tokenizing in the template literal expression we are going to augment stack to keep track of open and close brace.
///         (Token::OpenBrace, Action::append(MAIN, false)),
///         (Token::CloseBrace, Action::remove(false)),
///     ],
/// ));
///
/// let lex_template_string: Rc<Pattern<Token>> = Rc::new(
///     Pattern::new(
///         Token::TemplateString,
///         r"^([^`\\$]|\$[^{^`\\$]|\\[${`bfnrtv])+",
///     )
///     .unwrap(),
/// );
///
/// let template_punctuations = Punctuations::new(vec![
///     ("`", Token::TemplateTick),
///     ("${", Token::TemplateExprStart),
/// ])
/// .unwrap();
/// let template_punctuation_mixin = StateMixin::new(
///     template_punctuations,
///     vec![
///         (Token::TemplateTick, Action::remove(false)), // Encountering TemplateTick (`) indicates end of template literal state.
///         (Token::TemplateExprStart, Action::append(MAIN, false)),
///     ],
/// );
///
/// let mut combined_tokenizer = CombinedTokenizer::new(
///     MAIN,
///     vec![identifier, number_literal, expr_punctuation_mixin],
/// );
/// combined_tokenizer.add_state(
///     TEMPLATE,
///     vec![Rc::new(template_punctuation_mixin), lex_template_string],
/// );
///
/// let token_stream = combined_tokenizer
///     .tokenize(&Code::from("d=`Sum is ${a+b}`"))
///     .unwrap();
/// debug_assert_eq!(
///     token_stream,
///     vec![
///         Lex::new(Token::ID, 0, 1),
///         Lex::new(Token::Assign, 1, 2),
///         Lex::new(Token::TemplateTick, 2, 3),
///         Lex::new(Token::TemplateString, 3, 10),
///         Lex::new(Token::TemplateExprStart, 10, 12,),
///         Lex::new(Token::ID, 12, 13),
///         Lex::new(Token::Add, 13, 14),
///         Lex::new(Token::ID, 14, 15),
///         Lex::new(Token::CloseBrace, 15, 16),
///         Lex::new(Token::TemplateTick, 16, 17),
///         Lex::new(Token::EOF, 17, 17),
///     ]
/// );
///
/// ```

pub struct StateMixin<TLexeme: ILexeme> {
    lexeme: TLexeme,
    log: OnceCell<Log<&'static str>>,
    actions: Vec<(TLexeme::Token, Action<TLexeme::State>)>,
}

/// A lexeme utility to modify state stack based on [Action] received from the closure function.
///
/// This similar to [StateMixin] however, [Action] is received from the closure function.
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{Action, Pattern, Punctuations, ThunkStateMixin},
///     Code,
///     ITokenization, Lex, TokenImpl, Tokenizer,
/// };
/// use std::rc::Rc;
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     RegexLiteral,
///     ID,
///     Number,
///     Add,
///     Mul,
///     Div,
///     Assign,
///     Subtract,
///     EOF,
/// }
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool { *self != Self::EOF }
/// }
/// let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
/// let number_literal =
///     Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());
///
/// let punctuations = Punctuations::new(vec![
///     ("+", Token::Add),
///     ("*", Token::Mul),
///     ("/", Token::Div),
///     ("=", Token::Assign),
///     ("-", Token::Subtract),
/// ])
/// .unwrap();
///
/// let punctuation_mixin = Rc::new(ThunkStateMixin::new(
///     punctuations,
///     |lex_data, _code, stream| {
///         if lex_data.token == Token::Div {
///             let is_expr_continuation =
///                 stream
///                     .last()
///                     .map_or(false, |pre_data| match pre_data.token {
///                         Token::ID | Token::Number => true,
///                         _ => false,
///                     });
///             Action::None {
///                 discard: !is_expr_continuation,
///             } // If the symbol '/' immediately after id or number it is a div element.
///               // Otherwise discard the lexeme if it is part of regex expression
///         } else {
///             Action::None { discard: false }
///         }
///     },
/// ));
///
/// let regex_literal =
///     Rc::new(Pattern::new(Token::RegexLiteral, r"^/([^\\/\r\n\[]|\\.|\[[^]]+\])+/").unwrap());
///
/// let tokenizer = Tokenizer::new(vec![
///     identifier,
///     number_literal,
///     punctuation_mixin,
///     regex_literal, // Should appear after punctuation so that it will be checked once div '/' is rejected.
/// ]);
///
/// let lex = tokenizer.tokenize(&Code::from("2/xy/6")).unwrap();
/// assert_eq!(
///     lex,
///     [
///         Lex { token: Token::Number, start: 0, end: 1 },
///         Lex { token: Token::Div, start: 1, end: 2 },
///         Lex { token: Token::ID, start: 2, end: 4 },
///         Lex { token: Token::Div, start: 4, end: 5 },
///         Lex { token: Token::Number, start: 5, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 },
///     ]
/// );
/// let regex_lex = tokenizer.tokenize(&Code::from("a=/xy/")).unwrap();
/// assert_eq!(
///     regex_lex,
///     [
///         Lex { token: Token::ID, start: 0, end: 1 },
///         Lex { token: Token::Assign, start: 1, end: 2 },
///         Lex { token: Token::RegexLiteral, start: 2, end: 6 },
///         Lex { token: Token::EOF, start: 6, end: 6 },
///     ]
/// );
///
/// ```
pub struct ThunkStateMixin<
    TL: ILexeme,
    TF: Fn(&Lex<TL::Token>, &[u8], &Vec<Lex<TL::Token>>) -> Action<TL::State>,
> {
    lexeme: TL,
    log: OnceCell<Log<&'static str>>,
    thunk_action: TF,
}

/// A trait implementation utility to convert one lexeme utility to another higher order lexeme utility.
///
/// The trait is implemented for generic [ILexeme] types.
/// Therefore, the associated methods are available for all utilities which implement [ILexeme] trait.
pub trait LexemeBuilder: ILexeme {
    fn mapping(self, mapping: Vec<(&str, Self::Token)>) -> Result<Mapper<Self>, String>
    where
        Self: Sized;
    fn thunk_mapping<
        TF: Fn(&Lex<Self::Token>, &[u8], &Vec<Lex<Self::Token>>) -> Option<Self::Token>,
    >(
        self,
        f: TF,
    ) -> ThunkMapper<Self, TF>
    where
        Self: Sized;
    fn state_mixin(self, actions: Vec<(Self::Token, Action<Self::State>)>) -> StateMixin<Self>
    where
        Self: Sized;
    fn middleware<TM: Fn(&[u8], &Vec<Lex<Self::Token>>) -> bool>(
        self,
        middleware: TM,
    ) -> Middleware<Self, TM>
    where
        Self: Sized;
    fn thunk_mixin<
        TM: Fn(&Lex<Self::Token>, &[u8], &Vec<Lex<Self::Token>>) -> Action<Self::State>,
    >(
        self,
        middleware: TM,
    ) -> ThunkStateMixin<Self, TM>
    where
        Self: Sized;
}
