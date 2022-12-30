use crate::examples::json::{lexerless::json_lexerless_grammar, tokenized::json_grammar};
use serde_json::Value as SerdeValue;
use std::time::Instant;

#[test]
fn test_parse_time() {
    let parser = json_grammar();
    let lexerless_parser = json_lexerless_grammar();

    let code_part = std::fs::read("./src/examples/__tests__/benchmark/example.json").unwrap();
    let times = 1000;

    let serde_instant = Instant::now();

    for _ in 0..times {
        serde_json::from_slice::<SerdeValue>(&code_part).unwrap();
    }

    println!("Serde time:{:?}", serde_instant.elapsed());

    let toolkit_instance = Instant::now();

    for _ in 0..times {
        parser.tokenize_n_parse(&code_part).unwrap();
    }
    println!("Toolkit time:{:?}", toolkit_instance.elapsed());

    let lexerless_instant = Instant::now();
    for _ in 0..times {
        lexerless_parser.parse(&code_part).unwrap();
    }
    println!("Lexerless time:{:?}", lexerless_instant.elapsed());
}
#[test]
fn large_file_parse_time() {
    let parser = json_grammar();
    let lexerless_parser = json_lexerless_grammar();
    let code_part = std::fs::read("./src/examples/__tests__/benchmark/large-file.json").unwrap();

    let serde_instant = Instant::now();

    let times = 4;

    for _ in 0..times {
        serde_json::from_slice::<SerdeValue>(&code_part).unwrap();
    }

    println!("Serde time:{:?}", serde_instant.elapsed());

    let toolkit_instance = Instant::now();

    for _ in 0..times {
        parser.tokenize_n_parse(&code_part).unwrap();
    }
    println!("Toolkit time:{:?}", toolkit_instance.elapsed());

    let lexerless_instant = Instant::now();
    for _ in 0..times {
        lexerless_parser.parse(&code_part).unwrap();
    }
    println!("Lexerless time:{:?}", lexerless_instant.elapsed());
}
