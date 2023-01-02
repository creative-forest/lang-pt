use crate::lexeme::{Action, Mapper, Middleware, Pattern, Punctuations, StateMixin};
use crate::Code;
use crate::Lex;
use crate::TokenImpl;
use crate::{CombinedTokenizer, ITokenization};
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
    RegexLiteral,
    LineBreak,
    While,
    For,
    True,
    False,
    Null,
    Undefined,
    TemplateTick,
    TemplateExprStart,
    TemplateString,
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
    const MAIN: u8 = 0;
    const TEMPLATE: u8 = 1;

    let identifier: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let mapped_id = Mapper::new(
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
    let line_break = Pattern::new(Token::LineBreak, r"^[\r\n]+").unwrap();

    let regex_literal = Pattern::new(
        Token::RegexLiteral,
        r"^/([^\\/\r\n\[]|\\.|\[[^]]+\])+/[gmi]*",
    )
    .unwrap();

    let validated_regex_literal = Middleware::new(regex_literal, |_, lex_stream| {
        lex_stream.last().map_or(false, |d| match d.token {
            Token::ID | Token::Number | Token::CloseParen => false,
            _ => true,
        })
    });

    let expression_punctuations = Punctuations::new(vec![
        ("+", Token::Add),
        ("-", Token::Sub),
        ("*", Token::Mul),
        ("<", Token::LT),
        ("<=", Token::LTE),
        (">", Token::GT),
        (">=", Token::GTE),
        ("==", Token::EQ),
        ("/", Token::Div),
        ("=", Token::Assign),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("(", Token::OpenParen),
        (")", Token::CloseParen),
        ("[", Token::OpenBracket),
        ("]", Token::CloseBracket),
        (";", Token::Semicolon),
        ("`", Token::TemplateTick),
    ])
    .unwrap();

    let expression_punctuations_mixin = StateMixin::new(
        expression_punctuations,
        vec![
            (
                Token::TemplateTick,
                Action::Append {
                    state: TEMPLATE,
                    discard: false,
                },
            ),
            (
                Token::OpenBrace,
                Action::Append {
                    state: MAIN,
                    discard: false,
                },
            ),
            (Token::CloseBrace, Action::Pop { discard: false }),
        ],
    );

    let template_string: Pattern<Token> = Pattern::new(
        Token::TemplateString,
        r"^([^`\\$]|\$[^{^`\\$]|\\[${`bfnrtv])+",
    )
    .unwrap();

    let template_punctuations = Punctuations::new(vec![
        ("`", Token::TemplateTick),
        ("${", Token::TemplateExprStart),
    ])
    .unwrap();

    let template_punctuation_mixin = StateMixin::new(
        template_punctuations,
        vec![
            (Token::TemplateTick, Action::Pop { discard: false }),
            (
                Token::TemplateExprStart,
                Action::Append {
                    state: MAIN,
                    discard: false,
                },
            ),
        ],
    );

    let mut combined_tokenizer = CombinedTokenizer::new(
        MAIN,
        vec![
            Rc::new(non_break_space),
            Rc::new(line_break),
            Rc::new(mapped_id),
            Rc::new(number_literal),
            Rc::new(validated_regex_literal),
            Rc::new(expression_punctuations_mixin),
        ],
    );

    combined_tokenizer.add_state(
        TEMPLATE,
        vec![
            Rc::new(template_string),
            Rc::new(template_punctuation_mixin),
        ],
    );

    let token_stream = combined_tokenizer
        .tokenize(&Code::from("`Sum is ${a+b-c}`"))
        .unwrap();
    debug_assert_eq!(
        token_stream,
        vec![
            Lex::new(Token::TemplateTick, 0, 1),
            Lex::new(Token::TemplateString, 1, 8),
            Lex::new(Token::TemplateExprStart, 8, 10),
            Lex::new(Token::ID, 10, 11),
            Lex::new(Token::Add, 11, 12),
            Lex::new(Token::ID, 12, 13),
            Lex::new(Token::Sub, 13, 14),
            Lex::new(Token::ID, 14, 15),
            Lex::new(Token::CloseBrace, 15, 16),
            Lex::new(Token::TemplateTick, 16, 17),
            Lex::new(Token::EOF, 17, 17),
        ]
    );
}
