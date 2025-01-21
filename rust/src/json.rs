use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
use crate::types::{Json, JC, Accessor};
use crate::utils::{clone_json_stream, consume_json_stream};

/**
 * Serializes a JSON object into a stream of JC tokens
 */
pub fn serialize_json(value: &Json, sender: Sender<JC>) {
    match value {
        Json::Null => sender.send(JC::Null).unwrap(),
        Json::Bool(bool) => sender.send(JC::Bool(*bool)).unwrap(),
        Json::Number(num) => sender.send(JC::Number(*num)).unwrap(),
        Json::String(str) => sender.send(JC::String(str.clone())).unwrap(),
        Json::Array(arr) => {
            sender.send(JC::BeginArray).unwrap();
            for elem in arr {
                let (arr_sender, arr_receiver) = mpsc::channel();
                sender.send(JC::Element(arr_receiver)).unwrap();

                // send element
                serialize_json(elem, arr_sender);
            }
            sender.send(JC::EndArray).unwrap();
        }
        Json::Object(obj) => {
            sender.send(JC::BeginObject).unwrap();
            for (key, value) in obj {
                let (key_value_sender, key_value_receiver) = mpsc::channel::<JC>();
                sender.send(JC::Element(key_value_receiver)).unwrap();

                // send key
                key_value_sender.send(JC::String(key.clone())).unwrap();

                // send value
                serialize_json(value, key_value_sender);
            }
            sender.send(JC::EndObject).unwrap();
        }
    }
}

/**
 * Deserializes a stream of JC tokens into a JSON object
 */
pub fn deserialize_json(receiver: std::sync::mpsc::Receiver<JC>) -> Json {
    match receiver.recv().unwrap() {
        JC::Null => Json::Null,
        JC::Bool(bool) => Json::Bool(bool),
        JC::Number(num) => Json::Number(num),
        JC::String(str) => Json::String(str),
        JC::BeginArray => {
            let mut array = Vec::new();
            loop {
                match receiver.recv().unwrap() {
                    JC::EndArray => return Json::Array(array),
                    JC::Element(value_receiver) => array.push(deserialize_json(value_receiver)),
                    _ => panic!("Unexpected token in array deserialization"),
                }
            }
            
        }
        JC::BeginObject => {
            let mut object = HashMap::new();
            loop {
                match receiver.recv().unwrap() {
                    JC::EndObject => return Json::Object(object),
                    JC::Element(key_value_receiver) => {
                        // get key
                        let key = match key_value_receiver.recv().unwrap() {
                            JC::String(s) => s,
                            _ => panic!("Unexpected object key"),
                        };
                        // get value
                        let value = deserialize_json(key_value_receiver);

                        object.insert(key, value);
                    }
                    _ => panic!("Unexpected token in object deserialization"),
                }
            }
        }
        _ => panic!("Unexpected token in deserialization"),
    }
}

/**
 * Evaluates an accessor on a JSON stream
 */
pub fn eval(accessor: &Accessor, receiver: mpsc::Receiver<JC>, sender: mpsc::Sender<JC>) {
   match accessor {
        // end of the accessor
        Accessor::End => {
            clone_json_stream(&receiver, &sender);
        }
        // select the field from the object
        Accessor::Field(field, next) => {
            match receiver.recv().unwrap() {
                JC::BeginObject => {
                    let mut found = false;
                    loop {
                        match receiver.recv().unwrap() {
                            JC::EndObject => {
                                if !found {
                                    panic!("Field not found: {}", field);
                                }
                                break;
                            }
                            JC::Element(key_value_receiver) => {
                                let key = match key_value_receiver.recv().unwrap() {
                                    JC::String(s) => s,
                                    _ => panic!("Unexpected object key, must be string"),
                                };
                                if key == *field {
                                    // field found
                                    found = true;
                                    eval(next, key_value_receiver, sender.clone());
                                } else {
                                    // discard value
                                    consume_json_stream(&key_value_receiver);
                                }
                            }
                            _ => panic!("Unexpected token in object deserialization"),
                        }
                    }
                }
                other => panic!("Expected object, got: {:?}", other),
            }
        }
        // select the element from the array with the given index
        Accessor::Index(index, next) => { 
            match receiver.recv().unwrap() {
                JC::BeginArray => {
                    let mut found = false;
                    let mut i = 0;
                    loop{
                        match receiver.recv().unwrap() {
                            JC::EndArray => {
                                if !found {
                                    panic!("Index out of bounds: {}", index);
                                }
                                break;
                            }
                            JC::Element(value_receiver) => {
                                if i == *index {
                                    found = true;
                                    eval(next, value_receiver, sender.clone());
                                } else {
                                    consume_json_stream(&value_receiver);
                                }
                                i += 1;
                            }
                            _ => panic!("Unexpected token in array deserialization"),
                        }
                    }
                }
                other => panic!("Expected array, got: {:?}", other),
            }
        }
        // map array elements
        Accessor::Map(next) => {
            match receiver.recv().unwrap() {
                JC::BeginArray => {
                    sender.send(JC::BeginArray).unwrap();
                    loop {
                        match receiver.recv().unwrap() {
                            JC::EndArray => {
                                sender.send(JC::EndArray).unwrap();
                                break;
                            }
                            JC::Element(value_receiver) => {
                                let (mapped_sender, mapped_receiver) = mpsc::channel();
                                sender.send(JC::Element(mapped_receiver)).unwrap();

                                // map element
                                eval(next, value_receiver, mapped_sender);
                            }
                            _ => panic!("Unexpected token in array deserialization"),
                        }
                    }
                }
                other => panic!("Expected array, got: {:?}", other),
            }
        }
   }

}