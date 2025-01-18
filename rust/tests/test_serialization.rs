use std::collections::HashMap;
use std::sync::mpsc;
use json_serializer::types::Json;
use json_serializer::json::{serialise_json, deserialise_json};

#[test]
fn test_serialization() {

    let mut inner_object = HashMap::new();
    inner_object.insert("x".to_string(), Json::Number(42.0));
    inner_object.insert("y".to_string(), Json::Bool(true));

    let sample = Json::Object({
        let mut map = HashMap::new();
    
        map.insert("message".to_string(), Json::String("Hello World".to_string()));
        map.insert("nested".to_string(), Json::Object(inner_object));
        map.insert(
            "array".to_string(),
            Json::Array(vec![
                Json::Number(1.0),
                Json::Number(2.0),
                Json::Number(3.0),
            ]),
        );
        map.insert(
            "more arrays".to_string(),
            Json::Array(vec![
                Json::Array(vec![Json::Number(1.0), Json::Number(2.0), Json::Number(3.0)]),
                Json::Array(vec![Json::Number(4.0), Json::Number(5.0), Json::Number(6.0)]),
            ]),
        );
        map
    });
    
    let (sender, receiver) = mpsc::channel();
    serialise_json(&sample, sender);
    let deserialized = deserialise_json(receiver);
    assert_eq!(sample, deserialized);
}