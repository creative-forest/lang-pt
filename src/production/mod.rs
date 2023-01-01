//! A module consist of production utilities which are helper utilities to write grammar for the parser.
//!
//!
//! Each production utility represent a defined rule of operation for a set symbols.
//! As an example, non terminal production utility [Concat], represents concatenation associated symbols,
//! where as, [Union], will use the first production match from the set of alternative symbols.
//! The terminal utilities like [TokenField], [TokenFieldSet], will match the input token received from the tokenizer.  
//! where the production utilities like [RegexField]
//! [PunctuationsField], [ConstantField] will match string values.
//! Therefore the former utilities can be used to create a [LexerlessParser](crate::LexerlessParser)
//! which does not need to define a tokenizer.
//!
mod builder;
mod non_terminals;
mod terminals;
mod wrappers;
use once_cell::unsync::OnceCell;
use regex::bytes::Regex;
use std::{marker::PhantomData, rc::Rc};

#[cfg(test)]
mod __tests__;

use crate::{
    util::{Code, Log},
    ASTNode, CacheKey, FieldTree, FltrPtr, IProduction, NodeImpl, ParsedResult, ProductionError,
    TokenPtr, TokenImpl, TokenStream,
};

/// A terminal symbol which matches a given token with the input.
pub struct TokenField<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    token: TL,
    node_value: Option<TN>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A terminal symbol which matches any one token from the provided set of tokens.
pub struct TokenFieldSet<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    token_set: Vec<(TL, Option<TN>)>,
    debugger: OnceCell<Log<&'static str>>,
    rule_name: OnceCell<&'static str>,
}

/// A terminal symbol which matches the provided regex expression with the input.
pub struct RegexField<TN: NodeImpl = u8, TT = i8> {
    regexp: Regex,
    node_value: Option<TN>,
    _token: PhantomData<TT>,
    debugger: OnceCell<Log<&'static str>>,
    rule_name: OnceCell<&'static str>,
}

