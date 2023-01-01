use crate::production::ProductionBuilder;
use crate::util::Log;
use crate::{
    lexeme::{Action, Mapper, Pattern, Punctuations, StateMixin},
    production::{
        Concat, EOFProd, Lookahead, Node, SeparatedList, Suffixes, TokenField, TokenFieldSet, Union,
    },
    CombinedTokenizer, DefaultParser, NodeImpl, TokenImpl,
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
    LineBreak,
    If,
    Else,
    Colon,
    While,
    For,
    True,
    False,
    Null,
    InstanceOf,
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
            Token::Space | Token::LineBreak => false,
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
            ("instanceOf", Token::InstanceOf),
        ],
    )
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
            Rc::new(line_break),
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
    GT,
    GTE,
    LT,
    LTE,
    EQ,
    Product,
    Sum,
    Comparative,
    Truthy,
    InstanceOfExpr,
    Expr,
    Root,
    ExprTermination,
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

    let product_node = Rc::new(Node::new(&product, Some(NodeValue::Product)));

    let sum = Rc::new(SeparatedList::new(&product_node, &add_ops, false));

    let sum_node = Rc::new(Node::new(&sum, Some(NodeValue::Sum)));

    // Extending summation expression to compare arithmetic values.
    let cmp_ops = Rc::new(TokenFieldSet::new(vec![
        (Token::GT, Some(NodeValue::GT)),
        (Token::GTE, Some(NodeValue::GTE)),
        (Token::LT, Some(NodeValue::LT)),
        (Token::LTE, Some(NodeValue::LTE)),
        (Token::EQ, Some(NodeValue::EQ)),
    ]));

    // Implementing comparison expression.
    let cmp_expr = Rc::new(SeparatedList::new(&sum_node, &cmp_ops, true));

    let cmp_expr_node = Rc::new(Node::new(&cmp_expr, Some(NodeValue::Comparative)));

    let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));

    let ternary_op = Rc::new(TokenField::new(Token::Ternary, None));
    let colon = Rc::new(TokenField::new(Token::Colon, None));

    // The production comparison expression(cmp_expr) could be an expression or beginning part of true-false, instanceOf or typeof expression.
    // We will be implementing rest of the higher order expressions as suffixes to the comparison expression.

    let truthy_expr_part = Rc::new(Concat::new(
        "truthy_expr_part",
        vec![
            ternary_op,
            cmp_expr_node.clone(),
            colon,
            cmp_expr_node.clone(),
        ],
    ));

    truthy_expr_part
        .set_log(Log::Result("truthy-expr-part"))
        .unwrap();

    let instance_of = Rc::new(TokenField::new(Token::InstanceOf, None));
    let instance_of_expr_part = Rc::new(Concat::new(
        "instance_of_expr_part",
        vec![instance_of, cmp_expr_node.clone()],
    ));

    // Suffixes will return left production match with first match from the suffixes productions.
    let expr_part = Rc::new(Suffixes::new(
        "expr_part",
        &cmp_expr_node,
        true,
        vec![
            (truthy_expr_part.clone(), NodeValue::Truthy),
            (instance_of_expr_part, NodeValue::InstanceOfExpr),
        ],
    ));

    let lookahead_eof = Rc::new(Lookahead::new(
        &end_of_file,
        Some(NodeValue::ExprTermination),
    ));

    let close_brace = Rc::new(TokenField::new(Token::CloseBrace, None));

    let lookahead_close_brace = Rc::new(Lookahead::new(
        &close_brace,
        Some(NodeValue::ExprTermination),
    ));

    let hidden_null_white_space = Rc::new(TokenField::new(Token::Space, None).into_nullable());

    let line_break = Rc::new(TokenField::new(Token::LineBreak, None));

    let line_break_seq = Rc::new(
        Concat::new("line_break_seq", vec![hidden_null_white_space, line_break])
            .into_node(Some(NodeValue::ExprTermination)),
    );

    let expression_termination = Rc::new(Union::new(
        "line_termination",
        vec![
            semicolon,
            lookahead_eof,
            lookahead_close_brace,
            line_break_seq,
        ],
    ));

    let expression = Rc::new(Concat::new(
        "expression",
        vec![expr_part.clone(), expression_termination],
    ));

    let expr_node = Rc::new(Node::new(&expression, Some(NodeValue::Expr)));

    let root = Rc::new(Concat::new("root", vec![expr_node.clone(), end_of_file]));
    let root_node = Rc::new(Node::new(&root, Some(NodeValue::Root)));
    // Setting thr production for parenthesis_expr.

    let open_paren = Rc::new(TokenField::new(Token::OpenParen, None));
    let close_paren = Rc::new(TokenField::new(Token::CloseParen, None));
    paren_expr
        .set_symbols(vec![open_paren, expression, close_paren])
        .unwrap();

    let mut parser = DefaultParser::new(Rc::new(tokenizer()), root_node).unwrap();

    parser.add_debug_production("mul-expr", &product_node);
    parser.add_debug_production("sum-expr", &sum_node);

    let product_tree = parser.debug_production_at("mul-expr", b"a+b*4", 2).unwrap();
    product_tree[0].print().unwrap();
    /*
    Product # 2-5
    ├─ ID # 2-3
    ├─ Mul # 3-4
    └─ Number # 4-5
    */

    let sum_tree = parser.debug_production_at("sum-expr", b"a+b*4", 0).unwrap();
    sum_tree[0].print().unwrap();
    /*
    Sum # 0-5
    ├─ Product # 0-1
    │  └─ ID # 0-1
    ├─ Add # 1-2
    └─ Product # 2-5
       ├─ ID # 2-3
       ├─ Mul # 3-4
       └─ Number # 4-5
    */
}
