use crate::examples::json::lexerless::{json_lexerless_grammar, JSONNode};

#[test]
pub fn test1() {
    let parser = json_lexerless_grammar();

    let code_part = r#"
            [{"a":"A","b":"B"},{"c":"C","d":"D"}]
        "#;

    match parser.parse(code_part.as_bytes()) {
        Ok(s) => {
            s[0].print().unwrap();
            assert!(s[0].contains(&JSONNode::Array), "should contain array");
            assert!(
                s[0].contains(&JSONNode::Object),
                "should contain json object"
            );
        }
        Err(err) => {
            println!("Failed part:{}", &code_part[err.pointer..]);
            panic!("{:?}", err);
        }
    }

    // println!("{:#?}", parsed_tree);
}
#[test]
pub fn test2() {
    let parser = json_lexerless_grammar();

    let code_part = r#"
    {
        "quiz": {
            "sport": {
                "q1": {
                    "question": "Which one is correct team name in NBA?",
                    "options": [
                        "New York Bulls",
                        "Los Angeles Kings",
                        "Golden State Warriros",
                        "Huston Rocket"
                    ],
                    "answer": "Huston Rocket"
                }
            },
            "maths": {
                "q1": {
                    "question": "5 + 7 = ?",
                    "options": [
                        "10",
                        "11",
                        "12",
                        "13"
                    ],
                    "answer": "12"
                },
                "q2": {
                    "question": "12 - 8 = ?",
                    "options": [
                        "1",
                        "2",
                        "3",
                        "4"
                    ],
                    "answer": "4"
                }
            }
        }
    }
        "#;

    match parser.parse(code_part.as_bytes()) {
        Ok(s) => {
            s[0].print().unwrap();
            assert!(s[0].contains(&JSONNode::Array), "should contain array");
            assert!(
                s[0].contains(&JSONNode::Object),
                "should contain json object"
            );
        }
        Err(err) => {
            println!("Failed part:{}", &code_part[err.pointer..]);
            panic!("{:?}", err);
        }
    }

    // println!("{:#?}", parsed_tree);
}
