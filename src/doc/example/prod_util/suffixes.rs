use crate::production::{ConstantField, ProductionBuilder};
use crate::NodeImpl;
use crate::{
    production::{Concat, RegexField, SeparatedList, Suffixes, Union},
    LexerlessParser,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    ID,
    Number,
    ArrayAccess,
    FunctionCall,
    FuncArgs,
    NULL,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]

fn suffixes_test() {
    let id = Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::ID)).unwrap());
    let number = Rc::new(
        RegexField::new(
            r"^(?:0|[\d--0]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?",
            Some(NodeValue::Number),
        )
        .unwrap(),
    );

    let id_or_number = Rc::new(Union::new("ID_or_Number", vec![id.clone(), number]));

    let comma = Rc::new(ConstantField::new(",", None));
    let open_bracket = Rc::new(ConstantField::new("[", None));
    let close_bracket = Rc::new(ConstantField::new("]", None));
    let open_paren = Rc::new(ConstantField::new("(", None));
    let close_paren = Rc::new(ConstantField::new(")", None));

    let array_index = Rc::new(Concat::new(
        "ArrayIndex",
        vec![
            open_bracket.clone(),
            id_or_number.clone(),
            close_bracket.clone(),
        ],
    ));

    let function_arguments =
        Rc::new(SeparatedList::new(&id_or_number, &comma, false).into_node(NodeValue::FuncArgs));

    let function_call = Rc::new(Concat::new(
        "FunctionCall",
        vec![open_paren, function_arguments, close_paren],
    ));

    let array_index_or_func_call = Rc::new(Suffixes::new(
        "ArrayOrFuncCall",
        &id,
        false,
        vec![
            (array_index, NodeValue::ArrayAccess),
            (function_call, NodeValue::FunctionCall),
        ],
    ));

    let parser = LexerlessParser::new(array_index_or_func_call).unwrap();

    let array_tree = parser.parse(b"arr[b]").unwrap();
    array_tree[0].print().unwrap();
    /*
    ArrayAccess # 0-6
    ├─ ID # 0-3
    └─ ID # 4-5
    */

    let function_call_tree = parser.parse(b"func(arg1,arg2,arg3)").unwrap();
    function_call_tree[0].print().unwrap();
    /*
    FunctionCall # 0-20
    ├─ ID # 0-4
    └─ FuncArgs # 5-19
       ├─ ID # 5-9
       ├─ ID # 10-14
       └─ ID # 15-19
     */
}
