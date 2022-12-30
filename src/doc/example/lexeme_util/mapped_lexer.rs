use crate::{
    lexeme::{Mapper, Pattern},
    util::Code,
    ITokenization, Lex, TokenImpl, Tokenizer,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    ID,
    IF,
    ELSE,
    FOR,
    Space,
    True,
    False,
    EOF,
}

impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }

    fn is_structural(&self) -> bool {
        todo!()
    }
}

#[test]
fn f() {
    let id_lexer: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();

    let mapped_lexer = Mapper::new(
        id_lexer,
        vec![
            ("if", Token::IF),
            ("else", Token::ELSE),
            ("for", Token::FOR),
            ("true", Token::True),
            ("false", Token::False),
        ],
    )
    .unwrap();

    let tokenizer = Tokenizer::new(vec![Rc::new(mapped_lexer), Rc::new(space)]);
    let lex_stream = tokenizer.tokenize(&Code::from("abc xy")).unwrap();

    assert_eq!(
        lex_stream,
        vec![
            Lex {
                token: Token::ID,
                start: 0,
                end: 3
            },
            Lex {
                token: Token::Space,
                start: 3,
                end: 4
            },
            Lex {
                token: Token::ID,
                start: 4,
                end: 6
            },
            Lex {
                token: Token::EOF,
                start: 6,
                end: 6
            }
        ]
    );
    let lex = tokenizer.tokenize(&Code::from("if true")).unwrap();
    assert_eq!(
        lex,
        vec![
            Lex {
                token: Token::IF,
                start: 0,
                end: 2
            },
            Lex {
                token: Token::Space,
                start: 2,
                end: 3
            },
            Lex {
                token: Token::True,
                start: 3,
                end: 7
            },
            Lex {
                token: Token::EOF,
                start: 7,
                end: 7
            }
        ]
    );
}
