use std::{
    fmt::{Debug, Display},
    rc::Rc,
    time::Instant,
};

use crate::{
    production::{Concat, Node, Nullable, TokenField},
    LexerlessParser, NodeImpl, TokenImpl,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum Token {
    A,
    B,
    C,
    Eof,
}
impl TokenImpl for Token {
    fn eof() -> Self {
        Token::Eof
    }

    fn is_structural(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
enum NodeValue {
    P,
    Q,
    R,
    NULL,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test]
fn circular_dependency_test() {
    let p1 = Rc::new(TokenField::new(Token::A, Some(NodeValue::P)));
    let p2 = Rc::new(TokenField::new(Token::B, Some(NodeValue::Q)));

    let p3 = Rc::new(Concat::<NodeValue, Token>::init("ID1"));

    let p4 = Rc::new(Concat::<NodeValue, Token>::new(
        "ID2",
        vec![p3.clone(), p2.clone(), p1.clone()],
    ));

    p3.set_symbols(vec![p4.clone(), p2.clone(), p3.clone()])
        .unwrap();

    let p5 = Rc::new(Node::new(&p4, Some(NodeValue::R)));
    let now = Instant::now();
    match LexerlessParser::new(p5) {
        Ok(_) => panic!("Validation should fail."),
        Err(err) => println!("{:?}", err),
    }
    println!("Time elapsed: {:?}", now.elapsed());
}

#[test]
fn circular_dependency_test2() {
    let p1 = Rc::new(TokenField::new(Token::A, Some(NodeValue::P)));
    let p2 = Rc::new(TokenField::new(Token::B, Some(NodeValue::Q)));
    let np1 = Rc::new(Nullable::new(&p1));
    let np2 = Rc::new(Nullable::new(&p2));

    let np3 = Rc::new(Concat::<NodeValue, Token>::new("N_ID1", vec![np1, np2]));
    let p3 = Rc::new(Concat::<NodeValue, Token>::init("ID1"));

    let p4 = Rc::new(Concat::<NodeValue, Token>::new(
        "ID2",
        vec![p3.clone(), p2.clone(), p1.clone()],
    ));

    p3.set_symbols(vec![np3.clone(), p3.clone()]).unwrap();

    let p5 = Rc::new(Node::new(&p4, Some(NodeValue::R)));
    let now = Instant::now();
    match LexerlessParser::new(p5) {
        Ok(_) => panic!("Validation should fail."),
        Err(err) => println!("{:?}", err),
    }
    println!("Time elapsed: {:?}", now.elapsed());
}
#[test]
fn print_grammar_test() {
    let p1 = Rc::new(TokenField::new(Token::A, Some(NodeValue::P)));
    let p2 = Rc::new(TokenField::new(Token::B, Some(NodeValue::Q)));
    let p3 = Rc::new(TokenField::new(Token::C, Some(NodeValue::R)));

    let p4 = Rc::new(Concat::<NodeValue, Token>::new(
        "PROD4",
        vec![p3.clone(), p1.clone()],
    ));
    let p5 = Rc::new(Concat::<NodeValue, Token>::init("MAIN"));
    p5.set_symbols(vec![p4.clone(), p2.clone(), p1.clone()])
        .unwrap();
    let now = Instant::now();
    LexerlessParser::new(p5).unwrap();

    // println!(
    //     "Grammar:[
    //         {}
    //         ]",
    //     gu.print_grammar().unwrap()
    // );
    println!("Time elapsed: {:.4?}", now.elapsed());
}
