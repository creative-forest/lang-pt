# lang-pt

A language parser tool to generate recursive descent top down parser.

# Overview

Parsers written for the languages like Javascript are often custom handwritten due to the complexity of the languages. However, writing custom parser code often increases development and maintenance costs for the parser. With an intention to reduce development efforts, the library has been created for building a parser for a high-level language (HLL). The goal for this library is to develop a flexible library to support a wide range of grammar keeping a fair performance in comparison to a custom-written parser.

# Usage

We will walk through an example implementation of Javascript expressions to describe the steps to generate the parser.

## Tokenization

#### Token Types

First, we will be creating a token type that will represent value of the each element of the token stream data. The token type should implement TokenImpl trait so that Tokenizer can access the required default End of File(EOF) token. The default EOF token will be added at the end of the token stream. We will define enum types containing various parts of javascript expression and implement TokenImpl trait.

```rust
use lang_pt::TokenImpl;
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
    Semicolon,
    LineBreak,
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
    fn eof() -> Self { Self::EOF }
    fn is_structural(&self) -> bool { todo!() }
}

```

#### Building Tokenizer

Tokenization is a pre-processing process where the input text document is split into a sequence of tokens.
The token stream will then feed into the parser to parse the input document into a meaningful Abstract Syntax Tree (AST). This library can be used to create a LexerlessParser program which discarded the need to separately assign a Tokenizer. However, a tokenizer often makes a parser perform faster than parsing without a lexer.
In this example, we will be creating a DefaultParser program that gets tokenized input from the tokenizer.

Before building a complete tokenizer for javascript expression, we will first implement a tokenizer for simple arithmetic expressions consisting of number, identifier, and arithmetic operators. A tokenizer consists of lexeme utilities which are responsible to create tokens at incremental position of the input.
Check out the [lexeme](crate::lexeme) API documentation to get overview of the available lexeme utilities and their functionalities.

```rust
use lang_pt::lexeme::{Pattern, Punctuations};
use lang_pt::util::Code;
use lang_pt::Lex;
use lang_pt::{ITokenization, Tokenizer};
use std::rc::Rc;
let identifier: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
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
])
.unwrap();
let tokenizer = Tokenizer::new(vec![
    Rc::new(non_break_space),
    Rc::new(identifier),
    Rc::new(number_literal),
    Rc::new(expression_punctuations),
    Rc::new(line_break),
]);
let tokens1 = tokenizer.tokenize(&Code::from("a+b+c=d")).unwrap();
debug_assert_eq!(
    tokens1,
    vec![
        Lex { token: Token::ID, start: 0, end: 1 },
        Lex { token: Token::Add, start: 1, end: 2 },
        Lex { token: Token::ID, start: 2, end: 3 },
        Lex { token: Token::Add, start: 3, end: 4 },
        Lex { token: Token::ID, start: 4, end: 5 },
        Lex { token: Token::Assign, start: 5, end: 6 },
        Lex { token: Token::ID, start: 6, end: 7 },
        Lex { token: Token::EOF, start: 7, end: 7},
    ]
);
 let tokens2 = tokenizer.tokenize(&Code::from("if(true){}")).unwrap();
 debug_assert_eq!(
    tokens2,
    vec![
        Lex { token: Token::ID, start: 0, end: 2 },
        Lex { token: Token::OpenParen, start: 2, end: 3 },
        Lex { token: Token::ID, start: 3, end: 7 },
        Lex { token: Token::CloseParen, start: 7, end: 8 },
        Lex { token: Token::OpenBrace, start: 8, end: 9 },
        Lex { token: Token::CloseBrace, start: 9, end: 10 },
        Lex { token: Token::EOF, start: 10, end: 10 }
    ]
);

```

##### Keywords & Constants

In the 2nd tokenization example ‘if’, and ‘true’ are keyword, and constant value respectively.
However, our current tokenizer will tokenize the keywords and values as ID.
Thus, the tokenizer should be updated so that keywords and values produce appropriate tokens.
Let’s add keywords and constant fields in the token types.

