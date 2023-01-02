use crate::{
    production::{Concat, ConstantField, EOFProd, Node, PunctuationsField, RegexField},
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NodeValue {
    ID,
    Add,
    Sub,
    Mul,
    Div,
    NULL,
    Expr,
    Root,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
pub fn concat_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
    let operators = Rc::new(
        PunctuationsField::new(vec![
            ("+", Some(NodeValue::Add)),
            ("-", Some(NodeValue::Sub)),
            ("*", Some(NodeValue::Mul)),
            ("/", Some(NodeValue::Div)),
        ])
        .unwrap(),
    );
    let open_paren = Rc::new(ConstantField::new("(", None));
    let close_paren = Rc::new(ConstantField::new(")", None));

    let expression = Rc::new(Concat::new(
        "Expression",
        vec![id.clone(), operators.clone(), id.clone()],
    ));

    let expression_node = Rc::new(Node::new(&expression, NodeValue::Expr));

    let parenthesis_expression = Rc::new(Concat::new(
        "Parenthesis_Expression",
        vec![
            open_paren.clone(),
            expression_node.clone(),
            close_paren.clone(),
        ],
    ));

    let root = Rc::new(Concat::new("main", vec![parenthesis_expression, eof]));

    let root_node = Rc::new(Node::new(&root, NodeValue::Root));

    let parser = LexerlessParser::new(root_node).unwrap();

    let tree_list = parser.parse(b"(ax+by)").unwrap();
    tree_list.last().unwrap().print().unwrap();
}