/// A terminal symbol which matches the provided value with the input.
pub struct ConstantField<TN: NodeImpl = u8, TT = i8> {
    value: Vec<u8>,
    node_value: Option<TN>,
    _phantom_data: PhantomData<TT>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A terminal symbol which matches a set of punctuation field with the input.
pub struct PunctuationsField<TN: NodeImpl = u8, TT = i8> {
    tree: FieldTree<Option<TN>>,
    rule_name: OnceCell<&'static str>,
    values: Vec<(String, Option<TN>)>,
    _phantom_data: PhantomData<TT>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A terminal symbol which matches a set of string values with the input.

pub struct ConstantFieldSet<TN: NodeImpl = u8, TT = i8> {
    fields: Vec<(Vec<u8>, Option<TN>)>,
    rule_name: OnceCell<&'static str>,
    debugger: OnceCell<Log<&'static str>>,
    _token: PhantomData<TT>,
}
#[derive(Clone)]
/// A null production symbol for the grammar.
pub struct NullProd<TN: NodeImpl = u8, TL = i8> {
    debugger: OnceCell<Log<&'static str>>,
    _node: PhantomData<TN>,
    _token: PhantomData<TL>,
}

/// A terminal component which matches End of File(EOF) symbol.
pub struct EOFProd<TN: NodeImpl = u8, TL = i8> {
    node_value: Option<TN>,
    debugger: OnceCell<Log<&'static str>>,
    _token: PhantomData<TL>,
}

struct NTHelper {
    identifier: &'static str,
    nullability: OnceCell<bool>,
    null_hidden: OnceCell<bool>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A non-terminal production utility to derive concatenation of production symbols.
///
/// The production utility will try to parse all children symbols in series.
/// Once all the child productions are successfully parsed, it will return a vec of flattened tree nodes([ASTNode]).
///   
/// The general form for union production is
/// E -> X<sub>1</sub> X<sub>2</sub> X<sub>3</sub>... X<sub>n</sub>.
/// where, X<sub>i</sub>, i=1..n can be a non-terminal or terminal production.
/// # Example
/// ```
/// use lang_pt::{
///     production::{Concat, ConstantField, EOFProd, Node, PunctuationsField, RegexField},
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// pub enum NodeValue {
///     ID,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     NULL,
///     Expr,
///     Root,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let operators = Rc::new(
///     PunctuationsField::new(vec![
///         ("+", Some(NodeValue::Add)),
///         ("-", Some(NodeValue::Sub)),
///         ("*", Some(NodeValue::Mul)),
///         ("/", Some(NodeValue::Div)),
///     ])
///     .unwrap(),
/// );
/// let open_paren = Rc::new(ConstantField::new("(", None));
/// let close_paren = Rc::new(ConstantField::new(")", None));
///
/// let expression = Rc::new(Concat::new(
///     "Expression",
///     vec![id.clone(), operators.clone(), id.clone()],
/// ));
///
/// let expression_node = Rc::new(Node::new(&expression, Some(NodeValue::Expr)));
///
/// let parenthesis_expression = Rc::new(Concat::new(
///     "Parenthesis_Expression",
///     vec![
///         open_paren.clone(),
///         expression_node.clone(),
///         close_paren.clone(),
///     ],
/// ));
///
/// let root = Rc::new(Concat::new("main", vec![parenthesis_expression, eof]));
///
/// let root_node = Rc::new(Node::new(&root, Some(NodeValue::Root)));
///
/// let parser = LexerlessParser::new(root_node).unwrap();
///
/// let tree_list = parser.parse(b"(ax+by)").unwrap();
/// tree_list.last().unwrap().print().unwrap();
/// /*
/// Root # 0-7
/// └─ Expr # 1-6
///    ├─ ID # 1-3
///    ├─ Add # 3-4
///    └─ ID # 4-6
///  */
/// ```
pub struct Concat<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    symbols: OnceCell<Vec<Rc<dyn IProduction<Node = TN, Token = TL>>>>,
    nt_helper: NTHelper,
}

/// A non-terminal utility to implement alternative derivations of productions.
///
/// The production utility will try to parse each associated symbol and return first successful parsed tree.
///
/// The general form for union production is
/// X -> Y<sub>1</sub> | Y<sub>2</sub> | Y<sub>3</sub>|... Y<sub>n</sub>.
/// where, Y<sub>i</sub>, i=1..n can be a non-terminal or terminal production.
///
/// # Example
/// ```
/// use lang_pt::production::{EOFProd, Node};
/// use lang_pt::NodeImpl;
/// use lang_pt::{
///     production::{Concat, ConstantField, RegexField, Union},
///     LexerlessParser,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     ID,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     NULL,
///     Root
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let add = Rc::new(ConstantField::new("+", Some(NodeValue::Add)));
/// let sub = Rc::new(ConstantField::new("-", Some(NodeValue::Sub)));
/// let mul = Rc::new(ConstantField::new("*", Some(NodeValue::Mul)));
/// let div = Rc::new(ConstantField::new("/", Some(NodeValue::Div)));
/// let addition = Rc::new(Concat::new(
///     "addition",
///     vec![id.clone(), add.clone(), id.clone()],
/// ));
/// let subtraction = Rc::new(Concat::new(
///     "subtraction",
///     vec![id.clone(), sub.clone(), id.clone()],
/// ));
/// let multiplication = Rc::new(Concat::new(
///     "multiplication",
///     vec![id.clone(), mul.clone(), id.clone()],
/// ));
/// let division = Rc::new(Concat::new(
///     "division",
///     vec![id.clone(), div.clone(), id.clone()],
/// ));
/// let expression = Rc::new(Union::new(
///     "expression",
///     vec![addition, subtraction, multiplication, division],
/// ));
///
/// let main = Rc::new(Concat::new("main", vec![expression, eof]));
/// let main_node = Rc::new(Node::new(&main, Some(NodeValue::Root)));
///
/// let parser = LexerlessParser::new(main_node).unwrap();
/// let tree_list = parser.parse(b"ax+by").unwrap();
/// tree_list.last().unwrap().print().unwrap();
/// /*
/// Root # 0-5
/// ├─ ID # 0-2
/// ├─ Add # 2-3
/// └─ ID # 3-5
///  */
///
/// ```
pub struct Union<TN: NodeImpl = u8, TL: TokenImpl = i8> {
    symbols: OnceCell<Vec<Rc<dyn IProduction<Node = TN, Token = TL>>>>,
    nt_helper: NTHelper,
    first_set: OnceCell<(bool, Vec<(TL, Vec<usize>)>)>,
}

pub type TSuffixMap<TN, TL> = (Rc<dyn IProduction<Node = TN, Token = TL>>, TN);

/// A production utility to parse multiple tails/end symbols for same body/starting symbol.
///
/// Once the associated starting symbol is successfully parsed, each tails will be tried to parse sequentially
/// and return combined parsed tree on the first success.
///
/// The general form for this production is
/// E -> X Y<sub>1</sub> | X Y<sub>2</sub> | ... X Y<sub>n</sub> where, X and Y<sub>1</sub>..<sub>n</sub>, are non-terminal or terminal symbols.
/// The utility will first try to parse X.
/// The tails of the production Y<sub>1</sub>..<sub>n</sub> will then be sequentially tried to parse until it encounter first success.
/// The right most tail production Y<sub></sub> can also be a null production (ε) for standalone [Suffixes].
/// # Example
///
/// ```
/// use lang_pt::production::{ConstantField, ProductionBuilder};
/// use lang_pt::NodeImpl;
/// use lang_pt::{
///     production::{Concat, RegexField, SeparatedList, Suffixes, Union},
///     LexerlessParser,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     ID,
///     Number,
///     ArrayAccess,
///     FunctionCall,
///     FuncArgs,
///     NULL,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let number = Rc::new(
///     RegexField::new(
///         r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?",
///         Some(NodeValue::Number),
///     )
///     .unwrap(),
/// );
///
/// let id_or_number = Rc::new(Union::new("ID_or_Number", vec![id.clone(), number]));
///
/// let comma = Rc::new(ConstantField::new(",", None));
/// let open_bracket = Rc::new(ConstantField::new("[", None));
/// let close_bracket = Rc::new(ConstantField::new("]", None));
/// let open_paren = Rc::new(ConstantField::new("(", None));
/// let close_paren = Rc::new(ConstantField::new(")", None));
///
/// let array_index = Rc::new(Concat::new(
///     "ArrayIndex",
///     vec![
///         open_bracket.clone(),
///         id_or_number.clone(),
///         close_bracket.clone(),
///     ],
/// ));
///
/// let function_arguments = Rc::new(
///     SeparatedList::new(&id_or_number, &comma, false).into_node(Some(NodeValue::FuncArgs)),
/// );
///
/// let function_call = Rc::new(Concat::new(
///     "FunctionCall",
///     vec![open_paren, function_arguments, close_paren],
/// ));
///
/// let array_index_or_func_call = Rc::new(Suffixes::new(
///     "ArrayOrFuncCall",
///     &id,
///     false,
///     vec![
///         (array_index, NodeValue::ArrayAccess),
///         (function_call, NodeValue::FunctionCall),
///     ],
/// ));
///
/// let parser = LexerlessParser::new(array_index_or_func_call).unwrap();
///
/// let array_tree = parser.parse(b"arr[b]").unwrap();
/// array_tree[0].print().unwrap();
/// /*
/// ArrayAccess # 0-6
/// ├─ ID # 0-3
/// └─ ID # 4-5
/// */
///
/// let function_call_tree = parser.parse(b"func(arg1,arg2,arg3)").unwrap();
/// function_call_tree[0].print().unwrap();
/// /*
/// FunctionCall # 0-20
/// ├─ ID # 0-4
/// └─ FuncArgs # 5-19
///    ├─ ID # 5-9
///    ├─ ID # 10-14
///    └─ ID # 15
/// */
///
/// ```
pub struct Suffixes<TP: IProduction> {
    left: Rc<TP>,
    standalone: bool,
    suffixes: OnceCell<Vec<TSuffixMap<TP::Node, TP::Token>>>,
    nt_helper: NTHelper,
    suffix_first_set: OnceCell<(bool, Vec<(TP::Token, Vec<usize>)>)>,
    null_suffix_index: OnceCell<Option<usize>>,
}

/// An utility to parse a terminal or non-terminal symbols one or multiple times.
///
/// The general form for this production is
/// E -> X+ where, X can be a non-terminal or terminal symbol.
/// # Example
/// ```
/// use lang_pt::{
///     production::{Concat, EOFProd, List, ProductionBuilder, PunctuationsField, RegexField},
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     ID,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     UnaryList,
///     Expr,
///     Root,
///     NULL,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let operators = Rc::new(
///     PunctuationsField::new(vec![
///         ("+", Some(NodeValue::Add)),
///         ("-", Some(NodeValue::Sub)),
///         ("*", Some(NodeValue::Mul)),
///         ("/", Some(NodeValue::Div)),
///     ])
///     .unwrap(),
/// );
/// let unary_operators = Rc::new(
///     PunctuationsField::new(vec![
///         ("+", Some(NodeValue::Add)),
///         ("-", Some(NodeValue::Sub)),
///     ])
///     .unwrap(),
/// );
///
/// let unary_operators_list =
///     Rc::new(List::new(&unary_operators).into_node(Some(NodeValue::UnaryList)));
///
/// let expression = Rc::new(
///     Concat::new(
///         "Expression",
///         vec![
///             id.clone(),
///             operators.clone(),
///             unary_operators_list.clone(),
///             id.clone(),
///         ],
///     )
///     .into_node(Some(NodeValue::Expr)),
/// );
/// let root = Rc::new(Concat::new("root", vec![expression, eof]).into_node(Some(NodeValue::Root)));
///
/// let parser = LexerlessParser::new(root).unwrap();
///
/// let tree_list1 = parser.parse(b"ax*+by").unwrap();
/// tree_list1.iter().for_each(|tree| {
///     tree.print().unwrap();
/// });
///
/// let tree_list2 = parser.parse(b"ax*+-by").unwrap();
/// tree_list2.iter().for_each(|tree| {
///     tree.print().unwrap();
/// });
/// ```
pub struct List<TProd: IProduction> {
    symbol: Rc<TProd>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A production utility to parse list of terminal or non-terminal symbols separated by another symbol.
///
/// The general form for this production is
/// E -> X s X s....X s? where, X and s can be a non-terminal or terminal symbol.
/// The production can be non-inclusive to enforce symbol X to be at end of the production.
/// # Example
/// ```
/// use lang_pt::production::ProductionBuilder;
/// use lang_pt::{
///     production::{Concat, ConstantField, EOFProd, RegexField, SeparatedList, Union},
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     ID,
///     Number,
///     NULL,
///     Array,
///     Main,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
///
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let number = Rc::new(
///     RegexField::new(
///         r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?",
///         Some(NodeValue::Number),
///     )
///     .unwrap(),
/// );
///
/// let id_or_number = Rc::new(Union::new("id_or_Number", vec![id, number]));
/// let comma = Rc::new(ConstantField::new(",", None));
/// let open_bracket = Rc::new(ConstantField::new("[", None));
/// let close_bracket = Rc::new(ConstantField::new("]", None));
///
/// let array_items = Rc::new(SeparatedList::new(&id_or_number, &comma, false));
///
/// let array_literal = Rc::new(
///     Concat::new(
///         "ArrayLiteral",
///         vec![open_bracket, array_items, close_bracket],
///     )
///     .into_node(Some(NodeValue::Array)),
/// );
///
/// let main =
///     Rc::new(Concat::new("main", vec![array_literal, eof]).into_node(Some(NodeValue::Main)));
///
/// let parser = LexerlessParser::new(main).unwrap();
///
/// parser
///     .parse(b"[a,b,]")
///     .expect_err("Non-inclusive SeparatedList should fail to parse last comma(,)");
///
/// let tree_list = parser.parse(b"[a,b,2,3,c,4]").unwrap();
/// tree_list[0].print().unwrap();
/// /*
/// Main # 0-13
/// └─ Array # 0-13
///    ├─ ID # 1-2
///    ├─ ID # 3-4
///    ├─ Number # 5-6
///    ├─ Number # 7-8
///    ├─ ID # 9-10
///    └─ Number # 11-12
/// */
///
/// ```
pub struct SeparatedList<TP: IProduction, TS: IProduction<Node = TP::Node, Token = TP::Token>> {
    rule_name: OnceCell<&'static str>,
    production: Rc<TP>,
    separator: Rc<TS>,
    inclusive: bool,
    debugger: OnceCell<Log<&'static str>>,
}

/// A production utility which add null production as alternative symbol.
/// The general form for this production is
/// E -> X | ε where, X and s can be a non-terminal or terminal symbol and ε is a null terminal symbol.
///
/// # Example
/// ```
/// use lang_pt::{
///     production::{
///         Concat, EOFProd, List, Nullable, ProductionBuilder, PunctuationsField, RegexField,
///     },
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum Token {
///     ID,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     NULL,
///     UnaryList,
///     Expr,
///     Main,
/// }
///
/// impl NodeImpl for Token {
///     fn null() -> Self { Self::NULL }
/// }
///
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(Token::ID)).unwrap());
/// let operators = Rc::new(
///     PunctuationsField::new(vec![
///         ("+", Some(Token::Add)),
///         ("-", Some(Token::Sub)),
///         ("*", Some(Token::Mul)),
///         ("/", Some(Token::Div)),
///     ])
///     .unwrap(),
/// );
///
/// let unary_operators = Rc::new(
///     PunctuationsField::new(vec![("+", Some(Token::Add)), ("-", Some(Token::Sub))]).unwrap(),
/// );
///
/// let unary_operators_list =
///     Rc::new(List::new(&unary_operators).into_node(Some(Token::UnaryList)));
///
/// let nullable_unary_operator_list = Rc::new(Nullable::new(&unary_operators_list));
///
/// let expression = Rc::new(
///     Concat::new(
///         "Expression",
///         vec![
///             id.clone(),
///             operators.clone(),
///             nullable_unary_operator_list.clone(),
///             id.clone(),
///         ],
///     )
///     .into_node(Some(Token::Expr)),
/// );
/// let main = Rc::new(Concat::new("main", vec![expression, eof]).into_node(Some(Token::Main)));
///
/// let parser = LexerlessParser::new(main).unwrap();
///
/// let tree_list1 = parser.parse(b"ax+by").unwrap();
/// tree_list1[0].print().unwrap();
///
/// /*
/// Main # 0-5
/// └─ Expr # 0-5
///    ├─ ID # 0-2
///    ├─ Add # 2-3
///    ├─ NULL # 3-3
///    └─ ID # 3-5
/// */
///
/// let tree_list2 = parser.parse(b"ax*+-by").unwrap();
/// tree_list2[0].print().unwrap();
/// /*
/// Main # 0-7
/// └─ Expr # 0-7
///    ├─ ID # 0-2
///    ├─ Mul # 2-3
///    ├─ UnaryList # 3-5
///    │  ├─ Add # 3-4
///    │  └─ Sub # 4-5
///    └─ ID # 5-7
/// */
///
/// ```
pub struct Nullable<TP: IProduction> {
    production: Rc<TP>,
    debugger: OnceCell<Log<&'static str>>,
}

/// An utility to create a [AST](crate::ASTNode) node from the parsed children.
///
/// The [None] node value will hide the children tree  i.e. it will remove the children from the [ASTNode].
/// The non-terminal production will flatten the parsed tree i.e. it will sequentially add all parsed children tree into a vector.
/// Therefore, this wrapper utility can be used to create a node which will then be appended as a child to the parent tree.   
/// # Example
/// ```
/// use lang_pt::{
///     production::{Concat, EOFProd, Node, PunctuationsField, RegexField},
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     NULL,
///     ID,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     Main,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let operators = Rc::new(
///     PunctuationsField::new(vec![
///         ("+", Some(NodeValue::Add)),
///         ("-", Some(NodeValue::Sub)),
///         ("*", Some(NodeValue::Mul)),
///         ("/", Some(NodeValue::Div)),
///     ])
///     .unwrap(),
/// );
///
/// let expression = Rc::new(Concat::new(
///     "Expression",
///     vec![id.clone(), operators.clone(), id.clone()],
/// ));
///
/// let main = Rc::new(Concat::new("Main", vec![expression.clone(), eof]));
/// let main_node = Rc::new(Node::new(&main, Some(NodeValue::Main)));
///
/// let parser = LexerlessParser::new(main_node).unwrap();
///
/// let tree_node = parser.parse(b"ax+by").unwrap();
/// tree_node[0].print().unwrap();
///
/// ```

pub struct Node<TP: IProduction> {
    rule_name: OnceCell<&'static str>,
    production: Rc<TP>,
    node_value: Option<TP::Node>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A production utility to validate the parsed data based on the associated closure function.
///
/// Once the associated production symbol returns success result the closure will then be executed to validate parsed result.
/// # Example
/// ```
/// use lang_pt::production::ConstantField;
/// use lang_pt::production::ProductionBuilder;
/// use lang_pt::NodeImpl;
/// use lang_pt::{
///     production::{Concat, EOFProd, RegexField, Validator},
///     LexerlessParser, ProductionError,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// enum NodeValue {
///     NULL,
///     TagName,
///     Text,
///     Root,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self { Self::NULL }
/// }
///
/// let eof = Rc::new(EOFProd::new(None));
/// let xml_tag =
///     Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::TagName)).unwrap());
/// let xml_text = Rc::new(RegexField::new(r#"^([^><]|\\[><])*"#, Some(NodeValue::Text)).unwrap());
///
/// let open_angle = Rc::new(ConstantField::new("<", None));
/// let close_angle = Rc::new(ConstantField::new(">", None));
///
/// let open_angle_slash = Rc::new(ConstantField::new("</", None));
///
/// let xml_element = Rc::new(Concat::new(
///     "xml_element",
///     vec![
///         open_angle.clone(),
///         xml_tag.clone(),
///         close_angle.clone(),
///         xml_text.clone(),
///         open_angle_slash.clone(),
///         xml_tag.clone(),
///         close_angle.clone(),
///     ],
/// ));
///
/// let validated_xml_element = Rc::new(Validator::new(&xml_element, |children, code| {
///     let start_tag = &code[children[0].start..children[0].end];
///     let end_tag = &code[children[2].start..children[2].end];
///     if start_tag != end_tag {
///         return Err(ProductionError::Validation(children[0].start, unsafe {
///             format!(
///                 "Mismatch xml start tag {} and end tag {}",
///                 std::str::from_utf8_unchecked(start_tag),
///                 std::str::from_utf8_unchecked(end_tag)
///             )
///         }));
///     }
///     Ok(())
/// }));
/// let root_node = Rc::new(
///     Concat::new("main", vec![validated_xml_element, eof]).into_node(Some(NodeValue::Root)),
/// );
///
/// let parser = LexerlessParser::new(root_node).unwrap();
///
/// parser
///     .parse(b"<span>This is text.</div>")
///     .expect_err("Should through a validation error");
/// let tree_node = parser.parse(b"<span>This is text.</span>").unwrap();
/// tree_node[0].print().unwrap();
/// /*
/// Root # 0-26
/// ├─ TagName # 1-5
/// ├─ Text # 6-19
/// └─ TagName # 21-25
/// */
///
/// ```
pub struct Validator<
    TP: IProduction,
    TF: Fn(&Vec<ASTNode<TP::Node>>, &[u8]) -> Result<(), ProductionError>,
> {
    validation_fn: TF,
    production: Rc<TP>,
    debugger: OnceCell<Log<&'static str>>,
}

#[derive(Clone)]
/// A production utility to peek and validate the associated symbol without consuming the input.
/// # Example
/// ```
/// use lang_pt::{
///     production::{Concat, ConstantField, EOFProd, Lookahead, ProductionBuilder, RegexField, Union},
///     LexerlessParser, NodeImpl,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// pub enum NodeValue {
///     ID,
///     Null,
///     KeywordVar,
///     KeywordLet,
///     KeywordConst,
///     TypingNumber,
///     LineTermination,
///     Root,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self {
///         Self::Null
///     }
/// }
///
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
/// let white_space = Rc::new(RegexField::new(r"^[^\S\r\n]+", None).unwrap());
///
/// let keyword_var = Rc::new(ConstantField::new("var", Some(NodeValue::KeywordVar)));
/// let keyword_let = Rc::new(ConstantField::new("let", Some(NodeValue::KeywordLet)));
/// let keyword_const = Rc::new(ConstantField::new("const", Some(NodeValue::KeywordConst)));
///
/// let declaration_type = Rc::new(Union::new(
///     "var_type",
///     vec![keyword_var, keyword_let, keyword_const],
/// ));
///
/// let typing_type_number = Rc::new(ConstantField::new("number", Some(NodeValue::TypingNumber)));
/// let typing_type_string = Rc::new(ConstantField::new("string", Some(NodeValue::KeywordLet)));
/// let typing_type_object = Rc::new(ConstantField::new("object", Some(NodeValue::KeywordConst)));
/// let typing_type_boolean = Rc::new(ConstantField::new("boolean", Some(NodeValue::KeywordConst)));
///
/// let typing_type_union = Rc::new(Union::new(
///     "typings",
///     vec![
///         typing_type_number,
///         typing_type_string,
///         typing_type_object,
///         typing_type_boolean,
///     ],
/// ));
///
/// let hidden_colon = Rc::new(ConstantField::new(":", None));
/// let semi_colon = Rc::new(ConstantField::new(";", None));
///
/// let typing_declaration = Rc::new(Concat::new(
///     "typing_declaration",
///     vec![hidden_colon.clone(), typing_type_union.clone()],
/// ));
///
/// let lookahead_eof = Rc::new(Lookahead::new(&eof, Some(NodeValue::LineTermination)));
///
/// let statement_termination = Rc::new(Union::new(
///     "statement_termination",
///     vec![semi_colon, lookahead_eof],
/// ));
///
/// let var_declaration = Rc::new(Concat::new(
///     "var_declaration",
///     vec![
///         declaration_type.clone(),
///         white_space.clone(),
///         id.clone(),
///         typing_declaration.clone(),
///         statement_termination.clone(),
///     ],
/// ));
///
/// let root =
///     Rc::new(Concat::new("main", vec![var_declaration, eof]).into_node(Some(NodeValue::Root)));
///
/// let parser = LexerlessParser::new(root).unwrap();
///
/// let tree_node = parser.parse(b"let ax:number;").unwrap();
/// tree_node[0].print().unwrap();
/// /*
/// Root # 0-14
/// ├─ KeywordLet # 0-3
/// ├─ ID # 4-6
/// └─ TypingNumber # 7-13
///  */
///
/// let nullable_typing_node = parser.parse(b"let ax:string").unwrap();
/// nullable_typing_node[0].print().unwrap();
/// /*
/// Root # 0-13
/// ├─ KeywordLet # 0-3
/// ├─ ID # 4-6
/// ├─ KeywordLet # 7-13
/// └─ LineTermination # 13-13
/// */
/// ```
pub struct Lookahead<TProd: IProduction> {
    production: Rc<TProd>,
    node_value: Option<TProd::Node>, // hidden: bool,
    debugger: OnceCell<Log<&'static str>>,
}

#[derive(Clone)]
/// A production utility which makes all its children production to consume input on non filtered token stream.
///
/// For most of the programing languages like  Javascript, CSS, HTML
/// it is wise to build a grammar ignoring the non structural elements like
/// whitespace, line-break from the input tokens to improve performance.
/// However, for language like Javascript a line break can also signify a grammatical value like expression termination.
/// Thus, in this similar production should be wrapped with NonStructural utility to consume non-structural lexical items of the productions.       
/// # Example
/// ```
/// use lang_pt::{
///     lexeme::{LexemeBuilder, Pattern, Punctuations},
///     production::{
///         Concat, EOFProd, List, Lookahead, NonStructural, ProductionBuilder, TokenField,
///         TokenFieldSet, Union,
///     },
///     DefaultParser, NodeImpl, TokenImpl, Tokenizer,
/// };
/// use std::rc::Rc;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// enum Token {
///     ID,
///     Number,
///     Add,
///     Sub,
///     Mul,
///     Div,
///     LT,
///     LTE,
///     GT,
///     GTE,
///     EQ,
///     Space,
///     Colon,
///     LineBreak,
///     Semicolon,
///     KeywordVar,
///     KeywordConst,
///     KeywordLet,
///     KeywordIf,
///     KeywordNumber,
///     KeywordString,
///     KeywordObject,
///     KeywordBoolean,
///     EOF,
///     Assign,
///     OpenBrace,
///     CloseBrace,
///     OpenParen,
///     CloseParen,
///     OpenBracket,
///     CloseBracket,
/// }
///
/// impl TokenImpl for Token {
///     fn eof() -> Self { Self::EOF }
///     fn is_structural(&self) -> bool {
///         match self {
///             Token::Space | Token::LineBreak => false,
///             _ => true,
///         }
///     }
/// }
/// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
/// pub enum NodeValue {
///     ID,
///     Null,
///     KeywordVar,
///     KeywordLet,
///     KeywordConst,
///     TypingNumber,
///     TypingString,
///     TypingBool,
///     TypingObject,
///     EOFTermination,
///     NewLine,
///     VarAssignment,
///     Root,
/// }
///
/// impl NodeImpl for NodeValue {
///     fn null() -> Self {
///         Self::Null
///     }
/// }
///
/// let mapped_identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#)
///     .unwrap()
///     .mapping(vec![
///         ("var", Token::KeywordVar),
///         ("const", Token::KeywordConst),
///         ("let", Token::KeywordLet),
///         ("if", Token::KeywordIf),
///         ("boolean", Token::KeywordBoolean),
///         ("number", Token::KeywordNumber),
///         ("object", Token::KeywordObject),
///         ("string", Token::KeywordString),
///     ])
///     .unwrap();
///
/// let number_literal =
///     Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap();
/// let non_break_space = Pattern::new(Token::Space, r"^[^\S\r\n]+").unwrap();
/// let line_break = Pattern::new(Token::LineBreak, r"^[\r\n]+").unwrap();
///
/// let expression_punctuations = Punctuations::new(vec![
///     ("+", Token::Add),
///     ("-", Token::Sub),
///     ("*", Token::Mul),
///     ("/", Token::Div),
///     ("<", Token::LT),
///     ("<=", Token::LTE),
///     (">", Token::GT),
///     (">=", Token::GTE),
///     ("==", Token::EQ),
///     ("=", Token::Assign),
///     ("{", Token::OpenBrace),
///     ("}", Token::CloseBrace),
///     ("(", Token::OpenParen),
///     (")", Token::CloseParen),
///     ("[", Token::OpenBracket),
///     ("]", Token::CloseBracket),
///     (";", Token::Semicolon),
///     (":", Token::Colon),
/// ])
/// .unwrap();
/// let tokenizer=Tokenizer::new(vec![
///     Rc::new(non_break_space),
///     Rc::new(line_break),
///     Rc::new(mapped_identifier),
///     Rc::new(number_literal),
///     Rc::new(expression_punctuations),
/// ]);
/// let eof = Rc::new(EOFProd::new(None));
/// let id = Rc::new(TokenField::new(Token::ID, Some(NodeValue::ID)));
///
/// let declaration_type = Rc::new(TokenFieldSet::new(vec![
///     (Token::KeywordVar, Some(NodeValue::KeywordVar)),
///     (Token::KeywordConst, Some(NodeValue::KeywordConst)),
///     (Token::KeywordLet, Some(NodeValue::KeywordLet)),
/// ]));
///
/// let typing_type_union = Rc::new(TokenFieldSet::new(vec![
///     (Token::KeywordNumber, Some(NodeValue::TypingNumber)),
///     (Token::KeywordBoolean, Some(NodeValue::TypingBool)),
///     (Token::KeywordObject, Some(NodeValue::TypingObject)),
///     (Token::KeywordString, Some(NodeValue::TypingString)),
/// ]));
///
/// let hidden_colon = Rc::new(TokenField::new(Token::Colon, None));
/// let semi_colon = Rc::new(TokenField::new(Token::Semicolon, None));
///
/// let typing_declaration = Rc::new(Concat::new(
///     "typing_declaration",
///     vec![hidden_colon.clone(), typing_type_union.clone()],
/// ));
///
/// let lookahead_eof = Rc::new(Lookahead::new(&eof, Some(NodeValue::EOFTermination)));
///
/// // A new line is also expression terminal for language like Javascript.
/// // However, the new line tokens are filtered for improving performance.
/// // Therefore, a NonStructural utility force the parsing on unfiltered tokens.
///
/// let hidden_null_white_space = Rc::new(
///     TokenField::new(Token::Space, None)
///         .into_nullable()
///         .into_node(None),// Will hide all children tree from AST.
/// );
///
/// let line_break = Rc::new(TokenField::new(Token::LineBreak, Some(NodeValue::NewLine)));
///
/// let line_break_seq = Rc::new(Concat::new(
///     "line_break_seq",
///     vec![hidden_null_white_space, line_break],
/// ));
///
/// let non_structural_line_break = Rc::new(NonStructural::new(&line_break_seq, false));
///
/// let statement_termination = Rc::new(Union::new(
///     "statement_termination",
///     vec![semi_colon, lookahead_eof, non_structural_line_break],
/// ));
///
/// let var_declaration = Rc::new(
///     Concat::new(
///         "var_declaration",
///         vec![
///             declaration_type.clone(),
///             id.clone(),
///             typing_declaration.clone(),
///             statement_termination.clone(),
///         ],
///     )
///     .into_node(Some(NodeValue::VarAssignment)),
/// );
/// let list_var_declaration = Rc::new(List::new(&var_declaration));
///
/// let root = Rc::new(
///     Concat::new("main", vec![list_var_declaration, eof]).into_node(Some(NodeValue::Root)),
/// );
///
/// let parser = DefaultParser::new(Rc::new(tokenizer), root).unwrap();
///
/// let code = r"
///     let ax:number
///     let ax:string
/// ";
///
/// let tree_node = parser.parse(code.as_bytes()).unwrap();
/// tree_node[0].print().unwrap();
/// /*
/// Root # 9-49
/// ├─ VarAssignment # 9-31
/// │  ├─ KeywordLet # 9-12
/// │  ├─ ID # 13-15
/// │  ├─ TypingNumber # 16-22
/// │  └─ NewLine # 22-23
/// └─ VarAssignment # 31-49
///    ├─ KeywordLet # 31-34
///    ├─ ID # 35-37
///    ├─ TypingString # 38-44
///    └─ EOFTermination # 49-49
/// */
///
/// ```

pub struct NonStructural<TProd: IProduction> {
    production: Rc<TProd>,
    fill_range: bool,
    debugger: OnceCell<Log<&'static str>>,
}

#[derive(Clone)]
/// A utility to memorize and use the parsed result at particular positions of code (Packrat parsing technique.)
///
/// This wrapper utility will first look for memorize parsed result for the associated production at the particular pointer location.
/// If the parsed result is not available at the particular location it will then obtain the parsed result for the associated production and also save it to memory.
pub struct Cacheable<TProd: IProduction> {
    cache_key: CacheKey,
    production: Rc<TProd>,
    debugger: OnceCell<Log<&'static str>>,
}

/// A builder utility trait implemented for all generic [IProduction] structure.
pub trait ProductionBuilder: IProduction {
    fn into_list(self) -> List<Self>
    where
        Self: Sized;
    fn into_node(self, node_value: Option<Self::Node>) -> Node<Self>
    where
        Self: Sized;

