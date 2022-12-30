use crate::examples::json::tokenized::json_grammar;

#[test]
fn json_simple_array_test() {
    let parser = json_grammar();

    let code_part = r#"[2,3,4]"#;

    match parser.debug_production_at("array", code_part.as_bytes(), 0) {
        Ok(s) => {
            s[0].print().unwrap();
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
fn json_simple_object_test() {
    let parser = json_grammar();

    let code_part = r#"{"a":2,"b":true,"c":"d"}"#;

    match parser.debug_production_at("object", code_part.as_bytes(), 0) {
        Ok(s) => {
            s[0].print().unwrap();
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
