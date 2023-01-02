use crate::{
    production::{Concat, EOFProd, Node, PunctuationsField, RegexField},
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    NULL,
    ID,
    Add,
    Sub,
    Mul,
    Div,
    Main,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
fn node_test() {
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

    let expression = Rc::new(Concat::new(
        "Expression",
        vec![id.clone(), operators.clone(), id.clone()],
    ));

    let main = Rc::new(Concat::new("Main", vec![expression.clone(), eof]));
    let main_node = Rc::new(Node::new(&main, NodeValue::Main));

    let parser = LexerlessParser::new(main_node).unwrap();

    let tree_node = parser.parse(b"ax+by").unwrap();
    tree_node[0].print().unwrap();
}
