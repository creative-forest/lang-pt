use crate::{lexeme::Pattern, util::Code, ITokenization, Lex, TokenImpl, Tokenizer};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Token {
    ID,
    Space,
    EOF,
}
impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }

    fn is_structural(&self) -> bool {
        self != &Self::Space
    }
}

#[test]
fn f() {
    let identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
    let space = Pattern::new(Token::Space, r#"^\s+"#).unwrap();

    let tokenizer = Tokenizer::new(vec![Rc::new(identifier), Rc::new(space)]);
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
            },
        ]
    );
}
