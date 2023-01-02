use crate::production::ProductionBuilder;
use crate::{
    production::{Concat, ConstantField, EOFProd, RegexField, SeparatedList, Union},
    LexerlessParser, NodeImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    ID,
    Number,
    NULL,
    Array,
    Main,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
fn separated_list_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
    let number = Rc::new(
        RegexField::new(
            r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?",
            Some(NodeValue::Number),
        )
        .unwrap(),
    );

    let id_or_number = Rc::new(Union::new("id_or_Number", vec![id, number]));
    let comma = Rc::new(ConstantField::new(",", None));
    let open_bracket = Rc::new(ConstantField::new("[", None));
    let close_bracket = Rc::new(ConstantField::new("]", None));

    let array_items = Rc::new(SeparatedList::new(&id_or_number, &comma, false));

    let array_literal = Rc::new(
        Concat::new(
            "ArrayLiteral",
            vec![open_bracket, array_items, close_bracket],
        )
        .into_node(NodeValue::Array),
    );

    let main = Rc::new(Concat::new("main", vec![array_literal, eof]).into_node(NodeValue::Main));

    let parser = LexerlessParser::new(main).unwrap();

    parser
        .parse(b"[a,b,]")
        .expect_err("Non-inclusive SeparatedList should fail to parse last comma(,)");

    let tree_list = parser.parse(b"[a,b,2,3,c,4]").unwrap();
    tree_list[0].print().unwrap();
    /*
    Main # 0-13
    └─ Array # 0-13
       ├─ ID # 1-2
       ├─ ID # 3-4
       ├─ Number # 5-6
       ├─ Number # 7-8
       ├─ ID # 9-10
       └─ Number # 11-12
    */
}
