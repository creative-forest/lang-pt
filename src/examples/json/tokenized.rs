use crate::production::ProductionBuilder;
use crate::{
    lexeme::{Pattern, Punctuations},
    production::{Concat, EOFProd, Node, SeparatedList, TokenField, TokenFieldSet, Union},
    DefaultParser, NodeImpl, TokenImpl, Tokenizer,
};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
/// JSON token for the parsed document
pub enum JSONToken {
    EOF,
    String,
    Space,
    Colon,
    Comma,
    Number,
    Constant,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum JSONNode {
    Key,
    String,
    Number,
    Constant,
    Array,
    Object,
    Item,
    Main,
    NULL,
}

impl TokenImpl for JSONToken {
    fn eof() -> Self {
        JSONToken::EOF
    }
    fn is_structural(&self) -> bool {
        match self {
            JSONToken::Space => false,
            _ => true,
        }
    }
}
impl NodeImpl for JSONNode {
    fn null() -> Self {
        JSONNode::NULL
    }
}

pub fn json_tokenizer() -> Tokenizer<JSONToken> {
    let punctuations = Rc::new(
        Punctuations::new(vec![
            ("{", JSONToken::OpenBrace),
            ("}", JSONToken::CloseBrace),
            ("[", JSONToken::OpenBracket),
            ("]", JSONToken::CloseBracket),
            (",", JSONToken::Comma),
            (":", JSONToken::Colon),
        ])
        .unwrap(),
    );

    let dq_string = Rc::new(
        Pattern::new(
            JSONToken::String,
            r#"^"([^"\\\r\n]|(\\[^\S\r\n]*[\r\n][^\S\r\n]*)|\\.)*""#, //["\\bfnrtv]
        )
        .unwrap(),
    );

    let lex_space = Rc::new(Pattern::new(JSONToken::Space, r"^\s+").unwrap());
    let number_literal = Rc::new(
        Pattern::new(JSONToken::Number, r"^([0-9]+)(\.[0-9]+)?([eE][+-]?[0-9]+)?").unwrap(),
    );
    let const_literal = Rc::new(Pattern::new(JSONToken::Constant, r"^(true|false|null)").unwrap());

    Tokenizer::new(vec![
        lex_space,
        punctuations,
        dq_string,
        number_literal,
        const_literal,
    ])
}

pub fn json_grammar() -> DefaultParser<JSONNode, JSONToken> {
    let eof = Rc::new(EOFProd::new(None));

    let json_key = Rc::new(TokenField::new(JSONToken::String, Some(JSONNode::Key)));

    let json_primitive_values = Rc::new(TokenFieldSet::new(vec![
        (JSONToken::String, Some(JSONNode::String)),
        (JSONToken::Constant, Some(JSONNode::Constant)),
        (JSONToken::Number, Some(JSONNode::Number)),
    ]));

    // let json_space = utility.insert_prod(TokenTerminal::new( JSONToken::LSpace));

    let hidden_open_brace = Rc::new(TokenField::new(JSONToken::OpenBrace, None));

    let hidden_close_brace = Rc::new(TokenField::new(JSONToken::CloseBrace, None));

    let hidden_open_bracket = Rc::new(TokenField::new(JSONToken::OpenBracket, None));

    let hidden_close_bracket = Rc::new(TokenField::new(JSONToken::CloseBracket, None));

    let hidden_comma = Rc::new(TokenField::new(JSONToken::Comma, None));

    let hidden_colon = Rc::new(TokenField::new(JSONToken::Colon, None));

    let json_object = Rc::new(Concat::init("json_object"));
    let json_value_union = Rc::new(Union::init("json_value_union"));

    let json_object_item = Rc::new(Concat::new(
        "json_object_item",
        vec![
            json_key.clone(),
            hidden_colon.clone(),
            json_value_union.clone(),
        ],
    ));

    let json_object_item_node = Rc::new(Node::new(&json_object_item, Some(JSONNode::Item)));

    let json_object_item_list =
        Rc::new(SeparatedList::new(&json_object_item_node, &hidden_comma, true).into_nullable());
    let json_array_item_list =
        Rc::new(SeparatedList::new(&json_value_union, &hidden_comma, true).into_nullable());

    let json_array_node = Rc::new(
        Concat::new(
            "json_array",
            vec![
                hidden_open_bracket.clone(),
                json_array_item_list.clone(),
                hidden_close_bracket.clone(),
            ],
        )
        .into_node(Some(JSONNode::Array)),
    );

    let json_object_node = Rc::new(Node::new(&json_object, Some(JSONNode::Object)));

    json_value_union
        .set_symbols(vec![
            json_primitive_values.clone(),
            json_object_node.clone(),
            json_array_node.clone(),
        ])
        .unwrap();

    json_object
        .set_symbols(vec![
            hidden_open_brace.clone(),
            json_object_item_list,
            hidden_close_brace.clone(),
        ])
        .unwrap();

    let main = Rc::new(Concat::new("root", vec![json_value_union, eof]));
    let main_node = Rc::new(Node::new(&main, Some(JSONNode::Main)));

    let lexer = json_tokenizer();

    let parser = DefaultParser::new(Rc::new(lexer), main_node).unwrap();

    if cfg!(debug_assertions) {
        let mut mut_parser = parser;

        mut_parser.add_debug_production("object", &json_object_node);
        mut_parser.add_debug_production("array", &json_array_node);
        mut_parser
    } else {
        parser
    }
}