    fn into_lookahead(self, node_value: Option<Self::Node>) -> Lookahead<Self>
    where
        Self: Sized;

    fn into_separated_list<TS: IProduction<Node = Self::Node, Token = Self::Token>>(
        self,
        sep: &Rc<TS>,
        inclusive: bool,
    ) -> SeparatedList<Self, TS>
    where
        Self: Sized;
    fn into_suffixes<TP: IProduction<Node = Self::Node>>(
        self,
        id: &'static str,
        standalone: bool,
    ) -> Suffixes<Self>
    where
        Self: Sized;
    fn into_nullable(self) -> Nullable<Self>
    where
        Self: Sized;
    fn validate_with<TF: Fn(&Vec<ASTNode<Self::Node>>, &[u8]) -> Result<(), ProductionError>>(
        self,
        validation_fn: TF,
    ) -> Validator<Self, TF>
    where
        Self: Sized;
}

trait ProductionLogger {
    fn get_debugger(&self) -> Option<&Log<&'static str>>;

    fn log_entry(&self) {
        #[cfg(debug_assertions)]
        if let Some(log_label) = self.get_debugger() {
            if log_label.order() >= Log::Verbose(()).order() {
                println!("Entering '{}'", log_label)
            }
        }
    }
    fn log_filtered_result<TN: NodeImpl, TL: TokenImpl>(
        &self,
        _code: &Code,
        _index: FltrPtr,
        _stream: &TokenStream<TL>,
        _result: &ParsedResult<FltrPtr, TN>,
    ) {
        #[cfg(debug_assertions)]
        match _result {
            Ok(data) => {
                self.log_success(_code, _stream[_index].start, _stream[data.consumed_index].end)
            }
            Err(err) => self.log_error(_code, _stream[_index].start, err),
        }
    }
    fn log_lex_result<TN: NodeImpl, TL: TokenImpl>(
        &self,
        _code: &Code,
        _index: TokenPtr,
        _stream: &TokenStream<TL>,
        _result: &ParsedResult<TokenPtr, TN>,
    ) {
        #[cfg(debug_assertions)]
        match _result {
            Ok(data) => {
                self.log_success(_code, _stream[_index].start, _stream[data.consumed_index].end)
            }
            Err(err) => self.log_error(_code, _stream[_index].start, err),
        }
    }

