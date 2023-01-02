use std::rc::Rc;

use crate::{
    lexeme::{Action, Mapper, Pattern, Punctuations, StateMixin},
    production::{Concat, EOFProd, Node, SeparatedList, TokenField, TokenFieldSet, Union},
    CombinedTokenizer, DefaultParser, NodeImpl, TokenImpl,
};

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
    Ternary,
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
    While,
    For,
    True,
    False,
    Null,
    Colon,
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

fn tokenizer() -> CombinedTokenizer<Token> {
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
        ("?", Token::Ternary),
        ("{", Token::OpenBrace),
        ("}", Token::CloseBrace),
        ("(", Token::OpenParen),
        (")", Token::CloseParen),
        ("[", Token::OpenBracket),
        ("]", Token::CloseBracket),
        (";", Token::Semicolon),
        (":", Token::Colon),
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
            Rc::new(mapped_id),
            Rc::new(number_literal),
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
}

#[derive(Debug, Clone, Copy)]
enum NodeValue {
    NULL,
    ID,
    Number,
    Add,
    Sub,
    Mul,
    Div,
    Product,
    Sum,
    Expr,
    Root,
}
impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}

#[test]
fn flatten() {
    let identifier = Rc::new(TokenField::new(Token::ID, Some(NodeValue::ID)));
    let number = Rc::new(TokenField::new(Token::Number, Some(NodeValue::Number)));
    let end_of_file = Rc::new(EOFProd::new(None));

    let add_ops = Rc::new(TokenFieldSet::new(vec![
        (Token::Add, Some(NodeValue::Add)),
        (Token::Sub, Some(NodeValue::Sub)),
    ]));
    let mul_ops = Rc::new(TokenFieldSet::new(vec![
        (Token::Mul, Some(NodeValue::Mul)),
        (Token::Div, Some(NodeValue::Div)),
    ]));
    //We are going to implement following grammar for parsing an javascript expression.
    /*
        Value   ← [0-9]+ / '(' Expr ')'
        Product ← Value (('*' / '/') Value)*
        Sum     ← Product (('+' / '-') Product)*
        Expr    ← Sum
    */
    // The expression in the parenthesis is required before defining expression.
    // Let's initialize an parenthesis expression. We will set productions after defining expression.

    let paren_expr = Rc::new(Concat::init("paren_expr"));

    let value = Rc::new(Union::new(
        "value",
        vec![number, identifier, paren_expr.clone()],
    ));

    let product = Rc::new(SeparatedList::new(&value, &mul_ops, true)); // The separated should be inclusive i.e. operators should not be at the end of production.

    let product_node = Rc::new(Node::new(&product, NodeValue::Product));

    let sum = Rc::new(SeparatedList::new(&product_node, &add_ops, false));

    let sum_node = Rc::new(Node::new(&sum, NodeValue::Sum));

    let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));

    let expression = Rc::new(Concat::new("expression", vec![sum_node.clone(), semicolon]));

    let expr_node = Rc::new(Node::new(&expression, NodeValue::Expr));

    let root = Rc::new(Concat::new("root", vec![expr_node.clone(), end_of_file]));
    let root_node = Rc::new(Node::new(&root, NodeValue::Root));
    // Setting thr production for parenthesis_expr.

    let open_paren = Rc::new(TokenField::new(Token::OpenParen, None));
    let close_paren = Rc::new(TokenField::new(Token::CloseParen, None));
    paren_expr
        .set_symbols(vec![open_paren, expression, close_paren])
        .unwrap();

    let parser = DefaultParser::new(Rc::new(tokenizer()), root_node).unwrap();
    let parsed_addition_tree = parser.parse(b"a+b-10;").unwrap();
    assert_eq!(parsed_addition_tree.len(), 1);
    parsed_addition_tree[0].print().unwrap();

    /*
    Root # 0-7
    └─ Expr # 0-7
       └─ Sum # 0-6
          ├─ Product # 0-1
          │  └─ ID # 0-1
          ├─ Add # 1-2
          ├─ Product # 2-3
          │  └─ ID # 2-3
          ├─ Sub # 3-4
          └─ Product # 4-6
             └─ Number # 4-6*/

    let parsed_tree = parser.parse(b"a+b*c;").unwrap();
    assert_eq!(parsed_tree.len(), 1);
    parsed_tree[0].print().unwrap();

    /*
    Root # 0-6
    └─ Expr # 0-6
       └─ Sum # 0-5
          ├─ Product # 0-1
          │  └─ ID # 0-1
          ├─ Add # 1-2
          └─ Product # 2-5
             ├─ ID # 2-3
             ├─ Mul # 3-4
             └─ ID # 4-5
        */
}
