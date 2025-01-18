use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use crate::types::JC;

// copies json structure from src channel to dest channel
pub fn clone_json_stream(src: &Receiver<JC>, dest: &Sender<JC>) {
    match src.recv().unwrap() {
        JC::Null => dest.send(JC::Null).unwrap(),
        JC::Bool(bool) => dest.send(JC::Bool(bool)).unwrap(),
        JC::Number(num) => dest.send(JC::Number(num)).unwrap(),
        JC::String(str) => dest.send(JC::String(str)).unwrap(),
        JC::BeginArray => {
            dest.send(JC::BeginArray).unwrap();
            loop {
                match src.recv().unwrap() {
                    JC::EndArray => {
                        dest.send(JC::EndArray).unwrap();
                        break;
                    }
                    JC::Element(element_receiver) => {
                        // forward element
                        let (output_element_sender, output_element_receiver) = mpsc::channel();
                        dest.send(JC::Element(output_element_receiver)).unwrap();

                        // copy the element
                        clone_json_stream(&element_receiver, &output_element_sender);
                    }
                    _ => panic!("Unexpected token in JSON stream copy")
                }
            }
        }
        JC::BeginObject => {
            dest.send(JC::BeginObject).unwrap();
            loop {
                match src.recv().unwrap() {
                    JC::EndObject => {
                        dest.send(JC::EndObject).unwrap();
                        break;
                    }
                    JC::Element(field_receiver) => {
                        // forward field
                        let (output_field_sender, output_field_receiver) = mpsc::channel();
                        dest.send(JC::Element(output_field_receiver)).unwrap();

                        // first element (key) should be a string
                        let key = field_receiver.recv().unwrap();
                        if let JC::String(field_name) = key {
                            // forward key
                            output_field_sender.send(JC::String(field_name)).unwrap();

                            // copy value
                            clone_json_stream(&field_receiver, &output_field_sender);
                        } else {
                            panic!("Expected object key string, got: {:?}", key);
                        }
                    }
                    _ => panic!("Unexpected token in JSON stream copy")
                }
            }
        }
        _ => {
            panic!("Unexpected token in JSON stream copy")
        }
    }
}

// consumes json stream from channel
pub fn consume_json_stream(receiver: &Receiver<JC>) {
    match receiver.recv().unwrap() {
        JC::Null | JC::Bool(_) | JC::Number(_) | JC::String(_) => {} // consume values
        JC::BeginArray => {
            loop {
                match receiver.recv().unwrap() {
                    JC::EndArray => break,
                    JC::Element(element_receiver) => {
                        consume_json_stream(&element_receiver);
                    }
                    _ => panic!("Unexpected token in JSON stream consumption")
                }
            }
        }
        JC::BeginObject => {
            loop {
                match receiver.recv().unwrap() {
                    JC::EndObject => break,
                    JC::Element(field_receiver) => {
                        let key = field_receiver.recv().unwrap();
                        if let JC::String(_field_name) = key {
                            consume_json_stream(&field_receiver); // discard value
                        } else {
                            panic!("Expected object key string, got: {:?}", key);
                        }
                    }
                    _ => panic!("Unexpected token in JSON stream consumption")
                }
            }
        }
        _ => panic!("Unexpected token in JSON stream consumption")
    }
}