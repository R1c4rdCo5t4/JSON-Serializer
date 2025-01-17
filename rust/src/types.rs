use std::collections::HashMap;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

#[derive(Debug)]
pub enum JC {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    BeginArray,
    EndArray,
    BeginObject,
    EndObject,
    Element(mpsc::Receiver<JC>), // represents an element of an array or object
}