```rust
enum Token {
   ...
   If,
   Else,
   While,
   For,
   True,
   False,
   Null,
   Undefined,
}

```

Now we are going to map the tokenized ID into respective keywords. Therefore, we are going to wrap the identifier pattern with Mapper so that it maps keywords and values with associated tokens.

```rust
let identifier: Pattern<Token> = Pattern::new(Token::ID, r#"^[_$a-zA-Z][_$\w]*"#).unwrap();
let mapping_identifier = Mapper::new(
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
...
let tokenizer = Tokenizer::new(vec![
    Rc::new(non_break_space),
    Rc::new(mapping_identifier),
    Rc::new(number_literal),
    Rc::new(expression_punctuations),
]);

...

let tokens2 = tokenizer.tokenize(&Code::from("if(true){}")).unwrap();

debug_assert_eq!(
    tokens2,
    vec![
        Lex { token: Token::If, start: 0, end: 2 },
        Lex { token: Token::OpenParen, start: 2, end: 3 },
        Lex { token: Token::True, start: 3, end: 7 },
        Lex { token: Token::CloseParen, start: 7, end: 8 },
        Lex { token: Token::OpenBrace, start: 8, end: 9 },
        Lex { token: Token::CloseBrace, start: 9, end: 10 },
        Lex { token: Token::EOF, start: 10, end: 10 }
    ]
);
```

#### Regex literal

A regex literal for the Javascript language is defined by /pattern/[g][m][i]. Following lexeme utility can be implemented to parse Javascript regex literal.

```rust
let regex_literal = Pattern::new(
    Token::RegexLiteral,
    r"^/([^\\/\r\n\[]|\\.|\[[^]]+\])+/[gmi]*",
    )
    .unwrap();
```

However, /pattern/ could be a part of two division expression sequence. Therefore we will be looking into previous token to determine whether regex literal is valid in the current position.

```rust
let validated_regex_literal = Middleware::new(regex_literal, |_, lex_stream| {
    // Validate that latest position is not part of expression continuation.
    lex_stream.last().map_or(false, |d| match d.token {
        Token::ID | Token::Number | Token::CloseParen /* Parenthesis expr end.*/ => false,
        _ => true,
    })
});
// Adding Regex literal before punctuation.
let mut combined_tokenizer = CombinedTokenizer::new(
  MAIN,
  vec![
      ...
      Rc::new(validated_regex_literal),
      Rc::new(expression_punctuations_mixin),
  ],
);

```

### States

Multiple state-based tokenizer may be required to parse many language syntaxes like Javascript template literal. Thus, multiple states based CombinedTokenizer lexical analyzer may be formed to tokenize syntax like Javascript template literal. Each state provides a set of lexeme utilities that match relevant values or patterns for that particular state.

```rust
use lang_pt::lexeme::{Action, Mapper, Pattern, Punctuations, StateMixin};
use lang_pt::util::Code;
use lang_pt::Lex;
use lang_pt::TokenImpl;
use lang_pt::{CombinedTokenizer, ITokenization};
use std::rc::Rc;
const MAIN: u8 = 0;
const TEMPLATE: u8 = 1;
let expression_punctuations = Punctuations::new(vec![
   ...
    ("`", Token::TemplateTick),
])
.unwrap();

