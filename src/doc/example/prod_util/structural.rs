use crate::{
    lexeme::{LexemeBuilder, Pattern, Punctuations},
    production::{
        Concat, EOFProd, List, Lookahead, NonStructural, ProductionBuilder, TokenField,
        TokenFieldSet, Union,
    },
    DefaultParser, Log, NodeImpl, TokenImpl, Tokenizer,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Token {
    ID,
    String,
    Space,
    Colon,
    OpenAngle,
    CloseAngle,
    OpenAngleSlash,
    ForwardSlash,
    LineBreak,
    Semicolon,
    KeywordVar,
    KeywordConst,
    KeywordLet,
    KeywordNumber,
    KeywordString,
    KeywordObject,
    KeywordBoolean,
    EOF,
    Assign,
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

fn tokenizer() -> Tokenizer<Token> {
    let identifier = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();

    let string_literal = Pattern::new(
        Token::String,
        r#"^"([^"\\\r\n]|(\\[^\S\r\n]*[\r\n][^\S\r\n]*)|\\.)*""#,
    )
    .unwrap();
    let space = Pattern::new(Token::Space, r"^\s+").unwrap();

    let expression_punctuations = Punctuations::new(vec![
        ("<", Token::OpenAngle),
        (">", Token::CloseAngle),
        ("</", Token::OpenAngleSlash),
        ("/", Token::ForwardSlash),
        ("=", Token::Assign),
    ])
    .unwrap();
    Tokenizer::new(vec![
        Rc::new(identifier),
        Rc::new(space),
        Rc::new(string_literal),
        Rc::new(expression_punctuations),
    ])
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NodeValue {
    ID,
    Null,
    String,
    Attribute,
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
    let string_value = Rc::new(TokenField::new(Token::String, Some(NodeValue::String)));
    let hidden_assign = Rc::new(TokenField::new(Token::Assign, None));
    let hidden_open_angle = Rc::new(TokenField::new(Token::OpenAngle, None));
    let hidden_close_angle = Rc::new(TokenField::new(Token::OpenAngle, None));
    let hidden_angle_slash = Rc::new(TokenField::new(Token::OpenAngleSlash, None));

    let attribute = Rc::new(
        Concat::new("attribute", vec![id.clone(), hidden_assign, string_value])
            .into_node(NodeValue::Attribute),
    );

    let attribute_list = Rc::new(List::new(&attribute));

    let children=Concat::init(identifier);


    let element = Concat::new(
        "xml_element",
        vec![
            hidden_open_angle,
            id.clone(),
            attribute_list,
            hidden_close_angle,
        ],
    );

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

    let hidden_null_white_space = Rc::new(TokenField::new(Token::Space, None).into_null_hidden());

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

    statement_termination
        .set_log(Log::Verbose("statement-termination"))
        .unwrap();
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
        .into_node(NodeValue::VarAssignment),
    );

    let list_var_declaration = Rc::new(List::new(&var_declaration));

    let root =
        Rc::new(Concat::new("main", vec![list_var_declaration, eof]).into_node(NodeValue::Root));

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
