use crate::lexeme::{Action, Mapper, Middleware, Pattern, Punctuations, StateMixin};
use crate::{Code, Log};
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

    template_string
        .set_log(Log::Result("template-string"))
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

    combined_tokenizer
        .set_log(Log::Default("combined-tokenizer"))
        .unwrap();

    let div_token_stream = combined_tokenizer.tokenize(&Code::from("2/xy/6")).unwrap();
    debug_assert_eq!(
        div_token_stream,
        vec![
            Lex {
                token: Token::Number,
                start: 0,
                end: 1
            },
            Lex {
                token: Token::Div,
                start: 1,
                end: 2
            },
            Lex {
                token: Token::ID,
                start: 2,
                end: 4
            },
            Lex {
                token: Token::Div,
                start: 4,
                end: 5
            },
            Lex {
                token: Token::Number,
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
    let regex_token_stream = combined_tokenizer.tokenize(&Code::from("a=/xy/i")).unwrap();
    debug_assert_eq!(
        regex_token_stream,
        vec![
            Lex {
                token: Token::ID,
                start: 0,
                end: 1
            },
            Lex {
                token: Token::Assign,
                start: 1,
                end: 2
            },
            Lex {
                token: Token::RegexLiteral,
                start: 2,
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
