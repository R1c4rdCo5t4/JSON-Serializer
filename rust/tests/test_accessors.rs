use std::sync::mpsc;
use json_serializer::types::{Json, Accessor};
use json_serializer::json::{serialise_json, deserialise_json, eval};

#[test]
fn test_accessor_field() {
    let input_json = Json::Object(vec![
        ("name".to_string(), Json::String("Michael Scott".to_string())),
        ("age".to_string(), Json::Number(31.0)),
    ].into_iter().collect());
    
    let accessor = Accessor::Field("age".to_string(), Box::new(Accessor::End));
    let output = process_json(&input_json, &accessor);
    assert_eq!(output, Json::Number(31.0));
}

#[test]
fn test_accessor_index() {
    let input_json = Json::Array(vec![
        Json::Number(1.0),
        Json::Number(2.0),
        Json::Number(3.0),
    ]);

    let accessor = Accessor::Index(1, Box::new(Accessor::End));
    let output = process_json(&input_json, &accessor);
    assert_eq!(output, Json::Number(2.0));
}

#[test]
fn test_accessor_map() {
    let input_json = Json::Array(vec![
        Json::Number(1.0),
        Json::Number(2.0),
        Json::Number(3.0),
    ]);

    let accessor = Accessor::Map(Box::new(Accessor::End));
    let output = process_json(&input_json, &accessor);
    assert_eq!(output, input_json);
}

#[test]
fn test_accessor_map_with_field() {
    let input_json = Json::Array(vec![
        Json::Object(vec![
            ("name".to_string(), Json::String("Twitter".to_string())),
            ("link".to_string(), Json::String("https://twitter.com".to_string()))
        ].into_iter().collect()),
        Json::Object(vec![
            ("name".to_string(), Json::String("Facebook".to_string())),
            ("link".to_string(), Json::String("https://facebook.com".to_string()))
        ].into_iter().collect()),
    ]);

    let accessor = Accessor::Map(Box::new(
        Accessor::Field("name".to_string(), Box::new(Accessor::End))
    ));

    let output = process_json(&input_json, &accessor);
    let expected = Json::Array(vec![
        Json::String("Twitter".to_string()),
        Json::String("Facebook".to_string()),
    ]);
    assert_eq!(output, expected);
}

fn process_json(input_json: &Json, accessor: &Accessor) -> Json {
    // channel for serialization
    let (tx1, rx1) = mpsc::channel();
    
    // serialize
    serialise_json(input_json, tx1);
    
    // channel for the evaluation
    let (tx2, rx2) = mpsc::channel();
    
    // evaluate
    eval(accessor, rx1, tx2);
    
    // deserialize
    deserialise_json(rx2)
}