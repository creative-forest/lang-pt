use crate::{
    lexeme::{Pattern, Punctuations},
    util::Code,
    ITokenization, Lex, TokenImpl, Tokenizer,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    ID,
    Space,
    Add,
    Subtract,
    PlusPlus,
    MinusMinus,
    EOF,
}

impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }

    fn is_structural(&self) -> bool {
        *self != Self::EOF
    }
}

#[test]
fn f() {
    let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();
    let identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let punctuations: Punctuations<Token> = Punctuations::new(vec![
        ("+", Token::Add),
        ("++", Token::PlusPlus),
        ("--", Token::MinusMinus),
        ("-", Token::Subtract),
    ])
    .unwrap();

    let tokenizer = Tokenizer::new(vec![
        Rc::new(punctuations),
        Rc::new(space),
        Rc::new(identifier),
    ]);
    let lex = tokenizer.tokenize(&Code::from("a+++b")).unwrap();
    assert_eq!(
        lex,
        vec![
            Lex {
                token: Token::ID,
                start: 0,
                end: 1
            },
            Lex {
                token: Token::PlusPlus,
                start: 1,
                end: 3
            },
            Lex {
                token: Token::Add,
                start: 3,
                end: 4
            },
            Lex {
                token: Token::ID,
                start: 4,
                end: 5
            },
            Lex {
                token: Token::EOF,
                start: 5,
                end: 5
            }
        ]
    );
    let lex = tokenizer.tokenize(&Code::from("a+ ++b")).unwrap();
    assert_eq!(
        lex,
        vec![
            Lex {
                token: Token::ID,
                start: 0,
                end: 1
            },
            Lex {
                token: Token::Add,
                start: 1,
                end: 2
            },
            Lex {
                token: Token::Space,
                start: 2,
                end: 3
            },
            Lex {
                token: Token::PlusPlus,
                start: 3,
                end: 5
            },
            Lex {
                token: Token::ID,
                start: 5,
                end: 6
            },
            Lex {
                token: Token::EOF,
                start: 6,
                end: 6
            }
        ]
    );
}
