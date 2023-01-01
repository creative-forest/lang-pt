use crate::production::ProductionBuilder;
use crate::{
    production::{
        Concat, ConstantField, EOFProd, Node, Nullable, RegexField, SeparatedList, Union,
    },
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum JSONNode {
    String,
    Number,
    Constant,
    Array,
    Object,
    Item,
    Main,
    NULL,
}

impl NodeImpl for JSONNode {
    fn null() -> Self {
        JSONNode::NULL
    }
}

pub fn json_lexerless_grammar() -> LexerlessParser<JSONNode> {
    let eof = Rc::new(EOFProd::new(None));

    let json_string = Rc::new(
        RegexField::new(
            r#"^"([^"\\\r\n]|(\\[^\S\r\n]*[\r\n][^\S\r\n]*)|\\.)*""#,
            Some(JSONNode::String),
        )
        .unwrap(),
    );

    // json_string.change_log_label(crate::parser::helpers::abstracts::ProductionLogLabel::Verbose);

    let nullable_hidden_space = Rc::new(RegexField::new(r"^\s*", None).unwrap());
    let json_const =
        Rc::new(RegexField::new(r"^(true|false|null)", Some(JSONNode::Constant)).unwrap());
    let json_number = Rc::new(
        RegexField::new(
            r"^([0-9]+)(\.[0-9]+)?([eE][+-]?[0-9]+)?",
            Some(JSONNode::Number),
        )
        .unwrap(),
    );

    // let json_primitive_values = Rc::new(Union::new(
    //     "JSON_PRIMITIVE_VALUES",
    //     vec![],
    // ));

    // let json_space = utility.insert_prod(TokenTerminal::new("JSON_SPACE", JSONNode::LSpace));

    let hidden_open_brace = Rc::new(ConstantField::new("{", None));

    let hidden_close_brace = Rc::new(ConstantField::new("}", None));

    let hidden_open_bracket = Rc::new(ConstantField::new("[", None));

    let hidden_close_bracket = Rc::new(ConstantField::new("]", None));

    let hidden_comma = Rc::new(ConstantField::new(",", None));

    let hidden_colon = Rc::new(ConstantField::new(":", None));

    let hidden_space_comma_space = Rc::new(Concat::new(
        "SPACE_COMMA",
        vec![
            nullable_hidden_space.clone(),
            hidden_comma,
            nullable_hidden_space.clone(),
        ],
    ));

    let hidden_space_colon_space = Rc::new(Concat::new(
        "SPACE_COLON",
        vec![
            nullable_hidden_space.clone(),
            hidden_colon,
            nullable_hidden_space.clone(),
        ],
    ));

    let json_object = Rc::new(Concat::init("JSON_OBJECT"));
    let json_value_union = Rc::new(Union::init("JSON_VALUE_UNION"));

    let json_object_item = Rc::new(Concat::new(
        "JSON_OBJECT_ITEM",
        vec![
            json_string.clone(),
            hidden_space_colon_space.clone(),
            json_value_union.clone(),
        ],
    ));

    let json_object_item_node = Rc::new(Node::new(&json_object_item, Some(JSONNode::Item)));

    let json_object_item_list = Rc::new(SeparatedList::new(
        &json_object_item_node,
        &hidden_space_comma_space,
        true,
    ));
    let nullable_json_object_item_list = Rc::new(Nullable::new(&json_object_item_list));
    let json_array_item_list = Rc::new(SeparatedList::new(
        &json_value_union,
        &hidden_space_comma_space,
        true,
    ));
    let nullable_json_array_item_list = Rc::new(Nullable::new(&json_array_item_list));
    let json_array = Rc::new(Concat::new(
        "JSON_ARRAY",
        vec![
            hidden_open_bracket.clone(),
            nullable_hidden_space.clone(),
            nullable_json_array_item_list.clone(),
            nullable_hidden_space.clone(),
            hidden_close_bracket.clone(),
        ],
    ));

    let json_array_node = Rc::new(Node::new(&json_array, Some(JSONNode::Array)));

    let json_object_node = Rc::new(Node::new(&json_object, Some(JSONNode::Object)));

    json_value_union
        .set_symbols(vec![
            json_object_node.clone(),
            json_string.clone(),
            json_array_node.clone(),
            json_number,
            json_const,
        ])
        .unwrap();

    json_object
        .set_symbols(vec![
            hidden_open_brace.clone(),
            nullable_hidden_space.clone(),
            nullable_json_object_item_list,
            nullable_hidden_space.clone(),
            hidden_close_brace.clone(),
        ])
        .unwrap();

    let main_node = Rc::new(
        Concat::new(
            "MAIN",
            vec![
                nullable_hidden_space.clone(),
                json_value_union,
                nullable_hidden_space.clone(),
                eof,
            ],
        )
        .into_node(Some(JSONNode::Main)),
    );

    let parser = LexerlessParser::new(main_node).unwrap();

    if cfg!(debug_assertions) {
        let mut mut_parser = parser;

        mut_parser.add_debug_production("object", &json_object_node);
        mut_parser.add_debug_production("array", &json_array_node);
        mut_parser
    } else {
        parser
    }
}
