use crate::{
    lexeme::{Pattern, Punctuations},
    Code,
    ITokenization, Lex, TokenImpl, Tokenizer,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Number,
    Add,
    Sub,
    Mul,
    Div,
    Space,
    EOF,
    Exponent,
    OpenParen,
    CloseParen,
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
pub fn main() {
    // Implementing a lexer to tokenize a simple arithmetic expression.

    let number_literal =
        Rc::new(Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap());

    let lex_space = Rc::new(Pattern::new(Token::Space, r"^\s+").unwrap());

    let lex_main_punctuations = Rc::new(
        Punctuations::new(vec![
            ("+", Token::Add),
            ("-", Token::Sub),
            ("*", Token::Mul),
            ("^", Token::Exponent),
            ("/", Token::Div),
            ("(", Token::OpenParen),
            (")", Token::CloseParen),
        ])
        .unwrap(),
    );
    let tokenizer: Tokenizer<Token> =
        Tokenizer::new(vec![number_literal, lex_main_punctuations, lex_space]);
    let tokens_stream = tokenizer.tokenize(&Code::from("9^3+10^3")).unwrap();
    assert_eq!(
        tokens_stream,
        vec![
            Lex::new(Token::Number, 0, 1),
            Lex::new(Token::Exponent, 1, 2),
            Lex::new(Token::Number, 2, 3),
            Lex::new(Token::Add, 3, 4),
            Lex::new(Token::Number, 4, 6),
            Lex::new(Token::Exponent, 6, 7),
            Lex::new(Token::Number, 7, 8),
            Lex::new(Token::EOF, 8, 8),
        ]
    );
}
