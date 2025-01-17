use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
use crate::types::{Json, JC};

pub fn serialise_json(value: &Json, sender: Sender<JC>) {
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
                serialise_json(elem, arr_sender);
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
                serialise_json(value, key_value_sender);
            }
            sender.send(JC::EndObject).unwrap();
        }
    }
}

pub fn deserialise_json(receiver: std::sync::mpsc::Receiver<JC>) -> Json {
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
                    JC::Element(value_receiver) => array.push(deserialise_json(value_receiver)),
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
                        let value = deserialise_json(key_value_receiver);

                        object.insert(key, value);
                    }
                    _ => panic!("Unexpected token in object deserialization"),
                }
            }
        }
        _ => panic!("Unexpected token in deserialization"),
    }
}