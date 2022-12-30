use crate::production::{EOFProd, Node};
use crate::NodeImpl;
use crate::{
    production::{Concat, ConstantField, RegexField, Union},
    LexerlessParser,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    ID,
    Add,
    Sub,
    Mul,
    Div,
    NULL,
    Root
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
fn union_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
    let add = Rc::new(ConstantField::new("+", Some(NodeValue::Add)));
    let sub = Rc::new(ConstantField::new("-", Some(NodeValue::Sub)));
    let mul = Rc::new(ConstantField::new("*", Some(NodeValue::Mul)));
    let div = Rc::new(ConstantField::new("/", Some(NodeValue::Div)));
    let addition = Rc::new(Concat::new(
        "addition",
        vec![id.clone(), add.clone(), id.clone()],
    ));
    let subtraction = Rc::new(Concat::new(
        "subtraction",
        vec![id.clone(), sub.clone(), id.clone()],
    ));
    let multiplication = Rc::new(Concat::new(
        "multiplication",
        vec![id.clone(), mul.clone(), id.clone()],
    ));
    let division = Rc::new(Concat::new(
        "division",
        vec![id.clone(), div.clone(), id.clone()],
    ));
    let expression = Rc::new(Union::new(
        "expression",
        vec![addition, subtraction, multiplication, division],
    ));

    let main = Rc::new(Concat::new("main", vec![expression, eof]));
    let main_node = Rc::new(Node::new(&main, Some(NodeValue::Root)));

    let parser = LexerlessParser::new(main_node).unwrap();
    let tree_list = parser.parse(b"ax+by").unwrap();
    tree_list.last().unwrap().print().unwrap();
}
