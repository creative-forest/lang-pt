use crate::{
    lexeme::{Action, Pattern, Punctuations, ThunkStateMixin},
    Code,
    ITokenization, Lex, TokenImpl, Tokenizer,
};
use std::rc::Rc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    RegexLiteral,
    ID,
    Number,
    Add,
    Mul,
    Div,
    Assign,
    Subtract,
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
    let identifier = Rc::new(Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap());
    let number_literal =
        Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());

    let punctuations = Punctuations::new(vec![
        ("+", Token::Add),
        ("*", Token::Mul),
        ("/", Token::Div),
        ("=", Token::Assign),
        ("-", Token::Subtract),
    ])
    .unwrap();

    let punctuation_mixin = Rc::new(ThunkStateMixin::new(
        punctuations,
        |lex_data, _code, stream| {
            if lex_data.token == Token::Div {
                let is_expr_continuation =
                    stream
                        .last()
                        .map_or(false, |pre_data| match pre_data.token {
                            Token::ID | Token::Number => true,
                            _ => false,
                        });
                Action::None {
                    discard: !is_expr_continuation,
                } // If the symbol '/' immediately after id or number it is a div element.
                  // Otherwise discard the lexeme if it is part of regex expression
            } else {
                Action::None { discard: false }
            }
        },
    ));

    let regex_literal =
        Rc::new(Pattern::new(Token::RegexLiteral, r"^/([^\\/\r\n\[]|\\.|\[[^]]+\])+/").unwrap());

    let tokenizer = Tokenizer::new(vec![
        identifier,
        number_literal,
        punctuation_mixin,
        regex_literal, // Should appear after punctuation so that it will be checked once div '/' is rejected.
    ]);

    let lex = tokenizer.tokenize(&Code::from("2/xy/6")).unwrap();
    assert_eq!(
        lex,
        [
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
            },
        ]
    );
    let regex_lex = tokenizer.tokenize(&Code::from("a=/xy/")).unwrap();
    assert_eq!(
        regex_lex,
        [
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
                end: 6
            },
            Lex {
                token: Token::EOF,
                start: 6,
                end: 6
            },
        ]
    );
}