let expression_punctuations_mixin = StateMixin::new(
    expression_punctuations,
    vec![
        (
            Token::TemplateTick,
            Action::Append { state: TEMPLATE, discard: false, `},
        ),
        (
            Token::OpenBrace,
            Action::Append { state: MAIN, discard: false, `},
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
            Action::Append { state: MAIN, discard: false },
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

let token_stream = combined_tokenizer
    .tokenize(&Code::from("`Sum is ${a+b-c}`"))
    .unwrap();
debug_assert_eq!(
    token_stream,
    vec![
        Lex::new(Token::TemplateTick, 0, 1),
        Lex::new(Token::TemplateString, 1, 8),
        Lex::new(Token::TemplateExprStart, 8, 10),
        Lex::new(Token::ID, 10, 11),
        Lex::new(Token::Add, 11, 12),
        Lex::new(Token::ID, 12, 13),
        Lex::new(Token::Sub, 13, 14),
        Lex::new(Token::ID, 14, 15),
        Lex::new(Token::CloseBrace, 15, 16),
        Lex::new(Token::TemplateTick, 16, 17),
        Lex::new(Token::EOF, 17, 17),
    ]
);

```

## Parser

In this section, we will implement a Parser for Javascript expression. We will use the tokenized data from the tokenizer and parse it into AST according to the grammar of the language. Once we received the tokens from the tokenizer we like to filter non-grammatical tokens like ‘Space’ from the token list to simplify and speed up parsing. Therefore, we will update is_structural implementation to filter the non-structural token.

```rust
impl TokenImpl for Token {
    ...
    fn is_structural(&self) -> bool {
        match self {
            Token::Space | Token::LineBreak => false,
            _ => true,
        }
    }
}

```

We will also create Node values and implement NodeImpl trait to represent each node of the AST.

```rust
#[derive(Debug, Clone, Copy)]
enum NodeValue {
    NULL,
    ID,
    Number,
    Add,
    Sub,
    Mul,
    Div,
}
impl NodeImpl for NodeValue {
    fn null() -> Self {
        Self::NULL
    }
}
```

Now, we will be implementing the grammar for parsing Javascript expressions. Before writing our complete expression we will first implement a parser for simple arithmetic expressions.

```rust
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

let sum = Rc::new(SeparatedList::new(&product, &add_ops, false));

let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));

let expression = Rc::new(Concat::new("expression", vec![sum.clone(), semicolon]));

let root = Rc::new(Concat::new("root", vec![expression.clone(), end_of_file]));

// Setting the production for parenthesis_expr.

let open_paren = Rc::new(TokenField::new(Token::OpenParen, None));
let close_paren = Rc::new(TokenField::new(Token::CloseParen, None));
paren_expr
    .set_symbols(vec![open_paren, expression, close_paren])
    .unwrap();

let parser = DefaultParser::new(Rc::new(combined_tokenizer), root).unwrap();
let parsed_addition_tree = parser.parse(b"a+b-10;").unwrap();
println!("{:?}", parsed_addition_tree);
/*
[
    ASTNode { token: ID, start: 0, end: 1 },
    ASTNode { token: Add, start: 1, end: 2 },
    ASTNode { token: ID, start: 2, end: 3 },
    ASTNode { token: Sub, start: 3, end: 4 },
    ASTNode { token: Number, start: 4, end: 6 },
]
*/

let parsed_tree = parser.parse(b"a+b*c;").unwrap();
println!("{:?}", parsed_tree);

/*
[
    ASTNode { token: ID, start: 0, end: 1 },
    ASTNode { token: Add, start: 1, end: 2 },
    ASTNode { token: ID, start: 2, end: 3 },
    ASTNode { token: Mul, start: 3, end: 4 },
    ASTNode { token: ID, start: 4, end: 5 },
]
*/

```

By default, the production utilities Concat, List, or SeparatedList do not create any node in the parsed tree. Instead, they flatten the parsed tree and append it into a Vec. It is required to wrap a utility with Node to create a node in the AST. Let wrap multiplicative_term, addition, and expression with Node.

```rust
#[derive(Debug, Clone, Copy)]
enum NodeValue {
    ...
    Product,
    Sum,
    Expr,
    Root,
}
...
let product = Rc::new(SeparatedList::new(&value, &mul_ops, true)); // The separated should be inclusive i.e. operators should not be at the end of production.
let product_node = Rc::new(Node::new(&product, Some(NodeValue::Product)));
let sum = Rc::new(SeparatedList::new(&product_node, &add_ops, false));
let sum_node = Rc::new(Node::new(&sum, Some(NodeValue::Sum)));
let semicolon = Rc::new(TokenField::new(Token::Semicolon, None));
let expression = Rc::new(Concat::new("expression", vec![sum_node.clone(), semicolon]));
let expr_node = Rc::new(Node::new(&expression, Some(NodeValue::Expr)));
let root = Rc::new(Concat::new("root", vec![expr_node.clone(), end_of_file]));
let root_node = Rc::new(Node::new(&root, Some(NodeValue::Root)));
...
let parser = DefaultParser::new(Rc::new(combined_tokenizer), root_node).unwrap();
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
```

#### Higher order expression

Our current parser is not designed to parse higher-order expressions like truthy, instance-of expression, etc. Let us update our parser to parse higher-order expressions.

```rust
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
        (truthy_expr_part.clone(), Some(NodeValue::Truthy)),
        (instance_of_expr_part, Some(NodeValue::InstanceOfExpr)),
    ],
));

