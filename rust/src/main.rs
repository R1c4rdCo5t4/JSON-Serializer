use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use json_serializer::types::{Json, Accessor};
use json_serializer::json::{serialize_json, deserialize_json, eval};

fn main() {

    // build json object
    let mut address = HashMap::new();
    address.insert("city".to_string(), Json::String("New York".to_string()));
    address.insert("postalCode".to_string(), Json::Number(64780.0));
    address.insert("Country".to_string(), Json::String("USA".to_string()));

    let social_profiles_map = Json::Array(vec![
        Json::Object({
            let mut m = HashMap::new();
            m.insert("name".to_string(), Json::String("Twitter".to_string()));
            m.insert("link".to_string(), Json::String("https://twitter.com".to_string()));
            m
        }),
        Json::Object({
            let mut m = HashMap::new();
            m.insert("name".to_string(), Json::String("Facebook".to_string()));
            m.insert("link".to_string(), Json::String("https://www.facebook.com".to_string()));
            m 
        }),
    ]);

    let mut root = HashMap::new();
    root.insert("name".to_string(), Json::String("Jason Ray".to_string()));
    root.insert("profession".to_string(), Json::String("Software Engineer".to_string()));
    root.insert("age".to_string(), Json::Number(31.0));
    root.insert("address".to_string(), Json::Object(address));
    root.insert(
        "languages".to_string(), 
        Json::Array(vec![
            Json::String("Java".to_string()),
            Json::String("Node.js".to_string()),
            Json::String("JavaScript".to_string()),
            Json::String("JSON".to_string()),
        ])
    );
    root.insert("socialProfiles".to_string(), social_profiles_map);
    let json_object = Json::Object(root);

    // define accessor
    let accessor = Accessor::Field(
        "socialProfiles".to_string(), 
        Box::new(Accessor::Map(Box::new(
            Accessor::Field("name".to_string(), Box::new(Accessor::End))
        )))
    );

    // channel from the serializer to the evaluator
    let (json_stream_sender, json_stream_receiver) = mpsc::channel();

    // spawn the serializer thread
    let serializer_thread = thread::spawn(move || {
        serialize_json(&json_object.clone(), json_stream_sender);
    });

    // channel from the evaluator to the deserializer
    let (transformed_stream_sender, transformed_stream_receiver) = mpsc::channel();

    // spawn the evaluator thread
    let evaluator_thread = thread::spawn(move || {
        eval(&accessor.clone(), json_stream_receiver, transformed_stream_sender);
    });

    // spawn the deserializer and printer thread
    let deserializer_thread = thread::spawn(move || {
        let result = deserialize_json(transformed_stream_receiver);
        println!("Result:\n{:#?}", result);
    });

    // wait for threads to finish
    serializer_thread.join().unwrap();
    evaluator_thread.join().unwrap();
    deserializer_thread.join().unwrap();
}
