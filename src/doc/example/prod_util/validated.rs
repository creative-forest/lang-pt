use crate::production::ConstantField;
use crate::production::ProductionBuilder;
use crate::NodeImpl;
use crate::{
    production::{Concat, EOFProd, RegexField, Validator},
    LexerlessParser, ProductionError,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum NodeValue {
    NULL,
    TagName,
    Text,
    Root,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
fn validation_test() {
    let eof = Rc::new(EOFProd::new(None));
    let xml_tag =
        Rc::new(RegexField::new(r#"^[_$a-zA-Z][_$\w]*"#, Some(NodeValue::TagName)).unwrap());
    let xml_text = Rc::new(RegexField::new(r#"^([^><]|\\[><])*"#, Some(NodeValue::Text)).unwrap());

    let open_angle = Rc::new(ConstantField::new("<", None));
    let close_angle = Rc::new(ConstantField::new(">", None));

    let open_angle_slash = Rc::new(ConstantField::new("</", None));

    let xml_element = Rc::new(Concat::new(
        "xml_element",
        vec![
            open_angle.clone(),
            xml_tag.clone(),
            close_angle.clone(),
            xml_text.clone(),
            open_angle_slash.clone(),
            xml_tag.clone(),
            close_angle.clone(),
        ],
    ));

    let validated_xml_element = Rc::new(Validator::new(&xml_element, |children, code| {
        let start_tag = &code[children[0].start..children[0].end];
        let end_tag = &code[children[2].start..children[2].end];
        if start_tag != end_tag {
            return Err(ProductionError::Validation(children[0].start, unsafe {
                format!(
                    "Mismatch xml start tag {} and end tag {}",
                    std::str::from_utf8_unchecked(start_tag),
                    std::str::from_utf8_unchecked(end_tag)
                )
            }));
        }
        Ok(())
    }));
    let root_node =
        Rc::new(Concat::new("main", vec![validated_xml_element, eof]).into_node(NodeValue::Root));

    let parser = LexerlessParser::new(root_node).unwrap();

    parser
        .parse(b"<span>This is text.</div>")
        .expect_err("Should through a validation error");
    let tree_node = parser.parse(b"<span>This is text.</span>").unwrap();
    tree_node[0].print().unwrap();
    /*
    Root # 0-26
    ├─ TagName # 1-5
    ├─ Text # 6-19
    └─ TagName # 21-25
    */
}
