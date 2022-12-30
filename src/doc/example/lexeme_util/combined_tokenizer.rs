use crate::{
    lexeme::{Action, Pattern, Punctuations, StateMixin},
    util::Code,
    CombinedTokenizer, ITokenization, Lex, TokenImpl,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    EOF,
    Space,
    ID,
    Add,
    PlusPlus,
    OpenParen,
    CloseParen,
    Subtract,
    MinusMinus,
    Assign,
    OpenBrace,
    CloseBrace,
    TemplateTick,
    TemplateString,
    TemplateExprStart,
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
fn test_tokenizer() {
    const MAIN: u8 = 0;
    const TEMPLATE: u8 = 1;
    let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
    let lex_space = Rc::new(Pattern::new(Token::Space, r"^\s+").unwrap());

    let default_state_punctuation = Punctuations::new(vec![
        ("+", Token::Add),
        ("++", Token::PlusPlus),
        ("--", Token::MinusMinus),
        ("-", Token::Subtract),
        ("=", Token::Assign),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("(", Token::OpenParen),
        (")", Token::CloseParen),
        ("`", Token::TemplateTick),
    ])
    .unwrap();

    let punctuation_state_mixin = Rc::new(StateMixin::new(
        default_state_punctuation,
        vec![
            (Token::TemplateTick, Action::append(TEMPLATE, false)),
            (Token::OpenBrace, Action::append(MAIN, false)),
            (Token::CloseBrace, Action::remove(false)),
        ],
    ));

    let lex_template_string: Rc<Pattern<Token>> = Rc::new(
        Pattern::new(
            Token::TemplateString,
            r"^([^`\\$]|\$[^{^`\\$]|\\[${`bfnrtv])+",
        )
        .unwrap(),
    );

    let template_punctuations = Punctuations::new(vec![
        ("`", Token::TemplateTick),
        ("${", Token::TemplateExprStart),
    ])
    .unwrap();

    let template_punctuation_mixin = Rc::new(StateMixin::new(
        template_punctuations,
        vec![
            (Token::TemplateTick, Action::remove(false)),
            (Token::TemplateExprStart, Action::append(MAIN, false)),
        ],
    ));

    let mut combined_tokenizer =
        CombinedTokenizer::new(MAIN, vec![lex_space, identifier, punctuation_state_mixin]);

    combined_tokenizer.add_state(
        TEMPLATE,
        vec![template_punctuation_mixin, lex_template_string],
    );

    let token_stream = combined_tokenizer
        .tokenize(&Code::from("`${d=a+b+c}`"))
        .unwrap();
    // println!("{:?}", token_stream);
    debug_assert_eq!(
        token_stream,
        vec![
            Lex::new(Token::TemplateTick, 0, 1),
            Lex::new(Token::TemplateExprStart, 1, 3),
            Lex::new(Token::ID, 3, 4),
            Lex::new(Token::Assign, 4, 5),
            Lex::new(Token::ID, 5, 6),
            Lex::new(Token::Add, 6, 7),
            Lex::new(Token::ID, 7, 8),
            Lex::new(Token::Add, 8, 9),
            Lex::new(Token::ID, 9, 10),
            Lex::new(Token::CloseBrace, 10, 11),
            Lex::new(Token::TemplateTick, 11, 12),
            Lex::new(Token::EOF, 12, 12),
        ]
    );
    let token_stream = combined_tokenizer
        .tokenize(&Code::from("`Today is ${new Date()}`"))
        .unwrap();
    // println!("{:?}", token_stream);
    debug_assert_eq!(
        token_stream,
        vec![
            Lex::new(Token::TemplateTick, 0, 1),
            Lex::new(Token::TemplateString, 1, 10),
            Lex::new(Token::TemplateExprStart, 10, 12),
            Lex::new(Token::ID, 12, 15),
            Lex::new(Token::Space, 15, 16),
            Lex::new(Token::ID, 16, 20),
            Lex::new(Token::OpenParen, 20, 21),
            Lex::new(Token::CloseParen, 21, 22),
            Lex::new(Token::CloseBrace, 22, 23),
            Lex::new(Token::TemplateTick, 23, 24),
            Lex::new(Token::EOF, 24, 24),
        ]
    );
}
