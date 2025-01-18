use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use json_serializer::types::{Json, Accessor};
use json_serializer::json::{serialise_json, deserialise_json, eval};

fn main() {

    // build json object
    let mut address_map = HashMap::new();
    address_map.insert("city".to_string(), Json::String("New York".to_string()));
    address_map.insert("postalCode".to_string(), Json::Number(64780.0));
    address_map.insert("Country".to_string(), Json::String("USA".to_string()));

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

    let mut map = HashMap::new();
    map.insert("name".to_string(), Json::String("Jason Ray".to_string()));
    map.insert("profession".to_string(), Json::String("Software Engineer".to_string()));
    map.insert("age".to_string(), Json::Number(31.0));
    map.insert("address".to_string(), Json::Object(address_map));
    map.insert(
        "languages".to_string(), 
        Json::Array(vec![
            Json::String("Java".to_string()),
            Json::String("Node.js".to_string()),
            Json::String("JavaScript".to_string()),
            Json::String("JSON".to_string()),
        ])
    );
    map.insert("socialProfiles".to_string(), social_profiles_map);

    let json_object = Json::Object(map);

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
    let json_for_serializer = json_object.clone();
    let serializer_thread = thread::spawn(move || {
        serialise_json(&json_for_serializer, json_stream_sender);
    });

    // channel from the evaluator to the deserializer
    let (transformed_stream_sender, transformed_stream_receiver) = mpsc::channel();

    // spawn the evaluator thread
    let accessor_for_eval = accessor.clone();
    let evaluator_thread = thread::spawn(move || {
        eval(&accessor_for_eval, json_stream_receiver, transformed_stream_sender);
    });

    // spawn the deserializer and printer thread
    let deserializer_thread = thread::spawn(move || {
        let result = deserialise_json(transformed_stream_receiver);
        println!("Result:\n{:#?}", result);
    });

    // wait for threads to finish
    serializer_thread.join().unwrap();
    evaluator_thread.join().unwrap();
    deserializer_thread.join().unwrap();
}
