use crate::{
    production::{Concat, ConstantField, EOFProd, Lookahead, ProductionBuilder, RegexField, Union},
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NodeValue {
    ID,
    Null,
    KeywordVar,
    KeywordLet,
    KeywordConst,
    TypingNumber,
    LineTermination,
    Root,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::Null
    }
}

#[test]
fn nullable_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
    let white_space = Rc::new(RegexField::new(r"^[^\S\r\n]+", None).unwrap());

    let keyword_var = Rc::new(ConstantField::new("var", Some(NodeValue::KeywordVar)));
    let keyword_let = Rc::new(ConstantField::new("let", Some(NodeValue::KeywordLet)));
    let keyword_const = Rc::new(ConstantField::new("const", Some(NodeValue::KeywordConst)));

    let declaration_type = Rc::new(Union::new(
        "var_type",
        vec![keyword_var, keyword_let, keyword_const],
    ));

    let typing_type_number = Rc::new(ConstantField::new("number", Some(NodeValue::TypingNumber)));
    let typing_type_string = Rc::new(ConstantField::new("string", Some(NodeValue::KeywordLet)));
    let typing_type_object = Rc::new(ConstantField::new("object", Some(NodeValue::KeywordConst)));
    let typing_type_boolean = Rc::new(ConstantField::new("boolean", Some(NodeValue::KeywordConst)));

    let typing_type_union = Rc::new(Union::new(
        "typings",
        vec![
            typing_type_number,
            typing_type_string,
            typing_type_object,
            typing_type_boolean,
        ],
    ));

    let hidden_colon = Rc::new(ConstantField::new(":", None));
    let semi_colon = Rc::new(ConstantField::new(";", None));

    let typing_declaration = Rc::new(Concat::new(
        "typing_declaration",
        vec![hidden_colon.clone(), typing_type_union.clone()],
    ));

    let lookahead_eof = Rc::new(Lookahead::new(&eof, Some(NodeValue::LineTermination)));

    let statement_termination = Rc::new(Union::new(
        "statement_termination",
        vec![semi_colon, lookahead_eof],
    ));

    let var_declaration = Rc::new(Concat::new(
        "var_declaration",
        vec![
            declaration_type.clone(),
            white_space.clone(),
            id.clone(),
            typing_declaration.clone(),
            statement_termination.clone(),
        ],
    ));

    let root =
        Rc::new(Concat::new("main", vec![var_declaration, eof]).into_node(Some(NodeValue::Root)));

    let parser = LexerlessParser::new(root).unwrap();

    let tree_node = parser.parse(b"let ax:number;").unwrap();
    tree_node[0].print().unwrap();
    let nullable_typing_node = parser.parse(b"let ax:string").unwrap();
    nullable_typing_node[0].print().unwrap();
}