let expression = Rc::new(Concat::new(
    "expression",
    vec![expr_part.clone(), semicolon],
));
/*
Root # 0-17
└─ Expr # 0-17
   └─ Truthy # 0-16
      ├─ Comparative # 0-9
      │  ├─ Sum # 0-6
      │  │  ├─ Product # 0-1
      │  │  │  └─ ID # 0-1
      │  │  ├─ Add # 1-2
      │  │  ├─ Product # 2-3
      │  │  │  └─ ID # 2-3
      │  │  ├─ Sub # 3-4
      │  │  └─ Product # 4-6
      │  │     └─ Number # 4-6
      │  ├─ GT # 6-7
      │  └─ Sum # 7-9
      │     └─ Product # 7-9
      │        └─ Number # 7-9
      ├─ Comparative # 10-12
      │  └─ Sum # 10-12
      │     └─ Product # 10-12
      │        └─ Number # 10-12
      └─ Comparative # 13-16
         └─ Sum # 13-16
            ├─ Product # 13-14
            │  └─ ID # 13-14
            ├─ Add # 14-15
            └─ Product # 15-16
               └─ Number # 15-16
*/
```

#### Expression termination

Our current implementation require a semicolon(;) to terminate the javascript expression. However, a Javascript expression can also be termination by eof, close brace (}) or by line break character. We do not want to consume eof or close brace character because they are part of another production. Therefore, we will be implementing Lookahead utility to check if eof or '}' exist immediately after an expression.

Moreover, a new line character can also indicate a expression termination syntax for the Javascript language. However, is_structural implementation filtered out LineBreak tokens from token stream. Therefore, we will using NonStructural utility to enforce child production to consume unfiltered token stream. The complete production for expression termination is given below.

```rust
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
```

# Testing

A tokenizer and a parser built using this library consist of lexeme utilities and production utilities.
We can assign log levels for each lexeme and production utility so that every utility can be monitored separately.

### Logging tokenization

```rust
...
template_string.set_log(Log::Result("template-string")).unwrap();
...
combined_tokenizer.set_log(Log::Default("combined-tokenizer")).unwrap();
...
let token_stream = combined_tokenized.tokenize(b"`Sum is ${a+b-c}`").unwrap();
// Logs
/*
    Switching state 0 -> 1
    Entering template-string
    Lexeme Success for template-string : token: TemplateString from  { line: 1, column: 1 } to  { line: 1, column: 8 }.
    Entering template-string
    Lexeme error for template-string : at  { line: 1, column: 8 }
    Switching state 1 -> 0
    Switching state 0 -> 1
    Entering template-string
    Lexeme error for template-string : at  { line: 1, column: 16 }
*/
...
```

### Logging productions

```rust

truthy_expr_part.set_log(Log::Result("truthy-expr-part")).unwrap();

parser.parse(b"b instanceOf A;").unwrap();;
// Logs
/*
    Unparsed production 'truthy-expr-part': at  { line: 1, column: 2 }.
*/
parser.tokenize_n_parse(b"a+b-10>90?80:f+8;").unwrap();
// Log
/*
    Parsing Success for 'truthy-expr-part': from  { line: 1, column: 9 } to  { line: 1, column: 16 }.
*/

```

### Debugging parser

Moreover, each production can be tested separately by adding them for debugging as follows.

```rust

let mut parser = DefaultParser::new(Rc::new(combined_tokenizer), root_node).unwrap();

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

```

# License

Language Parser Tool (lang_pt) is provided under the MIT license. See [LICENSE](./LICENSE).
