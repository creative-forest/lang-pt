use crate::examples::json::lexerless::json_lexerless_grammar;

#[test]
pub fn json_array_production() {
    let parser = json_lexerless_grammar();
    println!("Ok");

    let code_part = r#"[2,3,4]"#;

    match parser.debug_parser_at("array", code_part.as_bytes(), 0) {
        Ok(s) => {
            s.last().unwrap().print().unwrap();
        }
        Err(err) => {
            println!(
                "Error:{:?}\nParsing failed at :{:?}",
                err,
                &code_part[err.pointer..]
            );
            panic!("Failed to parse")
        }
    }
}
#[test]
pub fn json_object_production() {
    let parser = json_lexerless_grammar();

    let code_part = r#"{"a":2,"b":true,"c":"d"}"#;

    match parser.debug_parser_at("object", code_part.as_bytes(), 0) {
        Ok(s) => {
            s.last().unwrap().print().unwrap();
        }
        Err(err) => {
            println!(
                "Error:{:?}\nParsing failed at :{:?}",
                err,
                &code_part[err.pointer..]
            );
            panic!("Failed to parse")
        }
    }
}