    fn log_result<TN: NodeImpl>(
        &self,
        _code: &Code,
        _index: usize,
        _result: &ParsedResult<usize, TN>,
    ) {
        #[cfg(debug_assertions)]
        match _result {
            Ok(data) => self.log_success(_code, _index, data.consumed_index),
            Err(err) => self.log_error(_code, _index, err),
        }
    }
    fn log_success(&self, _code: &Code, _start: usize, _end: usize) {
        #[cfg(debug_assertions)]
        if let Some(log_label) = self.get_debugger() {
            if log_label.order() >= Log::Success(()).order() {
                println!(
                    "Parsing Success for '{}': from {} to {}.",
                    log_label,
                    _code.obtain_position(_start),
                    _code.obtain_position(_end),
                )
            }
        }
    }

    fn log_error(&self, _code: &Code, _index: usize, _err: &ProductionError) {
        #[cfg(debug_assertions)]
        if let Some(log_label) = self.get_debugger() {
            if log_label.order() >= Log::Result(()).order() {
                match _err {
                    ProductionError::Unparsed => {
                        println!(
                            "Unparsed production '{}': at {}.",
                            log_label,
                            _code.obtain_position(_index),
                        )
                    }
                    ProductionError::Validation(pointer, message) => {
                        println!(
                            "Validation error '{}': at {}. {}",
                            log_label,
                            _code.obtain_position(*pointer),
                            message
                        )
                    }
                }
            }
        }
    }
}
