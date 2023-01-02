use crate::{
    production::{
        Concat, EOFProd, List, Nullable, ProductionBuilder, PunctuationsField, RegexField,
    },
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum Token {
    ID,
    Add,
    Sub,
    Mul,
    Div,
    NULL,
    UnaryList,
    Expr,
    Main,
}

impl NodeImpl for Token {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
pub fn list_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(Token::ID)).unwrap());
    let operators = Rc::new(
        PunctuationsField::new(vec![
            ("+", Some(Token::Add)),
            ("-", Some(Token::Sub)),
            ("*", Some(Token::Mul)),
            ("/", Some(Token::Div)),
        ])
        .unwrap(),
    );

    let unary_operators = Rc::new(
        PunctuationsField::new(vec![("+", Some(Token::Add)), ("-", Some(Token::Sub))]).unwrap(),
    );

    let unary_operators_list = Rc::new(List::new(&unary_operators).into_node(Token::UnaryList));

    let nullable_unary_operator_list = Rc::new(Nullable::new(&unary_operators_list));

    let expression = Rc::new(
        Concat::new(
            "Expression",
            vec![
                id.clone(),
                operators.clone(),
                nullable_unary_operator_list.clone(),
                id.clone(),
            ],
        )
        .into_node(Token::Expr),
    );
    let main = Rc::new(Concat::new("main", vec![expression, eof]).into_node(Token::Main));

    let parser = LexerlessParser::new(main).unwrap();

    let tree_list1 = parser.parse(b"ax+by").unwrap();
    tree_list1[0].print().unwrap();

    /*
    Main # 0-5
    └─ Expr # 0-5
       ├─ ID # 0-2
       ├─ Add # 2-3
       ├─ NULL # 3-3
       └─ ID # 3-5
    */

    let tree_list2 = parser.parse(b"ax*+-by").unwrap();
    tree_list2[0].print().unwrap();
    /*
    Main # 0-7
    └─ Expr # 0-7
       ├─ ID # 0-2
       ├─ Mul # 2-3
       ├─ UnaryList # 3-5
       │  ├─ Add # 3-4
       │  └─ Sub # 4-5
       └─ ID # 5-7
    */
}
