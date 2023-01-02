use crate::lexeme::Mapper;
use crate::lexeme::Pattern;
use crate::lexeme::Punctuations;
use crate::Code;
use crate::ITokenization;
use crate::Lex;
use crate::TokenImpl;
use crate::Tokenizer;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    ID,
    Number,
    Add,
    Sub,
    Mul,
    Div,
    LT,
    LTE,
    GT,
    GTE,
    EQ,
    Space,
    Semicolon,
    EOF,
    Assign,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    If,
    Else,
    While,
    For,
    True,
    False,
    Null,
    Undefined,
}

impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }

    fn is_structural(&self) -> bool {
        match self {
            Token::Space => false,
            _ => true,
        }
    }
}

#[test]
fn tokenizer() {
    let identifier: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let mapping_identifier = Mapper::new(
        identifier,
        vec![
            ("if", Token::If),
            ("else", Token::Else),
            ("while", Token::While),
            ("for", Token::For),
            ("true", Token::True),
            ("false", Token::False),
            ("null", Token::Null),
            ("undefined", Token::Undefined),
        ],
    )
    .unwrap();

    let number_literal =
        Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap();
    let non_break_space = Pattern::new(Token::Space, r"^[^\S\r\n]+").unwrap();

    let expression_punctuations = Punctuations::new(vec![
        ("+", Token::Add),
        ("-", Token::Sub),
        ("*", Token::Mul),
        ("/", Token::Div),
        ("<", Token::LT),
        ("<=", Token::LTE),
        (">", Token::GT),
        (">=", Token::GTE),
        ("==", Token::EQ),
        ("=", Token::Assign),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("(", Token::OpenParen),
        (")", Token::CloseParen),
        ("[", Token::OpenBracket),
        ("]", Token::CloseBracket),
        (";", Token::Semicolon),
    ])
    .unwrap();

    let tokenizer = Tokenizer::new(vec![
        Rc::new(non_break_space),
        Rc::new(mapping_identifier),
        Rc::new(number_literal),
        Rc::new(expression_punctuations),
    ]);

    let tokens2 = tokenizer.tokenize(&Code::from("if(true){}")).unwrap();

    debug_assert_eq!(
        tokens2,
        vec![
            Lex {
                token: Token::If,
                start: 0,
                end: 2
            },
            Lex {
                token: Token::OpenParen,
                start: 2,
                end: 3
            },
            Lex {
                token: Token::True,
                start: 3,
                end: 7
            },
            Lex {
                token: Token::CloseParen,
                start: 7,
                end: 8
            },
            Lex {
                token: Token::OpenBrace,
                start: 8,
                end: 9
            },
            Lex {
                token: Token::CloseBrace,
                start: 9,
                end: 10
            },
            Lex {
                token: Token::EOF,
                start: 10,
                end: 10
            }
        ]
    );
}
