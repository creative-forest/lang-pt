use crate::{
    production::{Concat, EOFProd, List, Node, ProductionBuilder, PunctuationsField, RegexField},
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    ID,
    Add,
    Sub,
    Mul,
    Div,
    UnaryList,
    Expr,
    Root,
    NULl,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULl
    }
}

#[test]
fn list_test() {
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
    let unary_operators = Rc::new(
        PunctuationsField::new(vec![
            ("+", Some(NodeValue::Add)),
            ("-", Some(NodeValue::Sub)),
        ])
        .unwrap(),
    );

    let unary_operators_list = Rc::new(List::new(&unary_operators));

    let unary_operator_list_node = Rc::new(Node::new(&unary_operators_list, NodeValue::UnaryList));

    let expression = Rc::new(
        Concat::new(
            "Expression",
            vec![
                id.clone(),
                operators.clone(),
                unary_operator_list_node.clone(),
                id.clone(),
            ],
        )
        .into_node(NodeValue::Expr),
    );
    let root = Rc::new(Concat::new("root", vec![expression, eof]).into_node(NodeValue::Root));

    let parser = LexerlessParser::new(root).unwrap();

    let tree_list1 = parser.parse(b"ax*+by").unwrap();
    tree_list1[0].print().unwrap();
    /*
    Root # 0-6
    └─ Expr # 0-6
       ├─ ID # 0-2
       ├─ Mul # 2-3
       ├─ UnaryList # 3-4
       │  └─ Add # 3-4
       └─ ID # 4-6
    */

    let tree_list2 = parser.parse(b"ax*+-by").unwrap();
    tree_list2[0].print().unwrap();
    /*
    Root # 0-7
    └─ Expr # 0-7
       ├─ ID # 0-2
       ├─ Mul # 2-3
       ├─ UnaryList # 3-5
       │  ├─ Add # 3-4
       │  └─ Sub # 4-5
       └─ ID # 5-7
    */
}
