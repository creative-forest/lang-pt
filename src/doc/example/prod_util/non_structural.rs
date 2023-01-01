use crate::{
    lexeme::{LexemeBuilder, Pattern, Punctuations},
    production::{
        Concat, EOFProd, List, Lookahead, NonStructural, ProductionBuilder, TokenField,
        TokenFieldSet, Union,
    },
    DefaultParser, NodeImpl, TokenImpl, Tokenizer, util::Log,
};
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
    Colon,
    LineBreak,
    Semicolon,
    KeywordVar,
    KeywordConst,
    KeywordLet,
    KeywordIf,
    KeywordNumber,
    KeywordString,
    KeywordObject,
    KeywordBoolean,
    EOF,
    Assign,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
}

impl TokenImpl for Token {
    fn eof() -> Self {
        Self::EOF
    }
    fn is_structural(&self) -> bool {
        match self {
            Token::Space | Token::LineBreak => false,
            _ => true,
        }
    }
}

fn tokenizer() -> Tokenizer<Token> {
    let mapped_identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#)
        .unwrap()
        .mapping(vec![
            ("var", Token::KeywordVar),
            ("const", Token::KeywordConst),
            ("let", Token::KeywordLet),
            ("if", Token::KeywordIf),
            ("boolean", Token::KeywordBoolean),
            ("number", Token::KeywordNumber),
            ("object", Token::KeywordObject),
            ("string", Token::KeywordString),
        ])
        .unwrap();

    let number_literal =
        Pattern::new(Token::Number, r"^(0|[\d--0]\d*)(\.\d+)?([eE][+-]?\d+)?").unwrap();
    let non_break_space = Pattern::new(Token::Space, r"^[^\S\r\n]+").unwrap();
    let line_break = Pattern::new(Token::LineBreak, r"^[\r\n]+").unwrap();

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
        (":", Token::Colon),
    ])
    .unwrap();
    Tokenizer::new(vec![
        Rc::new(non_break_space),
        Rc::new(line_break),
        Rc::new(mapped_identifier),
        Rc::new(number_literal),
        Rc::new(expression_punctuations),
    ])
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NodeValue {
    ID,
    Null,
    KeywordVar,
    KeywordLet,
    KeywordConst,
    TypingNumber,
    TypingString,
    TypingBool,
    TypingObject,
    EOFTermination,
    NewLine,
    VarAssignment,
    Root,
}

impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::Null
    }
}

#[test]
fn non_structural_test() {
    let eof = Rc::new(EOFProd::new(None));
    let id = Rc::new(TokenField::new(Token::ID, Some(NodeValue::ID)));

    let declaration_type = Rc::new(TokenFieldSet::new(vec![
        (Token::KeywordVar, Some(NodeValue::KeywordVar)),
        (Token::KeywordConst, Some(NodeValue::KeywordConst)),
        (Token::KeywordLet, Some(NodeValue::KeywordLet)),
    ]));

    let typing_type_union = Rc::new(TokenFieldSet::new(vec![
        (Token::KeywordNumber, Some(NodeValue::TypingNumber)),
        (Token::KeywordBoolean, Some(NodeValue::TypingBool)),
        (Token::KeywordObject, Some(NodeValue::TypingObject)),
        (Token::KeywordString, Some(NodeValue::TypingString)),
    ]));

    let hidden_colon = Rc::new(TokenField::new(Token::Colon, None));
    let semi_colon = Rc::new(TokenField::new(Token::Semicolon, None));

    let typing_declaration = Rc::new(Concat::new(
        "typing_declaration",
        vec![hidden_colon.clone(), typing_type_union.clone()],
    ));

    let lookahead_eof = Rc::new(Lookahead::new(&eof, Some(NodeValue::EOFTermination)));

    // A new line is also expression terminal for language like Javascript.
    // However, the new line tokens are filtered for improving performance.
    // Therefore, a NonStructural utility force the parsing on unfiltered tokens.

    let hidden_null_white_space = Rc::new(
        TokenField::new(Token::Space, None)
            .into_nullable()
            .into_node(None),// Will hide all children tree from AST.
    );

    let line_break = Rc::new(TokenField::new(Token::LineBreak, Some(NodeValue::NewLine)));

    let line_break_seq = Rc::new(Concat::new(
        "line_break_seq",
        vec![hidden_null_white_space, line_break],
    ));

    let non_structural_line_break = Rc::new(NonStructural::new(&line_break_seq, false));

    let statement_termination = Rc::new(Union::new(
        "statement_termination",
        vec![semi_colon, lookahead_eof, non_structural_line_break],
    ));

    statement_termination.set_log(Log::Verbose("statement-termination")).unwrap();
    let var_declaration = Rc::new(
        Concat::new(
            "var_declaration",
            vec![
                declaration_type.clone(),
                id.clone(),
                typing_declaration.clone(),
                statement_termination.clone(),
            ],
        )
        .into_node(Some(NodeValue::VarAssignment)),
    );

 

    let list_var_declaration = Rc::new(List::new(&var_declaration));

    let root = Rc::new(
        Concat::new("main", vec![list_var_declaration, eof]).into_node(Some(NodeValue::Root)),
    );

    let parser = DefaultParser::new(Rc::new(tokenizer()), root).unwrap();

    let code = "
    let ax:number
    let ax:string
    ";

    let tree_node = parser.parse(code.as_bytes()).unwrap();
    tree_node[0].print().unwrap();
    /*
    Root # 9-49
    ├─ VarAssignment # 9-31
    │  ├─ KeywordLet # 9-12
    │  ├─ ID # 13-15
    │  ├─ TypingNumber # 16-22
    │  └─ NewLine # 22-23
    └─ VarAssignment # 31-49
       ├─ KeywordLet # 31-34
       ├─ ID # 35-37
       ├─ TypingString # 38-44
       └─ EOFTermination # 49-49
    */
}
