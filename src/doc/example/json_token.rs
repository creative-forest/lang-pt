use crate::util::Code;
use crate::{
    lexeme::{Pattern, Punctuations},
    TokenImpl, Tokenizer,
};
use crate::{ITokenization, Lex};
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
/// JSON token for the parsed document
enum JSONToken {
    EOF,
    String,
    Space,
    Colon,
    Comma,
    Number,
    Constant,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
}

impl TokenImpl for JSONToken {
    fn eof() -> Self {
        JSONToken::EOF
    }
    fn is_structural(&self) -> bool {
        match self {
            JSONToken::Space => false,
            _ => true,
        }
    }
}

#[test]
fn json_tokenizer() {
    let punctuations = Rc::new(
        Punctuations::new(vec![
            ("{", JSONToken::OpenBrace),
            ("}", JSONToken::CloseBrace),
            ("[", JSONToken::OpenBracket),
            ("]", JSONToken::CloseBracket),
            (",", JSONToken::Comma),
            (":", JSONToken::Colon),
        ])
        .unwrap(),
    );

    let dq_string = Rc::new(
        Pattern::new(
            JSONToken::String,
            r#"^"([^"\\\r\n]|(\\[^\S\r\n]*[\r\n][^\S\r\n]*)|\\.)*""#, //["\\bfnrtv]
        )
        .unwrap(),
    );

    let lex_space = Rc::new(Pattern::new(JSONToken::Space, r"^\s+").unwrap());
    let number_literal = Rc::new(
        Pattern::new(JSONToken::Number, r"^([0-9]+)(\.[0-9]+)?([eE][+-]?[0-9]+)?").unwrap(),
    );
    let const_literal = Rc::new(Pattern::new(JSONToken::Constant, r"^(true|false|null)").unwrap());

    let tokenizer = Tokenizer::new(vec![
        lex_space,
        punctuations,
        dq_string,
        number_literal,
        const_literal,
    ]);

    let tokens1 = tokenizer
        .tokenize(&Code::from(r#"{"a":34,"b":null}"#))
        .unwrap();

    assert_eq!(
        tokens1,
        vec![
            Lex {
                token: JSONToken::OpenBrace,
                start: 0,
                end: 1
            },
            Lex {
                token: JSONToken::String,
                start: 1,
                end: 4
            },
            Lex {
                token: JSONToken::Colon,
                start: 4,
                end: 5
            },
            Lex {
                token: JSONToken::Number,
                start: 5,
                end: 7
            },
            Lex {
                token: JSONToken::Comma,
                start: 7,
                end: 8
            },
            Lex {
                token: JSONToken::String,
                start: 8,
                end: 11
            },
            Lex {
                token: JSONToken::Colon,
                start: 11,
                end: 12
            },
            Lex {
                token: JSONToken::Constant,
                start: 12,
                end: 16
            },
            Lex {
                token: JSONToken::CloseBrace,
                start: 16,
                end: 17
            },
            Lex {
                token: JSONToken::EOF,
                start: 17,
                end: 17
            }
        ]
    );
}
