use crate::{
    lexeme::{Pattern, ThunkMapper},
    Code,
    ITokenization, Lex, TokenImpl, Tokenizer,
};
use std::{io::BufRead, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    InlineComment,
    MultilineComment,
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
    let comment: Pattern<Token> = Pattern::new(Token::InlineComment, r#"^/\*(.|\n)*?\*/"#).unwrap();

    let comment_variants = ThunkMapper::new(comment, |data, code, _| {
        if code[data.start..data.end].lines().count() > 1 {
            Some(Token::MultilineComment)
        } else {
            None
        }
    });

    let tokenizer = Tokenizer::new(vec![Rc::new(comment_variants)]);
    let inline_comment = "/*This is inline comment*/";
    let inline_comment_tokens = tokenizer.tokenize(&Code::from(inline_comment)).unwrap();
    assert_eq!(
        inline_comment_tokens,
        vec![
            Lex {
                token: Token::InlineComment,
                start: 0,
                end: inline_comment.len()
            },
            Lex {
                token: Token::EOF, // EOF token
                start: inline_comment.len(),
                end: inline_comment.len()
            }
        ]
    );
    let multiline_comment = "/*This is first line\n.Another line comment*/";
    let multiline_comment_tokens = tokenizer.tokenize(&Code::from(multiline_comment)).unwrap();
    assert_eq!(
        multiline_comment_tokens,
        vec![
            Lex {
                token: Token::MultilineComment,
                start: 0,
                end: multiline_comment.len()
            },
            Lex {
                token: Token::EOF, // EOF token
                start: multiline_comment.len(),
                end: multiline_comment.len()
            }
        ]
    );
}
