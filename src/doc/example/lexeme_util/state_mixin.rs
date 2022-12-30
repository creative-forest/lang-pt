use crate::{
    lexeme::{Action, Pattern, Punctuations, StateMixin},
    util::Code,
    CombinedTokenizer, ITokenization, Lex, TokenImpl,
};
use std::rc::Rc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    ID,
    Number,
    Add,
    Assign,
    Subtract,
    EOF,
    TemplateTick,
    TemplateExprStart,
    TemplateString,
    OpenBrace,
    CloseBrace,
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
    const MAIN: u8 = 0;
    const TEMPLATE: u8 = 1;
    let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
    let number_literal =
        Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());

    let expression_punctuation = Punctuations::new(vec![
        ("+", Token::Add),
        ("-", Token::Subtract),
        ("=", Token::Assign),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("`", Token::TemplateTick),
    ])
    .unwrap();

    let expr_punctuation_mixin = Rc::new(StateMixin::new(
        expression_punctuation,
        vec![
            (Token::TemplateTick, Action::append(TEMPLATE, false)), // Encountering a TemplateTick (`) indicates beginning of template literal.
            // While tokenizing in the template literal expression we are going to augment stack to keep track of open and close brace.
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
    let template_punctuation_mixin = StateMixin::new(
        template_punctuations,
        vec![
            (Token::TemplateTick, Action::remove(false)), // Encountering TemplateTick (`) indicates end of template literal state.
            (Token::TemplateExprStart, Action::append(MAIN, false)),
        ],
    );

    let mut combined_tokenizer = CombinedTokenizer::new(
        MAIN,
        vec![identifier, number_literal, expr_punctuation_mixin],
    );
    combined_tokenizer.add_state(
        TEMPLATE,
        vec![Rc::new(template_punctuation_mixin), lex_template_string],
    );

    let token_stream = combined_tokenizer
        .tokenize(&Code::from("d=`Sum is ${a+b}`"))
        .unwrap();
    debug_assert_eq!(
        token_stream,
        vec![
            Lex::new(Token::ID, 0, 1),
            Lex::new(Token::Assign, 1, 2),
            Lex::new(Token::TemplateTick, 2, 3),
            Lex::new(Token::TemplateString, 3, 10),
            Lex::new(Token::TemplateExprStart, 10, 12,),
            Lex::new(Token::ID, 12, 13),
            Lex::new(Token::Add, 13, 14),
            Lex::new(Token::ID, 14, 15),
            Lex::new(Token::CloseBrace, 15, 16),
            Lex::new(Token::TemplateTick, 16, 17),
            Lex::new(Token::EOF, 17, 17),
        ]
    );
}
